<template>
  <div class="common-live-list-container">
    <div v-if="isLoading && rooms.length === 0" class="loading-initial-state">
      <p>正在加载主播列表 {{ categoryName ? 'for ' + categoryName : '' }}...</p>
    </div>
    <div v-else-if="!isLoading && rooms.length === 0 && categoryHref" class="no-streamers-state">
      <p>分类 {{ categoryName || categoryHref }} 下暂无主播。</p>
    </div>
    <div v-else-if="!categoryHref && !isLoading" class="no-category-state">
       <p>请先选择一个分类。</p>
    </div>

    <div class="live-grid-scroll-area" ref="scrollComponentRef">
      <div class="live-grid-common">
        <div 
          v-for="(room, index) in rooms" 
          :key="room.room_id + '-' + index" 
          class="streamer-card-common"
          @click="goToPlayer(room.room_id)"
        >
          <div class="card-preview-common">
            <img :src="room.room_cover || 'https://via.placeholder.com/320x180.png?text=No+Image'" :alt="room.title" class="preview-image-common" />
            <span class="viewers-count-overlay-common">
              <svg class="viewers-icon-common" width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>
              {{ room.viewer_count_str || 'N/A' }} 
            </span>
          </div>
          <div class="card-info-footer-common">
            <img :src="room.avatar || 'https://via.placeholder.com/40.png?text=N/A'" :alt="room.nickname" class="streamer-avatar-common" />
            <div class="text-details-common">
              <h3 class="room-title-common" :title="room.title">{{ room.title }}</h3>
              <p class="nickname-common" :title="room.nickname">{{ room.nickname || '主播' }}</p>
            </div>
          </div>
        </div>
      </div>
      <div ref="sentinelRef" class="scroll-sentinel"></div>
      <div v-if="isLoadingMore" class="loading-more-indicator">
        <p>正在加载更多主播...</p>
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
import type { CategorySelectedEvent } from '../../platforms/common/categoryTypes'
import { useHuyaLiveRooms } from './composables/useHuyaLiveRooms'
import { useDouyinLiveRooms } from './composables/useDouyinLiveRooms'
import { useBilibiliLiveRooms } from './composables/useBilibiliLiveRooms'

const props = defineProps<{
  selectedCategory: CategorySelectedEvent | null;
  categoriesData?: any[]; // Pass platform categories to resolve id mapping generically
  playerRouteName?: string; // e.g., 'huyaPlayer' or 'douyinPlayer'
  platformName?: 'huya' | 'douyin' | 'douyu' | 'bilibili' | string; // choose composable
  defaultPageSize?: number; // platform-specific default page size, Huya=120
}>();

const router = useRouter();
const scrollComponentRef = ref<HTMLElement | null>(null);
const sentinelRef = ref<HTMLElement | null>(null);
const categoryHref = computed(() => props.selectedCategory?.cate2Href || null);
const categoryName = computed(() => props.selectedCategory?.cate2Name || null);
const platformName = computed(() => props.platformName ?? 'huya');

// Resolve platform-specific gid/id from categoriesData based on cate2Href (Huya)
const resolvedSubcategoryId = computed(() => {
  const href = props.selectedCategory?.cate2Href;
  const data = props.categoriesData;
  if (!href || !Array.isArray(data)) return null;
  for (const c1 of data) {
    if (!Array.isArray(c1.subcategories)) continue;
    const c2 = c1.subcategories.find((s: any) => s.href === href);
    if (c2 && (c2.id || c2.gid)) return String(c2.id ?? c2.gid);
  }
  return null;
});

// Douyin: parse cate2Href to partition and partitionType
const douyinPartition = computed(() => { 
  const href = props.selectedCategory?.cate2Href;
  if (!href) return null;
  const parts = href.split('_');
  if (parts.length >= 1) {
    return parts[parts.length - 1];
  }
  return null;
});
const douyinPartitionType = computed(() => { 
  const href = props.selectedCategory?.cate2Href;
  if (!href) return null;
  const parts = href.split('_');
  if (parts.length >= 2) {
    return parts[parts.length - 2];
  }
  return null;
});

