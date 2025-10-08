<template>
  <div class="player-view">
    <MainPlayer
      :key="playerKey"
      :room-id="props.roomId"
      :platform="Platform.DOUYIN"
      :is-followed="isFollowed"
      @follow="handleFollow"
      @unfollow="handleUnfollow"
      @close-player="handleClosePlayer"
      @fullscreen-change="handlePlayerFullscreenChange"
      @request-player-reload="handlePlayerReload"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import MainPlayer from '../components/player/index.vue';
import { useFollowStore } from '../store/followStore';
import type { FollowedStreamer } from '../platforms/common/types';
import { Platform } from '../platforms/common/types';

const props = defineProps<{
  roomId: string;
}>();

const emit = defineEmits(['fullscreen-change']);

const router = useRouter();
const followStore = useFollowStore();
const playerKey = ref(0);

// 统一规范化为 room_id（如果传入的是短ID）
const normalizedRoomId = ref<string | null>(null);

type DouyinFollowListRoomInfo = {
  room_id_str: string;
  nickname: string;
  room_name: string;
  avatar_url: string;
  status: number;
};

async function normalizeRoomId() {
  const inputId = props.roomId;
  if (!inputId) {
    normalizedRoomId.value = null;
    return;
  }
  try {
    // 后端会根据长度自动处理 webRid 或 room_id
    const info = await invoke<DouyinFollowListRoomInfo>('fetch_douyin_room_info', { live_id: inputId });
    if (info && info.room_id_str) {
      normalizedRoomId.value = info.room_id_str;
      console.log('[DouyinPlayerView] normalizedRoomId ->', normalizedRoomId.value);
    } else {
      normalizedRoomId.value = inputId; // 回退
      console.warn('[DouyinPlayerView] fetch_douyin_room_info 返回为空，使用原始ID作为 room_id:', inputId);
    }
  } catch (e) {
    normalizedRoomId.value = inputId; // 回退
    console.warn('[DouyinPlayerView] 规范化 room_id 失败，使用原始ID:', inputId, e);
  }
}

// 初始化与监听路由参数变化
normalizeRoomId();
watch(() => props.roomId, () => {
  normalizeRoomId();
});

const isFollowed = computed(() => {
  const idToCheck = normalizedRoomId.value || props.roomId;
  return followStore.isFollowed(Platform.DOUYIN, idToCheck);
});

const handleFollow = () => {
  const idToSave = normalizedRoomId.value || props.roomId;
  const streamerToFollow: Omit<FollowedStreamer, 'platform' | 'id' | 'roomTitle' | 'isLive'> = {
    nickname: `主播${idToSave}`,
    avatarUrl: '',
  };

  console.log('[DouyinPlayerView] 即将写入关注缓存：', {
    platform: Platform.DOUYIN,
    id: idToSave,
    ...streamerToFollow,
  });

  followStore.followStreamer({
    ...streamerToFollow,
    id: idToSave,
    platform: Platform.DOUYIN,
  });

  // 查看 localStorage 中的缓存内容
  const cached = localStorage.getItem('followedStreamers');
  console.log('[DouyinPlayerView] 当前 localStorage.followedStreamers：', cached);
};

const handleUnfollow = () => {
  const idToRemove = normalizedRoomId.value || props.roomId;
  followStore.unfollowStreamer(Platform.DOUYIN, idToRemove);
};

const handleClosePlayer = () => {
  console.log('[DouyinPlayerView] Close button clicked. Navigating to Douyin home.');
  router.replace('/douyin');
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
  flex-direction: column;
  height: 100%;
  background-color: #0e0e10;
  color: white;
}
</style>