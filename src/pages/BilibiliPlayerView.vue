<template>
  <div class="player-view">
    <MainPlayer
      :key="playerKey"
      :room-id="props.roomId"
      :platform="Platform.BILIBILI"
      :is-followed="isFollowed"
      :cookie="cookieInput"
      @follow="handleFollow"
      @unfollow="handleUnfollow"
      @close-player="handleClosePlayer"
      @fullscreen-change="handlePlayerFullscreenChange"
      @request-player-reload="handlePlayerReload"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import MainPlayer from '../components/player/index.vue';
import { useFollowStore } from '../store/followStore';
import type { FollowedStreamer } from '../platforms/common/types';
import { Platform } from '../platforms/common/types';
import { ensureBilibiliCookieBootstrap } from '../platforms/bilibili/cookieHelper';

const props = defineProps<{ roomId: string }>();
const emit = defineEmits(['fullscreen-change']);

const router = useRouter();
const followStore = useFollowStore();
const playerKey = ref(0);
const cookieInput = ref<string>('');

onMounted(async () => {
  if (typeof localStorage === 'undefined') {
    return;
  }

  let stored = (localStorage.getItem('bilibili_cookie') || '').trim();
  if (stored) {
    cookieInput.value = stored;
  }

  const result = await ensureBilibiliCookieBootstrap();
  if (!stored && result?.cookie) {
    stored = result.cookie.trim();
    if (stored) {
      localStorage.setItem('bilibili_cookie', stored);
      cookieInput.value = stored;
    }
  }
});

const isFollowed = computed(() => {
  return followStore.isFollowed(Platform.BILIBILI, props.roomId);
});

const handleFollow = () => {
  const streamerToFollow: Omit<FollowedStreamer, 'platform' | 'id' | 'roomTitle' | 'isLive'> = {
    nickname: `主播${props.roomId}`,
    avatarUrl: '',
  };

  followStore.followStreamer({
    ...streamerToFollow,
    id: props.roomId,
    platform: Platform.BILIBILI,
  });
};

const handleUnfollow = () => {
  followStore.unfollowStreamer(Platform.BILIBILI, props.roomId);
};

const handleClosePlayer = () => {
  router.replace('/bilibili');
};

const handlePlayerFullscreenChange = (isFullscreen: boolean) => {
  emit('fullscreen-change', isFullscreen);
};

const handlePlayerReload = () => {
  playerKey.value++;
};
</script>

<style scoped>
.player-view {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-height: 0;
  width: 100%;
  background-color: #0e0e10;
  color: white;
}
</style>
