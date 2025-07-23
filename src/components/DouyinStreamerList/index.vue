<template>
  <div class="douyin-live-list-container">
    <div v-if="isLoading && rooms.length === 0" class="loading-initial-state">
      <p>正在加载抖音主播列表 {{ categoryName ? 'for ' + categoryName : '' }}...</p>
    </div>
    <div v-else-if="!isLoading && rooms.length === 0 && categoryHref" class="no-streamers-state">
      <p>分类 {{ categoryName || categoryHref }} 下暂无主播。</p>
    </div>
    <div v-else-if="!categoryHref && !isLoading" class="no-category-state">
       <p>请先选择一个抖音分类。</p>
    </div>

    <div class="live-grid-scroll-area" ref="scrollComponentRef">
      <div class="live-grid-douyin">
        <div 
          v-for="(room, index) in rooms" 
          :key="room.web_rid + '-' + index" 
          class="streamer-card-douyin"
          @click="goToPlayer(room.web_rid)"
        >
          <div class="card-preview-douyin">
            <img :src="room.room_cover || 'https://via.placeholder.com/320x180.png?text=No+Image'" :alt="room.title" class="preview-image-douyin" />
            <span class="viewers-count-overlay-douyin">
              <svg class="viewers-icon-douyin" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>
              {{ room.viewer_count_str || 'N/A' }} 
            </span>
          </div>
          <div class="card-info-footer-douyin">
            <img :src="room.avatar || 'https://via.placeholder.com/40.png?text=N/A'" :alt="room.nickname" class="streamer-avatar-douyin" />
            <div class="text-details-douyin">
              <h3 class="room-title-douyin" :title="room.title">{{ room.title }}</h3>
              <p class="nickname-douyin" :title="room.nickname">{{ room.nickname || '抖音主播' }}</p>
            </div>
          </div>
        </div>
      </div>
      <div ref="sentinelRef" class="scroll-sentinel"></div>
      <div v-if="isLoadingMore" class="loading-more-indicator">
        <p>正在加载更多抖音主播...</p>
      </div>
       <div v-if="error" class="error-state-message">
        <p>加载失败: {{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useDouyinLiveRooms } from './composables/useDouyinLiveRooms';
import type { DouyinCategorySelectedEvent } from '../DouyinCategory/types'; // Corrected path

const props = defineProps<{
  selectedCategory: DouyinCategorySelectedEvent | null;
}>();

const router = useRouter();
const scrollComponentRef = ref<HTMLElement | null>(null);
const sentinelRef = ref<HTMLElement | null>(null);
const categoryHref = computed(() => props.selectedCategory?.cate2Href || null);
const categoryName = computed(() => props.selectedCategory?.cate2Name || null);


const douyinPartition = computed(() => { 
  if (!props.selectedCategory?.cate2Href) {
    return null;
  }
  const parts = props.selectedCategory.cate2Href.split('_');
  
  if (parts.length >= 1) { // Check if there's at least one part (for the last part)
      const partitionId = parts[parts.length - 1];
      return partitionId;
  }
  return null; 
});

const douyinPartitionType = computed(() => { 
  if (!props.selectedCategory?.cate2Href) {
    return null;
  }
  const parts = props.selectedCategory.cate2Href.split('_');

   if (parts.length >= 2) { // Need at least two parts to get second-to-last
      const typeId = parts[parts.length - 2];
      return typeId;
  }
  return null; // Explicitly return null if type cannot be determined
});

const { 
  rooms,
  isLoading,
  isLoadingMore,
  error,
  hasMore,
  loadInitialRooms,
  loadMoreRooms
} = useDouyinLiveRooms(douyinPartition, douyinPartitionType);

let observer: IntersectionObserver | null = null;

const setupIntersectionObserver = () => {
  if (observer) observer.disconnect();
  const options = { root: scrollComponentRef.value, rootMargin: '0px', threshold: 0.1 };

  observer = new IntersectionObserver((entries) => {
    const entry = entries[0];
    if (entry.isIntersecting && hasMore.value && !isLoading.value && !isLoadingMore.value) {
      loadMoreRooms();
    }
  }, options);

  if (sentinelRef.value) observer.observe(sentinelRef.value);
};

onMounted(() => {
  nextTick(() => setupIntersectionObserver());
});

onBeforeUnmount(() => {
  if (observer) observer.disconnect();
});

watch(() => props.selectedCategory, (newCategory, _oldCategory) => {
  if (newCategory) {
    // console.log('[DouyinStreamerList] Selected category changed:', newCategory, _oldCategory);
    if (newCategory.cate2Href) { // Use cate2Href to check if a category is selected
        loadInitialRooms();
    }
  } else {
    rooms.value = [];
    hasMore.value = false;
    error.value = null;
  }
  nextTick(() => {
    if (scrollComponentRef.value && sentinelRef.value) setupIntersectionObserver();
  });
}, { immediate: true, deep: true });

const goToPlayer = (roomId: string) => {
  if (!roomId) return;
  router.push({ name: 'douyinPlayer', params: { roomId } }); 
};

</script>

<style scoped>
.douyin-live-list-container { /* Original: .live-list-container-infinite */
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  box-sizing: border-box;
  background-color: var(--primary-bg); /* Douyu style */
  color: var(--primary-text); /* Douyu style */
  overflow: hidden; 
  transition: background-color 0.3s ease, color 0.3s ease; /* Douyu style */
}

/* Loading/empty states - class names are similar, apply Douyu's text color */
.loading-initial-state,
.no-streamers-state,
.no-category-state,
.loading-more-indicator,
.error-state-message { /* .error-state-message is specific to Douyin, but style similarly */
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  color: var(--secondary-text); /* Douyu style for text color */
  font-size: 15px;
  text-align: center;
}
.loading-initial-state, .no-streamers-state, .no-category-state, .error-state-message { /* Added .error-state-message here */
    flex-grow: 1; 
}
.loading-more-indicator { /* Retain Douyin's, or make it match Douyu's if different */
    min-height: 60px; 
}

/* Scroll area - class name is the same */
.live-grid-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px 24px; /* Douyu style */
  position: relative; 
  scrollbar-width: thin; /* Douyu style */
  scrollbar-color: var(--scrollbar-thumb-bg, #444) var(--scrollbar-track-bg, #18181b); /* Douyu style */
}
:root[data-theme="light"] .live-grid-scroll-area {
  scrollbar-color: var(--scrollbar-thumb-bg-light, #adb5bd) var(--scrollbar-track-bg-light, #e9ecef); /* Douyu style */
}

.live-grid-scroll-area::-webkit-scrollbar {
  width: 8px; /* Douyu style */
}
.live-grid-scroll-area::-webkit-scrollbar-track {
  background: var(--scrollbar-track-bg, #18181b); /* Douyu style */
}
:root[data-theme="light"] .live-grid-scroll-area::-webkit-scrollbar-track {
  background: var(--scrollbar-track-bg-light, #e9ecef); /* Douyu style */
}

.live-grid-scroll-area::-webkit-scrollbar-thumb {
  background-color: var(--scrollbar-thumb-bg, #444); /* Douyu style */
  border-radius: 4px; /* Douyu style */
  border: 2px solid var(--scrollbar-track-bg, #18181b); /* Douyu style */
}
:root[data-theme="light"] .live-grid-scroll-area::-webkit-scrollbar-thumb {
  background-color: var(--scrollbar-thumb-bg-light, #adb5bd); /* Douyu style */
  border: 2px solid var(--scrollbar-track-bg-light, #e9ecef); /* Douyu style */
}

/* Grid itself */
.live-grid-douyin { /* Original: .live-grid-infinite */
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); /* Douyu style */
  gap: 24px; /* Douyu style */
  width: 100%;
}

/* Streamer card */
.streamer-card-douyin { /* Original: .streamer-card-revised */
  background-color: var(--card-bg); /* Douyu style */
  color: var(--primary-text); /* Douyu style */
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: transform 0.2s ease-out, box-shadow 0.2s ease-out, border-color 0.2s ease-out, background-color 0.3s ease; /* Douyu style */
  cursor: pointer;
  border: 1px solid transparent; 
  box-shadow: var(--card-shadow); /* Douyu style */
}

:root[data-theme="dark"] .streamer-card-douyin:hover { /* Original: :root[data-theme="dark"] .streamer-card-revised:hover */
  transform: translateY(-4px);
  box-shadow: 0 6px 12px rgba(0,0,0, 0.3); /* Douyu style */
  border-color: var(--border-color-light); /* Douyu style, careful with var name for dark theme */
}
:root[data-theme="light"] .streamer-card-douyin:hover { /* Original: :root[data-theme="light"] .streamer-card-revised:hover */
  transform: translateY(-4px);
  box-shadow: 0 10px 25px rgba(0,0,0, 0.15); /* Douyu style */
  border-color: transparent; /* Douyu style */
  background-color: var(--streamer-card-hover-bg-light, #f8f9fa); /* Douyu style */
}

/* Card preview */
.card-preview-douyin { /* Original: .card-preview-revised */
  width: 100%;
  aspect-ratio: 16 / 10; /* Douyu style */
  background-color: var(--secondary-bg); /* Douyu style */
  position: relative;
  overflow: hidden; /* Douyu style */
}

.preview-image-douyin { /* Original: .preview-image-revised */
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block; /* Douyu style */
}

/* Viewers count overlay */
.viewers-count-overlay-douyin { /* Original: .viewers-count-overlay */
  position: absolute;
  top: 8px;
  right: 8px;
  background-color: rgba(0, 0, 0, 0.6); /* Douyu style */
  color: white; /* Douyu style */
  padding: 3px 8px; /* Douyu style */
  border-radius: 4px; /* Douyu style */
  font-size: 0.75rem; /* Douyu style */
  display: flex;
  align-items: center;
  line-height: 1; /* Douyu style */
}

.viewers-icon-douyin { /* Original: .viewers-icon-revised */
  margin-right: 4px; /* Douyu style */
}

/* Card info footer */
.card-info-footer-douyin { /* Original: .card-info-footer-revised */
  display: flex;
  align-items: center;
  padding: 10px; /* Douyu style */
}

.streamer-avatar-douyin { /* Original: .streamer-avatar-revised */
  width: 36px;
  height: 36px;
  border-radius: 50%;
  margin-right: 10px;
  flex-shrink: 0;
  object-fit: cover;
  background-color: #444; /* Douyu style fallback */
}

.text-details-douyin { /* Original: .text-details-revised */
  overflow: hidden;
  flex-grow: 1;
}

.room-title-douyin { /* Original: .room-title-revised */
  font-size: 0.9rem; /* Douyu style */
  margin: 0 0 2px 0; /* Douyu style */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
/* Apply Douyu's themed text colors */
:root[data-theme="dark"] .room-title-douyin {
  color: var(--streamer-title-text-dark, #e0e0e0);
}
:root[data-theme="light"] .room-title-douyin {
  color: var(--streamer-title-text-light, #000000);
}

.nickname-douyin { /* Original: .nickname-revised */
  font-size: 0.8rem; /* Douyu style */
  color: #909090; /* Douyu style, consider var(--text-secondary-light/dark) */
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Sentinel - class name is the same */
.scroll-sentinel {
  height: 10px; /* Douyu style */
  width: 100%; /* Douyu style */
}
</style> 