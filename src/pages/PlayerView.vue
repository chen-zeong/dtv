<template>
  <div class="player-view">
    <MainPlayer v-if="roomId" :platform="Platform.DOUYU" :room-id="roomId" :is-followed="isFollowed" @follow="handleFollow" @unfollow="handleUnfollow" @close-player="handleClosePlayer" />
    <div v-else>
      <p>无效的房间ID。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import MainPlayer from '../components/player/index.vue'; // Assuming MainPlayer is in components/player
import { useFollowStore } from '../store/followStore'; // Placeholder for follow state
import type { FollowedStreamer } from '../platforms/common/types'; // Keep as type-only import
import { Platform } from '../platforms/common/types'; // Regular import for enum

const props = defineProps<{
  roomId: string;
}>();

const router = useRouter(); // Initialize router
const followStore = useFollowStore(); // Placeholder

const isFollowed = computed(() => {
  return followStore.isFollowed(Platform.DOUYU, props.roomId);
});

const handleFollow = (streamerData: Omit<FollowedStreamer, 'platform'>) => {
  followStore.followStreamer({ ...streamerData, platform: Platform.DOUYU, id: props.roomId });
};

const handleUnfollow = (platformId: string) => {
  followStore.unfollowStreamer(Platform.DOUYU, platformId);
  console.log('PlayerView: Unfollowed', platformId);
};

const handleClosePlayer = () => {
  console.log('PlayerView: Close player event received. Navigating back.');
  router.back();
};

</script>

<style scoped>
.player-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #0e0e10; /* Twitch-like dark background */
}
</style> 