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

const props = defineProps<{ roomId: string }>();
const emit = defineEmits(['fullscreen-change']);

const router = useRouter();
const followStore = useFollowStore();
const playerKey = ref(0);
const cookieInput = ref<string>('');

onMounted(() => {
  const saved = localStorage.getItem('bilibili_cookie');
  if (saved) cookieInput.value = saved;
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
  flex-direction: column;
  height: 100%;
  background-color: #0e0e10;
  color: white;
}
.cookie-panel {
  padding: 10px;
}
.cookie-panel details {
  background: #1f1f23;
  border-radius: 8px;
  padding: 8px;
}
.cookie-panel textarea {
  width: 100%;
  min-height: 80px;
  margin-top: 8px;
  border-radius: 6px;
  border: 1px solid #333;
  background: #0e0e10;
  color: #ddd;
}
.cookie-actions {
  display: flex;
  gap: 10px;
  margin-top: 8px;
}
.cookie-actions button {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background: #9147ff;
  color: white;
  cursor: pointer;
}
.cookie-tip {
  color: #aaa;
  font-size: 12px;
  margin-top: 6px;
}
</style>