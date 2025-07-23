<template>
  <div class="live-list-container-infinite">
    <div v-if="isLoading && streamers.length === 0" class="loading-initial-state">
      <p>正在加载主播列表 {{ props.categoryName ? 'for ' + props.categoryName : '' }}...</p>
    </div>
    <div v-else-if="!isLoading && streamers.length === 0 && props.categoryId" class="no-streamers-state">
      <p>分类 {{ props.categoryName || props.categoryId }} 下暂无主播。</p>
    </div>
    <div v-else-if="!props.categoryId && !isLoading" class="no-category-state">
       <p>请先选择一个分类。</p>
    </div>

    <div class="live-grid-scroll-area" ref="scrollComponentRef">
      <div class="live-grid-infinite">
        <div 
          v-for="(streamer, index) in streamers" 
          :key="streamer.rid + '-' + index" 
          class="streamer-card-revised"
          @click="goToPlayer(streamer.rid)"
        >
          <div class="card-preview-revised">
            <img :src="streamer.roomSrc || 'https://via.placeholder.com/320x180.png?text=No+Image'" :alt="streamer.roomName" class="preview-image-revised" />
            <span class="viewers-count-overlay">
              <svg class="viewers-icon-revised" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>
              {{ streamer.hn }} 
            </span>
          </div>
          <div class="card-info-footer-revised">
            <img :src="streamer.avatar || 'https://via.placeholder.com/40.png?text=N/A'" :alt="streamer.nickname" class="streamer-avatar-revised" />
            <div class="text-details-revised">
              <h3 class="room-title-revised" :title="streamer.roomName">{{ streamer.roomName }}</h3>
              <p class="nickname-revised" :title="streamer.nickname">{{ streamer.nickname }}</p>
            </div>
          </div>
        </div>
      </div>
      <div ref="sentinelRef" class="scroll-sentinel"></div>
      <div v-if="isLoading && streamers.length > 0" class="loading-more-indicator">
        <p>正在加载更多主播...</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useLiveData } from './composables/useLiveData'

const props = defineProps<{
  categoryType: 'cate2' | 'cate3';
  categoryId: string;
  categoryName?: string;
}>()

const router = useRouter();
const scrollComponentRef = ref<HTMLElement | null>(null);
const sentinelRef = ref<HTMLElement | null>(null);

const { 
  streamers, 
  hasMore, 
  isLoading, 
  loadNextPage, 
  resetAndFetch 
} = useLiveData(); 

let observer: IntersectionObserver | null = null;

const setupIntersectionObserver = () => {
  if (observer) observer.disconnect();

  const options = {
    root: scrollComponentRef.value,
    rootMargin: '0px',
    threshold: 0.1 
  };

  observer = new IntersectionObserver((entries) => {
    const entry = entries[0];
    if (entry.isIntersecting && hasMore.value && !isLoading.value) {
      if (props.categoryType && props.categoryId) {
        loadNextPage(props.categoryType, props.categoryId);
      }
    }
  }, options);

  if (sentinelRef.value) {
    observer.observe(sentinelRef.value);
  } else {
    console.warn('[LiveList InfiniteScroll] Sentinel ref not found during observer setup.');
  }
};


onMounted(() => {
  nextTick(() => {
      setupIntersectionObserver();
  });
});

onBeforeUnmount(() => {
  if (observer) {
    observer.disconnect();
  }
});

watch(() => ({ type: props.categoryType, id: props.categoryId }), (newCategory, _oldCategory) => {
  if (newCategory && newCategory.id) {
    resetAndFetch(newCategory.type, newCategory.id);
  } else {
    streamers.value = []; 
    hasMore.value = false; 
  }
  nextTick(() => {
      if(sentinelRef.value && observer) {
      } else if (scrollComponentRef.value && sentinelRef.value) {
          setupIntersectionObserver();
      }
  });
}, { immediate: true, deep: true });

const goToPlayer = (roomId: string) => {
  if (!roomId) return;
  router.push({ name: 'douyuPlayer', params: { roomId } });
}

</script>

<style scoped>
.live-list-container-infinite {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  box-sizing: border-box;
  background-color: var(--primary-bg);
  color: var(--primary-text);
  overflow: hidden; 
  transition: background-color 0.3s ease, color 0.3s ease;
}

