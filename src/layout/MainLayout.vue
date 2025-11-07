<template>
  <div class="app">
    <Sidebar 
      v-show="!shouldHidePlayerChrome"
      :followedAnchors="followedStreamersFromStore" 
      @selectAnchor="handleStreamerSelect"
      @unfollow="handleUnfollowStore"
      @reorderList="handleReorderListStore"
    />
    <div class="main-content">
      <Header 
        v-show="!shouldHidePlayerChrome"
        @select-anchor="handleStreamerSelect"
        @follow="handleFollowStore"
        @unfollow="handleUnfollowStore"
      />
      <router-view 
        v-slot="{ Component, route }" 
        @follow="handleFollowStore"
        @unfollow="handleUnfollowStore"
        @fullscreen-change="handleFullscreenChange"
      >
        <transition name="fade" mode="out-in">
          <keep-alive :include="['HomeView', 'DouyinHomeView', 'HuyaHomeView', 'BilibiliHomeView']">
            <component :is="Component" :key="route.path" />
          </keep-alive>
        </transition>
      </router-view>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'
import Sidebar from './Sidebar.vue'
import Header from './Header.vue'
import { useFollowStore } from '../store/followStore'
import type { FollowedStreamer } from '../platforms/common/types'
import { Platform } from '../platforms/common/types'

const router = useRouter()
const followStore = useFollowStore()

const followedStreamersFromStore = computed(() => followStore.getFollowedStreamers)

const isPlayerFullscreen = ref(false)
const isWindowMaximized = ref(false)
const currentWindow = typeof window !== 'undefined' ? getCurrentWindow() : null
let unlistenResize: UnlistenFn | null = null

const syncWindowMaximizedState = async () => {
  if (!currentWindow) return
  try {
    isWindowMaximized.value = await currentWindow.isMaximized()
  } catch (error) {
    console.error('[MainLayout] Failed to query maximized state', error)
  }
}

onMounted(async () => {
  if (!currentWindow) return
  await syncWindowMaximizedState()
  try {
    unlistenResize = await currentWindow.onResized(syncWindowMaximizedState)
  } catch (error) {
    console.error('[MainLayout] Failed to listen for resize events', error)
  }
})

onBeforeUnmount(async () => {
  if (unlistenResize) {
    await unlistenResize()
    unlistenResize = null
  }
})

const isPlayerRoute = computed(() => {
  const name = router.currentRoute.value.name
  return (
    name === 'douyuPlayer' ||
    name === 'douyinPlayer' ||
    name === 'huyaPlayer' ||
    name === 'bilibiliPlayer'
  )
})

const shouldHidePlayerChrome = computed(() => {
  return isPlayerRoute.value && (isPlayerFullscreen.value || isWindowMaximized.value)
})

const handleStreamerSelect = (streamer: FollowedStreamer) => {
  let routeName = '';
  if (streamer.platform === Platform.DOUYU) {
    routeName = 'douyuPlayer';
  } else if (streamer.platform === Platform.DOUYIN) {
    routeName = 'douyinPlayer';
  } else if (streamer.platform === Platform.HUYA) {
    routeName = 'huyaPlayer';
  } else if (streamer.platform === Platform.BILIBILI) {
    routeName = 'bilibiliPlayer';
  } else {
    console.error('Unsupported platform for player:', streamer.platform);
    return;
  }

  router.push({
    name: routeName,
    params: {
      roomId: streamer.id,
    },
  });
}

const handleFollowStore = (streamer: FollowedStreamer) => {
  followStore.followStreamer(streamer)
}

const handleUnfollowStore = (payload: {platform: Platform, id: string} | string) => {
  if (typeof payload === 'string') {
    followStore.unfollowStreamer(Platform.DOUYU, payload)
  } else {
    followStore.unfollowStreamer(payload.platform, payload.id)
  }
}

const handleReorderListStore = (reorderedList: FollowedStreamer[]) => {
  followStore.updateOrder(reorderedList)
}

const handleFullscreenChange = (isFullscreen: boolean) => {
  isPlayerFullscreen.value = isFullscreen
}
</script>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  background: var(--main-bg);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 全屏模式时隐藏UI */
.app.hide-ui > :not(.main-content) {
  display: none !important;
}

.app.hide-ui .main-content > :not(.player-view-container) {
  display: none !important;
}

.app.hide-ui {
  background: transparent !important;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
