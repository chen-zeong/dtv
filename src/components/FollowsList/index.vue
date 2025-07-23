<template>
    <div class="follow-list">
      <div class="list-header">
        <h3 class="header-title">关注列表</h3>
        <div class="header-actions">
          <button 
            @click="refreshList" 
            class="action-btn refresh-btn"
            :disabled="isRefreshing"
            title="刷新列表"
          >
            <span class="icon" :class="{ 'refreshing': isRefreshing }">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
                <path d="M3 3v5h5"></path>
              </svg>
            </span>
          </button>
        </div>
      </div>
      
      <div class="list-content" ref="listRef">
        <div v-if="streamers.length === 0" class="empty-state">
          <div class="empty-image">
            <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="feather feather-heart">
              <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"></path>
            </svg>
          </div>
          <h3 class="empty-title">暂无关注主播</h3>
          <p class="empty-text">关注主播后，他们会出现在这里</p>
        </div>
        
        <TransitionGroup 
          v-else 
          tag="ul" 
          name="streamer-list"
          class="streamers-list"
        >
          <li
            v-for="(streamer, index) in streamers"
            :key="streamer.id"
            class="streamer-item"
            :class="[
              getStreamerItemClass(streamer),
              { 
                'is-dragging': isDragging && draggedIndex === index,
                'just-added': justAddedIds.includes(streamer.id)
              }
            ]"
            @mousedown="handleMouseDown($event, index)"
            @click="handleClick($event, streamer)"
          >
            <div class="item-content">
              <div class="avatar-container">
                <img 
                  v-if="streamer.avatarUrl" 
                  :src="streamer.avatarUrl" 
                  :alt="streamer.nickname"
                  class="avatar-image"
                >
                <div v-else class="avatar-fallback">{{ streamer.nickname[0] }}</div>
              </div>
              
              <div class="streamer-details">
                <div class="primary-row">
                  <span class="nickname" :title="streamer.nickname">{{ streamer.nickname }}</span>
                </div>
                
                <div class="secondary-row" :title="streamer.roomTitle">
                  {{ streamer.roomTitle || '暂无直播标题' }}
                </div>
              </div>
            </div>
            
            <div class="status-container">
              <div class="live-indicator" :class="getLiveIndicatorClass(streamer)"></div>
            </div>
          </li>
        </TransitionGroup>
      </div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, onMounted, computed, watch, onUnmounted } from 'vue';
  import type { FollowedStreamer, LiveStatus } from '../../platforms/common/types';
  import { Platform } from '../../platforms/common/types';
  // import type { DouyuRoomInfo } from '../../platforms/douyu/types'; // No longer needed here
  // import type { DouyinRoomInfo } from './types'; // No longer defined here

  import { refreshDouyuFollowedStreamer } from '../../platforms/douyu/followListHelper';
  import { refreshDouyinFollowedStreamer } from '../../platforms/douyin/followListHelper';
  
  // Updated DouyinRoomInfo to match the Rust struct DouyinFollowListRoomInfo
  // interface DouyinRoomInfo { // This will be the type for `data` from invoke
  
  const props = defineProps<{
    followedAnchors: FollowedStreamer[]
  }>();
  
  const emit = defineEmits<{
    (e: 'selectAnchor', streamer: FollowedStreamer): void;
    (e: 'unfollow', payload: { platform: Platform, id: string }): void; // Ensure Platform type is used here if not already
    (e: 'reorderList', newList: FollowedStreamer[]): void;
  }>();
  
  const isRefreshing = ref(false);
  const listRef = ref<HTMLElement | null>(null);
  const isDragging = ref(false);
  const draggedIndex = ref(-1);
  const startY = ref(0);
  const currentY = ref(0);
  const justAddedIds = ref<string[]>([]);
  const animationTimeout = ref<number | null>(null);
  
  const MIN_ANIMATION_DURATION = 1500;
  
  const getLiveStatusSortOrder = (status: LiveStatus | undefined): number => {
    switch (status) {
      case 'LIVE': return 1;
      case 'REPLAY': return 2;
      case 'OFFLINE': return 3;
      case 'UNKNOWN': return 4;
      default: return 5; // Should not happen with defined types
    }
  };

  const streamers = computed(() => props.followedAnchors);
  
  // Method to determine class for the list item itself
  const getStreamerItemClass = (streamer: FollowedStreamer) => {
    return {
      'status-live': streamer.liveStatus === 'LIVE',
      'status-replay': streamer.liveStatus === 'REPLAY',
      'status-offline': streamer.liveStatus === 'OFFLINE' || !streamer.liveStatus || streamer.liveStatus === 'UNKNOWN',
    };
  };

  // Method to determine class for the live indicator dot
  const getLiveIndicatorClass = (streamer: FollowedStreamer) => {
    switch (streamer.liveStatus) {
      case 'LIVE':
        return 'is-live'; // Existing class for green
      case 'REPLAY':
        return 'is-replay'; // New class for yellow
      case 'OFFLINE':
      case 'UNKNOWN':
      default:
        return 'is-offline'; // New or existing class for gray/default
    }
  };
  
  watch(() => props.followedAnchors, (newVal, oldVal) => {
    if (!oldVal || oldVal.length === 0) return;
    
    const oldIds = oldVal.map(streamer => streamer.id);
    const newStreamers = newVal.filter(streamer => !oldIds.includes(streamer.id));
    
    if (newStreamers.length > 0) {
      newStreamers.forEach(streamer => {
        justAddedIds.value.push(streamer.id);
        setTimeout(() => {
          justAddedIds.value = justAddedIds.value.filter(id => id !== streamer.id);
        }, 3000);
      });
    }
  }, { deep: true });
  
  const handleClick = (e: MouseEvent, streamer: FollowedStreamer) => {
    if (isDragging.value && draggedIndex.value !== -1) {
      e.preventDefault();
      return;
    }
    emit('selectAnchor', streamer);
  };
  
  const handleMouseDown = (e: MouseEvent, index: number) => {
    if (e.button !== 0) return;
    
    isDragging.value = true;
    draggedIndex.value = index;
    startY.value = e.clientY;
    currentY.value = e.clientY; 
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    
    e.preventDefault();
  };
  
  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging.value || draggedIndex.value === -1) return;
    
    currentY.value = e.clientY;
    const container = listRef.value?.querySelector('.streamers-list');
    if (!container) return;
    
    const items = Array.from(container.children) as HTMLElement[];
    const draggedItem = items[draggedIndex.value];
    if (!draggedItem) return;
    
    const moveY = currentY.value - startY.value;
    const itemHeight = draggedItem.offsetHeight;
    
    let targetIndex = draggedIndex.value;
    let accumulatedHeight = 0;
    if (moveY > 0) {
      for (let i = draggedIndex.value + 1; i < items.length; i++) {
        accumulatedHeight += items[i].offsetHeight;
        if (moveY < accumulatedHeight) break;
        targetIndex = i;
      }
    } else {
      for (let i = draggedIndex.value - 1; i >= 0; i--) {
        accumulatedHeight -= items[i].offsetHeight;
        if (moveY > accumulatedHeight) break;
        targetIndex = i;
      }
    }
  
    if (targetIndex !== draggedIndex.value) {
      const reorderedStreamers = [...streamers.value];
      const [removed] = reorderedStreamers.splice(draggedIndex.value, 1);
      reorderedStreamers.splice(targetIndex, 0, removed);
      
      emit('reorderList', reorderedStreamers);
      
      draggedIndex.value = targetIndex;
      startY.value = e.clientY - (targetIndex - draggedIndex.value) * itemHeight;
    }
  };
  
  const handleMouseUp = () => {
    if (!isDragging.value) return;
    
    isDragging.value = false;
    draggedIndex.value = -1;
    
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  };
  
  const clearAnimationTimeout = () => {
    if (animationTimeout.value !== null) {
      clearTimeout(animationTimeout.value);
      animationTimeout.value = null;
    }
  };
  
  const refreshList = async () => {
    if (isRefreshing.value) return;
    const startTime = Date.now();
    isRefreshing.value = true;
    
    try {
      const updates = await Promise.all(
        props.followedAnchors.map(async (streamer) => {
          let updatedStreamerData: Partial<FollowedStreamer> = {};
          try {
            if (streamer.platform === Platform.DOUYU) {
              updatedStreamerData = await refreshDouyuFollowedStreamer(streamer);
            } else if (streamer.platform === Platform.DOUYIN) {
              updatedStreamerData = await refreshDouyinFollowedStreamer(streamer);
            } else {
              console.warn(`Unsupported platform for refresh: ${streamer.platform}`);
              return streamer;
            }
            return {
              ...streamer,
              ...updatedStreamerData,
            } as FollowedStreamer;
          } catch (e) {
            console.error(`[FollowsList] Error during refresh for ${streamer.platform}/${streamer.id}, returning original:`, e);
            return streamer; 
          }
        })
      );

      const validUpdates = updates.filter((update: FollowedStreamer | undefined): update is FollowedStreamer => !!update && typeof update.id !== 'undefined');
      
      if (validUpdates.length > 0) {
        const sortedUpdates = [...validUpdates].sort((a, b) => {
          const statusOrderA = getLiveStatusSortOrder(a.liveStatus);
          const statusOrderB = getLiveStatusSortOrder(b.liveStatus);
          return statusOrderA - statusOrderB;
        });
        
        const hasChanged = JSON.stringify(sortedUpdates) !== JSON.stringify(props.followedAnchors);

        if (hasChanged) {
          emit('reorderList', sortedUpdates); 
        }
      }
    } finally {
      const elapsedTime = Date.now() - startTime;
      if (elapsedTime < MIN_ANIMATION_DURATION) {
        clearAnimationTimeout();
        animationTimeout.value = window.setTimeout(() => {
          isRefreshing.value = false;
          animationTimeout.value = null;
        }, MIN_ANIMATION_DURATION - elapsedTime);
      } else {
        isRefreshing.value = false;
      }
    }
  };
  
  onMounted(() => {
    refreshList();
  });
  
  onUnmounted(() => {
    clearAnimationTimeout();
  });
  </script>
  
  <style scoped>
  .follow-list {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--sidebar-bg-dark, #1a1b1e);
    border-radius: 8px;
    padding: 8px;
    box-sizing: border-box;
    overflow: hidden;
  }
  
  /* Light Theme Overrides for Follow List itself */
  :root[data-theme="light"] .follow-list {
    background: var(--sidebar-bg-light, #f6f6f6);
  }
  
  /* Header styles */
  .list-header {
    padding: 8px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--border-color-softer, rgba(255, 255, 255, 0.05));
    background: transparent;
    flex-shrink: 0;
    border-radius: 0;
  }
  
  :root[data-theme="light"] .list-header {
    border-bottom-color: var(--border-color-light-softer, rgba(0,0,0,0.05));
  }
  
  :root[data-theme="light"] .list-header .header-title {
    color: var(--text-primary-light, #2d3748);
  }
  :root[data-theme="light"] .list-header .action-btn {
    color: var(--text-secondary-light, #718096);
  }
  :root[data-theme="light"] .list-header .action-btn:hover {
    color: var(--text-primary-light, #2d3748);
    background-color: var(--control-hover-bg-light, #edf2f7);
  }
  
  .header-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--primary-text, #ffffff);
    margin: 0;
    line-height: 1.2;
  }
  
  .header-actions {
    display: flex;
    gap: 8px;
  }
  
  .action-btn {
    background: rgba(255, 255, 255, 0.05);
    border: none;
    border-radius: 8px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--secondary-text, rgba(255, 255, 255, 0.6));
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
  
  .action-btn:hover {
    background: var(--button-hover-bg, rgba(255, 255, 255, 0.15));
    color: var(--primary-text, #ffffff);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
  
  .action-btn:active {
    transform: scale(0.95);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  }
  
  /* Night mode specific hover/active for refresh button */
  :root[data-theme="dark"] .refresh-btn:not(:disabled):hover {
    background-color: rgba(255, 255, 255, 0.2);
    color: #ffffff;
  }
  
  :root[data-theme="dark"] .refresh-btn:not(:disabled):active {
    background-color: rgba(255, 255, 255, 0.25);
    color: #ffffff;
  }
  
  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .icon.refreshing {
    color: rgba(80, 130, 255, 0.9);
    animation: spin 2s linear infinite;
  }
  
  .refresh-btn {
    background: rgba(255, 255, 255, 0.05);
  }
  
  .refresh-btn:disabled {
    background: rgba(80, 130, 255, 0.15); /* This purplish background is for when it's actually refreshing/disabled */
    cursor: default;
    transform: none;
  }
  
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(-360deg); }
  }
  
  /* Content area */
  .list-content {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px;
  }
  
  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--secondary-text, rgba(255, 255, 255, 0.6));
    padding: 16px;
    text-align: center;
  }
  
  :root[data-theme="light"] .empty-state .empty-title {
    color: var(--text-primary-light, #333);
  }
  :root[data-theme="light"] .empty-state .empty-text {
    color: var(--text-secondary-light, #666);
  }
  :root[data-theme="light"] .empty-state .empty-image svg {
    stroke: var(--text-secondary-light, #aaa);
  }
  
  .empty-image {
    margin-bottom: 16px;
    opacity: 0.2;
  }
  
  .empty-title {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
    color: var(--primary-text, #ffffff);
  }
  
  .empty-text {
    margin: 0;
    font-size: 14px;
  }
  
  /* Streamer list */
  .streamers-list {
    list-style: none;
    margin: 0 auto;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }
  
  :root[data-theme="light"] .streamer-item {
    background: var(--card-bg-light, #ffffff);
    color: var(--text-primary-light, #2d3748);
    box-shadow: var(--card-shadow-light, 0 1px 3px rgba(0,0,0,0.1));
    border: 1px solid var(--border-color-light, #e2e8f0);
  }
  
  .streamer-item {
    display: flex;
    align-items: center;
    padding: 5px 8px;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s ease-in-out;
    position: relative;
    overflow: hidden; /* Important for rounded corners with backdrop-filter */
    
    /* Night mode default background */
    background: #252730; 
    /* backdrop-filter: blur(12px); removed as background is opaque */
    /* -webkit-backdrop-filter: blur(12px); removed as background is opaque */
    color: var(--text-primary-dark, #e0e0e0);
    border: 1px solid var(--streamer-item-border-dark-glass, rgba(255, 255, 255, 0.1)); 
    box-shadow: var(--streamer-item-shadow-dark, 0 2px 5px rgba(0,0,0,0.2)); 
  }
  
  :root[data-theme="light"] .streamer-item:hover {
    background: var(--card-hover-bg-light, #f7fafc);
    box-shadow: var(--card-hover-shadow-light, 0 4px 6px rgba(0,0,0,0.1));
  }
  
  .streamer-item:hover {
    background: #2C2E33;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
  
  .streamer-item:active {
    transform: translateY(0);
    border: 1px solid rgba(16, 185, 129, 0.15);
    box-shadow: 0 2px 8px rgba(16, 185, 129, 0.1);
  }
  
  /* Live streamer styling - Restored card look */
  .streamer-item.is-live {
    background: linear-gradient(to right, 
      rgba(16, 185, 129, 0.15) 0%,
      rgba(16, 185, 129, 0.20) 50%,
      rgba(16, 185, 129, 0.15) 100%
    );
    border: 1px solid rgba(16, 185, 129, 0.4);
    box-shadow: 0 2px 8px rgba(16, 185, 129, 0.2);
  }
  
  :root[data-theme="light"] .streamer-item.is-live {
    background: linear-gradient(to right, 
      rgba(16, 185, 129, 0.08) 0%,
      rgba(16, 185, 129, 0.12) 50%,
      rgba(16, 185, 129, 0.08) 100%
    );
    border: 1px solid rgba(16, 185, 129, 0.3);
    box-shadow: 0 2px 8px rgba(16, 185, 129, 0.15);
  }
  
  .streamer-item.is-live:hover {
    background: linear-gradient(to right, 
      rgba(16, 185, 129, 0.2) 0%,
      rgba(16, 185, 129, 0.25) 50%,
      rgba(16, 185, 129, 0.2) 100%
    );
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.25);
  }
  
  :root[data-theme="light"] .streamer-item.is-live:hover {
    background: var(--card-hover-bg-light, #f7fafc);
    box-shadow: var(--card-hover-shadow-light, 0 4px 6px rgba(0,0,0,0.1));
  }
  
  .streamer-item.is-dragging {
    opacity: 0.7;
    transform: scale(1.02);
    z-index: 10;
    box-shadow: 0 10px 20px rgba(0, 0, 0, 0.3);
  }
  
  .streamer-item.just-added {
    animation: glow-new 2s ease;
  }
  
  @keyframes glow-new {
    0% { box-shadow: 0 0 0 rgba(16, 185, 129, 0); border-color: rgba(16, 185, 129, 0.1); }
    30% { box-shadow: 0 0 15px rgba(16, 185, 129, 0.3); border-color: rgba(16, 185, 129, 0.4); }
    100% { box-shadow: 0 0 0 rgba(16, 185, 129, 0); border-color: transparent; }
  }
  
  .item-content {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-grow: 1;
    min-width: 0;
  }
  
  .avatar-container {
    position: relative;
    width: 40px;
    height: 40px;
    flex-shrink: 0;
    border-radius: 10px;
    overflow: hidden;
    background: var(--card-bg, rgba(255, 255, 255, 0.03));
  }
  
  :root[data-theme="light"] .avatar-container {
    background: var(--card-bg-light, #e2e8f0); 
  }
  
  .avatar-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    font-size: 16px;
    font-weight: 600;
  }
  
  .avatar-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #a0aec0, #718096);
    color: #ffffff;
  }
  
  :root[data-theme="light"] .avatar-fallback {
    background: linear-gradient(135deg, #a0aec0, #718096); 
    color: #ffffff;
  }
  
  .status-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }
  
  .live-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: #757575;
    transition: background-color 0.3s ease;
  }
  
  .live-indicator.is-live {
    background-color: #4CAF50;
  }
  
  .live-indicator.is-replay {
    background-color: #ffc107;
  }
  
  .live-indicator.is-offline {
    background-color: #757575;
  }
  
  .streamer-details {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  .primary-row {
    display: flex;
    align-items: center;
  }
  
  .nickname {
    font-size: 14px;
    font-weight: 600;
    color: var(--primary-text, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
    letter-spacing: 0.2px;
    text-align: left;
  }
  
  :root[data-theme="light"] .nickname {
    color: var(--text-primary-light, #2d3748);
  }
  
  /* Live streamer text styling */
  .streamer-item.is-live .nickname {
    color: rgba(16, 185, 129, 0.9);
    text-shadow: 0 0 8px rgba(16, 185, 129, 0.3);
  }
  
  :root[data-theme="light"] .streamer-item.is-live .nickname {
    color: #0F7C6F; 
    text-shadow: none; 
  }
  
  .secondary-row {
    font-size: 12px;
    color: #9298a8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
    text-align: left;
  }
  
  /* Live streamer subtitle styling for dark mode */
  .streamer-item.is-live .secondary-row {
    color: #9298a8;
  }
  
  :root[data-theme="light"] .secondary-row {
    color: var(--text-secondary-light, #718096);
  }
  
  /* Live streamer subtitle styling for light mode */
  :root[data-theme="light"] .streamer-item.is-live .secondary-row {
    color: var(--text-secondary-light, #718096);
  }
  
  /* List transitions */
  .streamer-list-enter-active,
  .streamer-list-leave-active {
    transition: all 0.3s ease;
  }
  
  .streamer-list-enter-from {
    opacity: 0;
    transform: translateX(20px);
  }
  
  .streamer-list-leave-to {
    opacity: 0;
    transform: translateX(-20px);
  }
  
  .streamer-list-move {
    transition: transform 0.5s ease;
  }
  
  /* Scrollbar styles */
  .list-content::-webkit-scrollbar {
    width: 4px;
  }
  
  .list-content::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .list-content::-webkit-scrollbar-thumb {
    background: var(--border-color, rgba(255, 255, 255, 0.1));
    border-radius: 2px;
  }
  
  .list-content::-webkit-scrollbar-thumb:hover {
    background: var(--secondary-text, rgba(255, 255, 255, 0.2));
  }
  </style>