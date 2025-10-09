<template>
    <div class="follow-list">
      <div class="list-header">
        <h3 class="header-title">关注列表</h3>
        <div class="header-actions">
          <button 
            v-if="!isRefreshing"
            @click="refreshList" 
            class="action-btn refresh-btn"
            title="刷新列表"
          >
            <span class="icon">
              <!-- 刷新按钮保留默认图标/完成勾号，刷新完成后展示 1 秒 -->
              <svg v-if="!showCheckIcon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-rotate-cw-icon lucide-rotate-cw"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 6L9 17l-5-5" />
              </svg>
            </span>
          </button>
          <!-- 刷新转圈与进度文本合并为一个元素，转圈在左，进度在右；共享统一的圆角矩形背景 -->
          <span v-if="isRefreshing" class="progress-with-spinner" aria-live="polite">
            <span class="refresh-spinner-inline" aria-hidden="true"></span>
            <span class="progress-label">{{ progressCurrent }}/{{ progressTotal }}</span>
          </span>
          <!-- 展开悬浮关注列表按钮 -->
          <button 
            ref="expandBtnRef"
            @click="openOverlay" 
            class="action-btn expand-btn"
            title="展开关注列表"
          >
            <span class="icon">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M4 6h16v2H4V6zm0 5h16v2H4v-2zm0 5h16v2H4v-2z"/>
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
            <StreamerItem 
              :streamer="streamer"
              :getAvatarSrc="getAvatarSrc"
              :handleImgError="handleImgError"
              :getLiveIndicatorClass="getLiveIndicatorClass"
              :proxyBase="proxyBase"
              @clickItem="(s) => emit('selectAnchor', s)"
            />
          </li>
        </TransitionGroup>
      </div>

      <!-- 悬浮关注列表：使用组件 FollowOverlay -->
      <FollowOverlay 
        :show="showOverlay"
        :items="filteredStreamers"
        :getAvatarSrc="getAvatarSrc"
        :handleImgError="handleImgError"
        :getLiveIndicatorClass="getLiveIndicatorClass"
        :proxyBase="proxyBase"
        :alignTop="overlayAlignTop"
        :alignLeft="overlayAlignLeft"
        :isRefreshing="isRefreshing"
        @select="selectFromOverlay"
        @close="closeOverlay"
        @refresh="refreshList"
      >
        <template #filters>
          <FilterChips 
            :visiblePlatforms="visiblePlatforms"
            :activeFilter="activeFilter"
            @update:activeFilter="setFilter"
          />
        </template>
      </FollowOverlay>
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
  import { invoke } from '@tauri-apps/api/core';
  import StreamerItem from './StreamerItem.vue';
  import FollowOverlay from './FollowOverlay.vue';
  import FilterChips from './FilterChips.vue';
  import { useImageProxy } from './useProxy';

  const expandBtnRef = ref<HTMLButtonElement | null>(null)
  const overlayAlignTop = ref<number>(64)
  const overlayAlignLeft = ref<number>(240)
  
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
  // 新增：刷新完成后显示打勾图标 1 秒
  const showCheckIcon = ref(false);
  watch(isRefreshing, (newVal, oldVal) => {
    if (oldVal && !newVal) {
      showCheckIcon.value = true;
      setTimeout(() => { showCheckIcon.value = false; }, 1000);
    }
  });

  const listRef = ref<HTMLElement | null>(null);
  const isDragging = ref(false);
  const draggedIndex = ref(-1);
  const startY = ref(0);
  const currentY = ref(0);
  const justAddedIds = ref<string[]>([]);
  const animationTimeout = ref<number | null>(null);

  // 并发与延迟设置：降低启动时对后端的压力，优先让分类/主播列表完成首屏加载
  const FOLLOW_REFRESH_CONCURRENCY = 4; // 可根据机器性能与后端并发能力调整
  const REFRESH_INITIAL_DELAY_MS = 1500; // 首次进入页面延迟触发关注列表刷新
  function requestIdle(fn: () => void, timeout = REFRESH_INITIAL_DELAY_MS) {
    // 在浏览器空闲或设定超时后再触发，避免与首页的分类/主播列表争抢网络与后端资源
    if (typeof (window as any).requestIdleCallback === 'function') {
      (window as any).requestIdleCallback(fn, { timeout });
    } else {
      setTimeout(fn, timeout);
    }
  }
  
  // 头像代理：使用可复用的组合式函数
  const { proxyBase, ensureProxyStarted, getAvatarSrc: proxyGetAvatarSrc } = useImageProxy();

  function getAvatarSrc(s: FollowedStreamer): string {
    return proxyGetAvatarSrc(s.platform as unknown as string, s.avatarUrl);
  }

  function handleImgError(ev: Event, s: FollowedStreamer) {
    const target = ev.target as HTMLImageElement | null;
    if (!target) return;
    const base = proxyBase.value;
    const isProxied = !!base && target.src.startsWith(base);
    // 如果是代理后的 B 站图片加载失败，不再回退到原始地址（避免 403 报错）
    if (s.platform === Platform.BILIBILI) {
      // 可选择在此设置占位图，当前保持不变以显示 fallback 文本
      return;
    }
    if (isProxied) {
      target.src = s.avatarUrl || '';
    }
  }
  const MIN_ANIMATION_DURATION = 1500;
  

  const streamers = computed(() => props.followedAnchors);
  
  // Overlay: floating full follow list with platform filters
  const showOverlay = ref(false);
  type FilterType = 'ALL' | Platform;
  const activeFilter = ref<FilterType>('ALL');
  const openOverlay = () => { 
    const headerRect = document.querySelector('.app-header')?.getBoundingClientRect() as DOMRect | undefined
    overlayAlignTop.value = headerRect ? Math.round(headerRect.bottom + 8) : 72
    const rect = expandBtnRef.value?.getBoundingClientRect()
    overlayAlignLeft.value = rect ? Math.round(rect.right + 12) : 240
    showOverlay.value = true; 
  };
  const closeOverlay = () => { showOverlay.value = false; };
  const setFilter = (f: FilterType) => { activeFilter.value = f; };
  const platformsOrder: Platform[] = [Platform.DOUYU, Platform.DOUYIN, Platform.HUYA, Platform.BILIBILI];
  const visiblePlatforms = computed(() => {
    const present = new Set<Platform>();
    for (const s of streamers.value) {
      if (s.platform !== undefined) present.add(s.platform);
    }
    return platformsOrder.filter(p => present.has(p));
  });
  const filteredStreamers = computed(() => {
    if (activeFilter.value === 'ALL') return streamers.value;
    return streamers.value.filter(s => s.platform === activeFilter.value);
  });

  const selectFromOverlay = (s: FollowedStreamer) => {
    emit('selectAnchor', s);
    closeOverlay();
  };
  
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
  
  // 新增：刷新进度与完成提示
  const progressCurrent = ref(0);
  const progressTotal = ref(0);

  // 简易并发控制器：限制同时运行的刷新任务数量
  async function runWithConcurrency<T>(items: T[], worker: (item: T, index: number) => Promise<void>, limit: number) {
    let cursor = 0;
    const runners: Promise<void>[] = [];
    const runner = async () => {
      while (cursor < items.length) {
        const i = cursor++;
        await worker(items[i], i);
        // 让出事件循环，避免持续占用主线程
        await new Promise(res => setTimeout(res, 0));
      }
    };
    const n = Math.min(limit, items.length);
    for (let k = 0; k < n; k++) runners.push(runner());
    await Promise.all(runners);
  }

  const refreshList = async () => {
    if (isRefreshing.value) return;
    const startTime = Date.now();
    isRefreshing.value = true;

    // 初始化进度
    progressCurrent.value = 0;
    progressTotal.value = props.followedAnchors.length;
    try {
      // 仅在包含 B 站主播时启动静态代理（用于头像等图片代理）
      const hasBili = props.followedAnchors.some(s => s.platform === Platform.BILIBILI);
      if (hasBili) {
        await ensureProxyStarted();
      }

      const updates: FollowedStreamer[] = [];
      const items = [...props.followedAnchors];

      await runWithConcurrency(items, async (streamer) => {
        let updatedStreamerData: Partial<FollowedStreamer> = {};
        try {
          if (streamer.platform === Platform.DOUYU) {
            updatedStreamerData = await refreshDouyuFollowedStreamer(streamer);
          } else if (streamer.platform === Platform.DOUYIN) {
            updatedStreamerData = await refreshDouyinFollowedStreamer(streamer);
          } else if (streamer.platform === Platform.HUYA) {
            try {
              const res: any = await invoke('get_huya_unified_cmd', { roomId: streamer.id, quality: '原画' });
              const live: boolean = !!(res && res.is_live);
              const liveStatus: LiveStatus = live ? 'LIVE' : 'OFFLINE';
              updatedStreamerData = {
                liveStatus,
                isLive: live,
                nickname: (res && res.nick) ? res.nick : streamer.nickname,
                roomTitle: (res && res.title) ? res.title : streamer.roomTitle,
                avatarUrl: (res && res.avatar) ? res.avatar : streamer.avatarUrl,
              };
            } catch (err: any) {
              const msg = typeof err === 'string' ? err : (err?.message || '');
              if (msg.includes('主播未开播或获取虎牙房间详情失败')) {
                updatedStreamerData = {
                  liveStatus: 'OFFLINE',
                  isLive: false,
                  nickname: streamer.nickname,
                  roomTitle: streamer.roomTitle,
                  avatarUrl: streamer.avatarUrl,
                };
              } else {
                throw err;
              }
            }
          } else if (streamer.platform === Platform.BILIBILI) {
            const payload = { args: { room_id_str: streamer.id } };
            const savedCookie = (typeof localStorage !== 'undefined') ? (localStorage.getItem('bilibili_cookie') || null) : null;
            const res: any = await invoke('fetch_bilibili_streamer_info', { payload, cookie: savedCookie });
            const liveStatus: LiveStatus = (res && res.status === 1) ? 'LIVE' : 'OFFLINE';
            updatedStreamerData = {
              liveStatus,
              isLive: liveStatus === 'LIVE',
              nickname: (res && res.anchor_name) ? res.anchor_name : streamer.nickname,
              roomTitle: (res && res.title) ? res.title : streamer.roomTitle,
              avatarUrl: (res && res.avatar) ? res.avatar : streamer.avatarUrl,
            };
          } else {
            console.warn(`Unsupported platform for refresh: ${streamer.platform}`);
            updates.push(streamer);
            progressCurrent.value++;
            return;
          }

          updates.push({
            ...streamer,
            ...updatedStreamerData,
          } as FollowedStreamer);
        } catch (e) {
          console.error(`[FollowsList] Error during refresh for ${streamer.platform}/${streamer.id}, returning original:`, e);
          updates.push(streamer);
        } finally {
          // 更新进度
          progressCurrent.value++;
        }
      }, FOLLOW_REFRESH_CONCURRENCY);

      const validUpdates = updates.filter((update: FollowedStreamer | undefined): update is FollowedStreamer => !!update && typeof update.id !== 'undefined');
      if (validUpdates.length > 0) {
        // Preserve original user-defined order: map updates back onto the original list order (use platform:id to avoid collisions)
        const toKey = (s: FollowedStreamer) => `${s.platform}:${s.id}`;
        const updateMap = new Map<string, FollowedStreamer>(validUpdates.map(u => [toKey(u), u]));
        const reorderedPreservingOrder = props.followedAnchors.map(orig => updateMap.get(toKey(orig)) ?? orig);
        const hasChanged = JSON.stringify(reorderedPreservingOrder) !== JSON.stringify(props.followedAnchors);
        if (hasChanged) {
          emit('reorderList', reorderedPreservingOrder);
        }
      }
    } finally {
      const elapsedTime = Date.now() - startTime;
      const finish = () => {
        isRefreshing.value = false;
        showCheckIcon.value = true;
        setTimeout(() => { showCheckIcon.value = false; }, 1000);
      };
      if (elapsedTime < MIN_ANIMATION_DURATION) {
        clearAnimationTimeout();
        animationTimeout.value = window.setTimeout(() => {
          finish();
          animationTimeout.value = null;
        }, MIN_ANIMATION_DURATION - elapsedTime);
      } else {
        finish();
      }
    }
  };
  
  onMounted(async () => {
    // 在初次渲染前，若包含 B 站主播则先启动静态代理，避免头像首次以原始地址加载导致 403
    const hasBili = props.followedAnchors.some(s => s.platform === Platform.BILIBILI);
    if (hasBili) {
      await ensureProxyStarted();
    }
    // 延迟到页面空闲或设定时间后再刷新关注列表，避免影响斗鱼分类/主播列表的首屏加载
    requestIdle(() => { refreshList(); });
  });
  
  onUnmounted(() => {
    clearAnimationTimeout();
  });
  </script>
  
  <style src="./index.css" scoped></style>
<style scoped>
/* 让刷新按钮在刷新中显示与 FollowOverlay 相同的 spinner */
.action-btn.refresh-btn .icon .refresh-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
}
.action-btn.refresh-btn .icon.refreshing .refresh-spinner {
  animation: spin 0.9s linear infinite;
}

/* 进度左侧的内联转圈样式 */
.refresh-spinner-inline {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.9s linear infinite;
  margin: 0; /* 使用容器的 gap 控制间距 */
}

/* 进度与转圈合并后的容器样式：共享统一圆角矩形背景 */
.progress-with-spinner {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 8px;
  border-radius: 8px;
  background: rgba(0, 218, 198, 0.12);
}
:root[data-theme="light"] .progress-with-spinner {
  background: rgba(80, 130, 255, 0.10);
}
/* 去掉进度文本自身背景，统一由容器提供 */
.progress-with-spinner .progress-label {
  background: transparent;
  padding: 0;
  margin: 0;
}

@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>