.live-list-header {
  padding: 8px 16px;
  color: #e0e0e0;
  font-size: 1.1em;
  background-color: #2a2a2e;
  border-bottom: 1px solid #333;
  flex-shrink: 0;
}
.live-list-header h2 {
  margin: 0;
  font-size: 1em;
  font-weight: 600;
}

.loading-initial-state,
.no-streamers-state,
.no-category-state,
.loading-more-indicator {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  color: var(--secondary-text);
  font-size: 15px;
  text-align: center;
}
.loading-initial-state, .no-streamers-state, .no-category-state {
    flex-grow: 1; 
}
.loading-more-indicator {
    min-height: 60px; 
}

.live-grid-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 16px 24px;
  position: relative; 
  scrollbar-width: thin; 
  scrollbar-color: var(--scrollbar-thumb-bg, #444) var(--scrollbar-track-bg, #18181b);
}
:root[data-theme="light"] .live-grid-scroll-area {
  scrollbar-color: var(--scrollbar-thumb-bg-light, #adb5bd) var(--scrollbar-track-bg-light, #e9ecef);
}

.live-grid-scroll-area::-webkit-scrollbar {
  width: 8px;
}
.live-grid-scroll-area::-webkit-scrollbar-track {
  background: var(--scrollbar-track-bg, #18181b);
}
:root[data-theme="light"] .live-grid-scroll-area::-webkit-scrollbar-track {
  background: var(--scrollbar-track-bg-light, #e9ecef);
}

.live-grid-scroll-area::-webkit-scrollbar-thumb {
  background-color: var(--scrollbar-thumb-bg, #444);
  border-radius: 4px;
  border: 2px solid var(--scrollbar-track-bg, #18181b);
}
:root[data-theme="light"] .live-grid-scroll-area::-webkit-scrollbar-thumb {
  background-color: var(--scrollbar-thumb-bg-light, #adb5bd);
  border: 2px solid var(--scrollbar-track-bg-light, #e9ecef);
}

.live-grid-infinite {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 24px;
  width: 100%;
}

.streamer-card-revised {
  background-color: var(--card-bg);
  color: var(--primary-text);
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: transform 0.2s ease-out, box-shadow 0.2s ease-out, border-color 0.2s ease-out, background-color 0.3s ease;
  cursor: pointer;
  border: 1px solid transparent; 
  box-shadow: var(--card-shadow);
}

:root[data-theme="dark"] .streamer-card-revised:hover {
  transform: translateY(-4px);
  box-shadow: 0 6px 12px rgba(0,0,0, 0.3);
  border-color: var(--border-color-light);
}
:root[data-theme="light"] .streamer-card-revised:hover {
  transform: translateY(-4px);
  box-shadow: 0 10px 25px rgba(0,0,0, 0.15);
  border-color: transparent;
  background-color: var(--streamer-card-hover-bg-light, #f8f9fa);
}

.card-preview-revised {
  width: 100%;
  aspect-ratio: 16 / 10;
  background-color: var(--secondary-bg);
  position: relative;
  overflow: hidden;
}

.preview-image-revised {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.viewers-count-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
  background-color: rgba(0, 0, 0, 0.6);
  color: white;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
  display: flex;
  align-items: center;
  line-height: 1;
}

.viewers-icon-revised {
  margin-right: 4px;
}

.card-info-footer-revised {
  display: flex;
  align-items: center;
  padding: 10px;
}

.streamer-avatar-revised {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  margin-right: 10px;
  flex-shrink: 0;
  object-fit: cover;
  background-color: #444;
}

.text-details-revised {
  overflow: hidden;
  flex-grow: 1;
}

.room-title-revised {
  font-size: 0.9rem;
  margin: 0 0 2px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

:root[data-theme="dark"] .room-title-revised {
  color: var(--streamer-title-text-dark, #e0e0e0);
}
:root[data-theme="light"] .room-title-revised {
  color: var(--streamer-title-text-light, #000000);
}

.nickname-revised {
  font-size: 0.8rem;
  color: #909090;
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scroll-sentinel {
  height: 10px;
  width: 100%;
}
</style> 