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
            <!-- 新增：刷新进度 -->
            <span v-if="isRefreshing" class="progress-label">{{ progressCurrent }}/{{ progressTotal }}</span>
          </button>
          <!-- 新增：展开悬浮关注列表按钮 -->
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
        <!-- 新增：刷新完成提示 -->
        <div v-if="showRefreshToast" class="refresh-toast">刷新完成</div>
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
  const listRef = ref<HTMLElement | null>(null);
  const isDragging = ref(false);
  const draggedIndex = ref(-1);
  const startY = ref(0);
  const currentY = ref(0);
  const justAddedIds = ref<string[]>([]);
  const animationTimeout = ref<number | null>(null);
  
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
  const showRefreshToast = ref(false);

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
  
      // 顺序刷新以便显示进度
      const updates: FollowedStreamer[] = [];
      for (const streamer of props.followedAnchors) {
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
            continue;
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
      }

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
      const finish = () => {
        isRefreshing.value = false;
        showRefreshToast.value = true;
        setTimeout(() => { showRefreshToast.value = false; }, 1500);
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
    refreshList();
  });
  
  onUnmounted(() => {
    clearAnimationTimeout();
  });
  </script>
  
  <style src="./index.css" scoped></style>