const resolvedParentCategoryId = computed(() => {
  const href = props.selectedCategory?.cate2Href;
  const data = props.categoriesData;
  if (!href || !Array.isArray(data)) return null;
  for (const c1 of data) {
    if (!Array.isArray(c1.subcategories)) continue;
    const c2 = c1.subcategories.find((s: any) => s.href === href);
    if (c2 && (c2.parent_id || c2.parentId || c1.id)) return String(c2.parent_id ?? c2.parentId ?? c1.id);
  }
  return null;
});

const huyaComposable = useHuyaLiveRooms(resolvedSubcategoryId, { defaultPageSize: props.defaultPageSize ?? 120 });
const douyinComposable = useDouyinLiveRooms(douyinPartition, douyinPartitionType);
const bilibiliComposable = useBilibiliLiveRooms(resolvedSubcategoryId, resolvedParentCategoryId);
const selectedComposable = computed(() => {
  if (platformName.value === 'douyin') return douyinComposable;
  if (platformName.value === 'bilibili') return bilibiliComposable;
  return huyaComposable;
});

const rooms = computed(() => selectedComposable.value.rooms.value);
const isLoading = computed(() => selectedComposable.value.isLoading.value);
const isLoadingMore = computed(() => selectedComposable.value.isLoadingMore.value);
const error = computed(() => selectedComposable.value.error.value);
const hasMore = computed(() => selectedComposable.value.hasMore.value);
const loadInitialRooms = () => selectedComposable.value.loadInitialRooms();
const loadMoreRooms = () => selectedComposable.value.loadMoreRooms();

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
    if (newCategory.cate2Href) { 
        loadInitialRooms();
    }
  } else {
    // Reset states when no category
    if (platformName.value === 'douyin') {
      douyinComposable.rooms.value = [];
      douyinComposable.hasMore.value = false;
      douyinComposable.error.value = null;
    } else if (platformName.value === 'bilibili') {
      bilibiliComposable.rooms.value = [];
      bilibiliComposable.hasMore.value = false;
      bilibiliComposable.error.value = null;
    } else {
      huyaComposable.rooms.value = [];
      huyaComposable.hasMore.value = false;
      huyaComposable.error.value = null;
    }
  }
  nextTick(() => {
    if (scrollComponentRef.value && sentinelRef.value) setupIntersectionObserver();
  });
}, { immediate: true, deep: true });

const goToPlayer = (roomId: string) => {
  if (!roomId) return;
  if (props.playerRouteName) {
    router.push({ name: props.playerRouteName, params: { roomId } });
  }
};

</script>

<style scoped>
/* Styles mirror DouyinStreamerList */
.common-live-list-container {
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

.loading-initial-state,
.no-streamers-state,
.no-category-state,
.loading-more-indicator,
.error-state-message {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
  color: var(--secondary-text);
  font-size: 15px;
  text-align: center;
}
.loading-initial-state, .no-streamers-state, .no-category-state, .error-state-message {
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

.live-grid-common {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 24px;
  width: 100%;
}

.streamer-card-common {
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

:root[data-theme="dark"] .streamer-card-common:hover {
  transform: translateY(-4px);
  box-shadow: 0 6px 12px rgba(0,0,0, 0.3);
  border-color: var(--border-color-light);
}
:root[data-theme="light"] .streamer-card-common:hover {
  transform: translateY(-4px);
  box-shadow: 0 10px 25px rgba(0,0,0, 0.15);
  border-color: transparent;
  background-color: var(--streamer-card-hover-bg-light, #f8f9fa);
}

.card-preview-common {
  width: 100%;
  aspect-ratio: 16 / 10;
  background-color: var(--secondary-bg);
  position: relative;
  overflow: hidden;
}

.preview-image-common {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.viewers-count-overlay-common {
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

.viewers-icon-common {
  margin-right: 4px;
}

.card-info-footer-common {
  display: flex;
  align-items: center;
  padding: 10px;
}

.streamer-avatar-common {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  margin-right: 10px;
  flex-shrink: 0;
  object-fit: cover;
  background-color: #444;
}

.text-details-common {
  overflow: hidden;
  flex-grow: 1;
}

.room-title-common {
  font-size: 0.9rem;
  margin: 0 0 2px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
:root[data-theme="dark"] .room-title-common {
  color: var(--streamer-title-text-dark, #e0e0e0);
}
:root[data-theme="light"] .room-title-common {
  color: var(--streamer-title-text-light, #000000);
}

.nickname-common {
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