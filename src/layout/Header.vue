<template>
  <header class="app-header" data-tauri-drag-region>
    <div class="search-container" data-tauri-drag-region>
      <div class="search-box">
        <input 
          v-model="searchQuery" 
          :placeholder="placeholderText" 
          @input="handleSearch"
          @focus="showResults = true"
          @blur="handleBlur"
          class="search-input"
        />
        <button class="search-button" data-tauri-drag-region="none" @click="doSearch" :disabled="isLoadingSearch">
          <svg v-if="!isLoadingSearch" width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M7.333 12.667A5.333 5.333 0 1 0 7.333 2a5.333 5.333 0 0 0 0 10.667zM14 14l-4-4" 
                  stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <div v-else class="mini-spinner"></div>
        </button>
      </div>
      
      <div v-show="showResults" class="search-results-wrapper">
        <div v-if="isLoadingSearch" class="search-loading">搜索中...</div>
        <div v-else-if="searchError" class="search-error-message">{{ searchError }}</div>
        <div v-else-if="searchResults.length > 0" class="search-results-list">
          <div v-for="anchor in searchResults" 
              :key="anchor.platform + '-' + anchor.roomId"
              class="search-result-item"
              @mousedown="selectAnchor(anchor)"
          >
            <div class="result-avatar">
              <img v-if="anchor.avatar" :src="anchor.avatar" :alt="anchor.userName" class="avatar-img">
              <div v-else class="avatar-placeholder">{{ anchor.userName[0] }}</div>
            </div>
            
            <div class="result-main-content">
              <div class="result-line-1-main">
                <span class="result-name" :title="anchor.userName">{{ anchor.userName }}</span>
                <span class="live-status-badge styled-badge" :class="{ 'is-live': anchor.liveStatus }">
                  {{ anchor.liveStatus ? '直播中' : '未开播' }}
                </span>
              </div>
              <div class="result-line-2-main">
                <span class="result-room-title" :title="anchor.roomTitle || '无标题'">
                  {{ anchor.roomTitle || '无直播标题' }}
                </span>
                <span class="result-roomid styled-badge">
                  ID: {{ anchor.webId || anchor.roomId }}
                </span>
              </div>
            </div>

            <div class="result-meta-right">
              <span class="platform-tag styled-badge" 
                    :class="[anchor.platform.toLowerCase(), { 
                      'douyu': anchor.platform === Platform.DOUYU, 
                      'douyin': anchor.platform === Platform.DOUYIN,
                      'huya': anchor.platform === Platform.HUYA 
                    }]"
              >
                {{ anchor.platform === Platform.DOUYU ? '斗鱼' : (anchor.platform === Platform.DOUYIN ? '抖音' : (anchor.platform === Platform.HUYA ? '虎牙' : anchor.platform)) }}
              </span>
            </div>

          </div>
        </div>
        <div v-else-if="trimmedQuery && !isLoadingSearch && !searchError" class="search-no-results">
            无匹配结果。
            <button
              v-if="isPureNumeric(trimmedQuery)"
              class="search-fallback-btn"
              @mousedown.prevent="tryEnterRoom(trimmedQuery)"
              @click.prevent="tryEnterRoom(trimmedQuery)"
            >
              尝试进入直播间 {{ trimmedQuery }}
            </button>
        </div>
      </div>
    </div>

    <div class="header-actions">
        <button 
        @click="toggleTheme" 
        class="theme-btn"
        :class="{ 'is-animating': themeToggleAnimating }"
        :title="effectiveTheme === 'dark' ? '切换到日间模式' : '切换到夜间模式'"
        data-tauri-drag-region="none"
      >
        <Transition name="theme-icon" mode="out-in">
          <Sun
            v-if="effectiveTheme === 'dark'"
            key="sun"
            class="theme-icon"
            :stroke-width="1.8"
          />
          <Moon
            v-else
            key="moon"
            class="theme-icon"
            :stroke-width="1.8"
          />
        </Transition>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Platform } from '../platforms/common/types';
import { useThemeStore } from '../stores/theme';
import { useRoute } from 'vue-router';
import { Sun, Moon } from 'lucide-vue-next';

interface DouyinApiStreamInfo {
  title?: string | null;
  anchor_name?: string | null;
  avatar?: string | null;
  status?: number | null;
  error_message?: string | null;
  web_rid?: string | null;
}

interface HuyaAnchorItem {
  room_id: string;
  avatar: string;
  user_name: string;
  live_status: boolean;
  title: string;
}

interface SearchResultItem {
  platform: Platform;
  roomId: string;
  webId?: string | null;
  userName: string;
  roomTitle?: string | null;
  avatar: string | null;
  liveStatus: boolean;
  fansCount?: string;
  category?: string;
  rawStatus?: number | null;
}

const searchQuery = ref('');
const trimmedQuery = computed(() => searchQuery.value.trim());
const searchResults = ref<SearchResultItem[]>([]);
const showResults = ref(false);
const searchError = ref<string | null>(null);
const isLoadingSearch = ref(false);

const emit = defineEmits(['selectAnchor']);

const themeStore = useThemeStore();
const route = useRoute();

// Proxy support for Bilibili avatar images in search results
const proxyBase = ref<string | null>(null);
const ensureProxyStarted = async () => {
  if (!proxyBase.value) {
    try {
      const base = await invoke<string>('start_static_proxy_server');
      proxyBase.value = base;
    } catch (e) {
      console.error('[Header] Failed to start static proxy server', e);
    }
  }
};
const proxify = (url?: string | null): string | null => {
  if (!url) return null;
  if (proxyBase.value) {
    return `${proxyBase.value}/image?url=${encodeURIComponent(url)}`;
  }
  return url;
};

const effectiveTheme = computed(() => themeStore.getEffectiveTheme());

const currentPlatform = computed<Platform>(() => {
  const name = route.name as string | undefined;
  const path = route.path;

  // Prefer route name for accuracy
  if (name) {
    if (name === 'douyinPlayer' || name === 'DouyinHome') return Platform.DOUYIN;
    if (name === 'huyaPlayer' || name === 'HuyaHome') return Platform.HUYA;
    if (name === 'bilibiliPlayer' || name === 'BilibiliHome') return Platform.BILIBILI;
    if (name === 'douyuPlayer' || name === 'DouyuHome') return Platform.DOUYU;
  }

  // Fallback to path matching (covers both home and player routes)
  if (path.startsWith('/player/douyin') || path.startsWith('/douyin')) return Platform.DOUYIN;
  if (path.startsWith('/player/huya') || path.startsWith('/huya')) return Platform.HUYA;
  if (path.startsWith('/player/bilibili') || path.startsWith('/bilibili')) return Platform.BILIBILI;
  if (path.startsWith('/player/douyu') || path.startsWith('/')) return Platform.DOUYU;

  // Default to Douyu
  return Platform.DOUYU;
});

const placeholderText = computed(() => {
  if (currentPlatform.value === Platform.DOUYU) return '搜索斗鱼主播';
  if (currentPlatform.value === Platform.HUYA) return '搜索虎牙主播';
  if (currentPlatform.value === Platform.DOUYIN) return '搜索抖音房间ID';
  if (currentPlatform.value === Platform.BILIBILI) return '搜索B站房间号';
  return '搜索主播';
});

const themeToggleAnimating = ref(false);
let themeAnimationTimer: number | null = null;

const triggerThemeAnimation = () => {
  if (themeAnimationTimer !== null) {
    window.clearTimeout(themeAnimationTimer);
  }
  themeToggleAnimating.value = true;
  themeAnimationTimer = window.setTimeout(() => {
    themeToggleAnimating.value = false;
    themeAnimationTimer = null;
  }, 360);
};

const toggleTheme = () => {
  triggerThemeAnimation();
  const currentTheme = themeStore.getEffectiveTheme();
  if (currentTheme === 'light') {
    themeStore.setUserPreference('dark');
  } else {
    themeStore.setUserPreference('light');
  }
};

onBeforeUnmount(() => {
  if (themeAnimationTimer !== null) {
    window.clearTimeout(themeAnimationTimer);
  }
});

let searchTimeout: number | null = null;

const isPureNumeric = (value: string): boolean => /^\d+$/.test(value);

const resetSearchState = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
    searchTimeout = null;
  }
  searchQuery.value = '';
  searchResults.value = [];
  searchError.value = null;
  showResults.value = false;
  isLoadingSearch.value = false;
};

const handleSearch = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  searchError.value = null;
  isLoadingSearch.value = true;
  
  searchTimeout = window.setTimeout(() => {
    performSearchBasedOnInput();
  }, 500);
};

const performSearchBasedOnInput = async () => {
  const query = trimmedQuery.value;
  if (!query) {
    searchResults.value = [];
    showResults.value = false;
    isLoadingSearch.value = false;
    return;
  }
  searchQuery.value = query;

  if (currentPlatform.value === Platform.DOUYIN) {
    await performDouyinIdSearch(query);
  } else if (currentPlatform.value === Platform.HUYA) {
    await performHuyaSearch(query);
  } else if (currentPlatform.value === Platform.BILIBILI) {
    await performBilibiliSearch(query);
  } else {
    await performDouyuSearch(query);
  }
  isLoadingSearch.value = false;
};

const performDouyinIdSearch = async (userInputRoomId: string) => {
  searchResults.value = [];
  searchError.value = null;
  isLoadingSearch.value = true;
  try {
    const payloadData = { args: { room_id_str: userInputRoomId } };
    const douyinInfo = await invoke<DouyinApiStreamInfo>('fetch_douyin_streamer_info', {
      payload: payloadData,
    });
    isLoadingSearch.value = false;
      if (douyinInfo) {
        if (douyinInfo.anchor_name) {
          const isLive = douyinInfo.status === 2;
          const webId = (douyinInfo as any).web_rid ?? userInputRoomId;
          searchResults.value = [{
            platform: Platform.DOUYIN,
            roomId: webId,
            webId,
            userName: douyinInfo.anchor_name || '未知抖音主播',
            roomTitle: douyinInfo.title || null,
            avatar: douyinInfo.avatar || null,
            liveStatus: isLive,
            rawStatus: douyinInfo.status,
        }];
        }
    } else {
      searchError.value = '搜索服务暂时不可用，请稍后再试。';
    }
  } catch (e: any) {
    isLoadingSearch.value = false;
    searchError.value = '搜索服务暂时不可用，请稍后再试。';
  }
  showResults.value = true;
};

const performHuyaSearch = async (keyword: string) => {
  searchResults.value = [];
  searchError.value = null;
  isLoadingSearch.value = true;
  try {
    const items = await invoke<HuyaAnchorItem[]>('search_huya_anchors', { keyword, page: 1 });
    // Ensure static proxy server is running for Huya avatars
    await ensureProxyStarted();
    isLoadingSearch.value = false;
    if (Array.isArray(items) && items.length > 0) {
      searchResults.value = items.map((item): SearchResultItem => ({
        platform: Platform.HUYA,
        roomId: item.room_id,
        userName: item.user_name || '虎牙主播',
        roomTitle: item.title || null,
        avatar: proxify(item.avatar || null),
        liveStatus: !!item.live_status,
      }));
      searchError.value = null;
    }
  } catch (e) {
    isLoadingSearch.value = false;
    searchError.value = '搜索服务暂时不可用，请稍后再试。';
  }
  showResults.value = true;
};

const performDouyuSearch = async (keyword: string) => {
  searchResults.value = [];
  searchError.value = null;
  isLoadingSearch.value = true;
  try {
    const response = await invoke<string>('search_anchor', { keyword });
    isLoadingSearch.value = false;
    const data = JSON.parse(response);
    if (data.error === 0 && data.data && data.data.relateUser) {
      searchResults.value = data.data.relateUser
        .filter((item: any) => item.type === 1)
        .map((item: any): SearchResultItem => {
          const anchorInfo = item.anchorInfo;
          const isReallyLive = anchorInfo.isLive === 1 && anchorInfo.videoLoop !== 1;
          return {
            platform: Platform.DOUYU,
            roomId: anchorInfo.rid.toString(),
            userName: anchorInfo.nickName,
            roomTitle: anchorInfo.roomName || anchorInfo.description || null,
            avatar: anchorInfo.avatar,
            liveStatus: isReallyLive,
            fansCount: anchorInfo.fansNumStr,
            category: anchorInfo.cateName,
          };
        });
      searchError.value = null;
    } else {
      searchError.value = '搜索服务暂时不可用，请稍后再试。';
    }
  } catch (e) {
    isLoadingSearch.value = false;
    searchError.value = '搜索服务暂时不可用，请稍后再试。';
  }
  showResults.value = true;
};

const doSearch = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  isLoadingSearch.value = true;
  performSearchBasedOnInput();
};

const handleBlur = () => {
  setTimeout(() => {
    if (!isLoadingSearch.value && !searchError.value) {
       showResults.value = false;
    }
  }, 300);
};

type BilibiliSearchItem = {
  room_id: string;
  title: string;
  cover: string;
  anchor: string;
  avatar: string;
  watching: string;
  area: string;
  is_live: boolean;
};

const performBilibiliSearch = async (keyword: string) => {
  searchResults.value = [];
  searchError.value = null;
  isLoadingSearch.value = true;
  try {
    const response = await invoke<BilibiliSearchItem[]>('search_bilibili_rooms', {
      keyword,
      page: 1,
    });
    await ensureProxyStarted();
    if (Array.isArray(response) && response.length > 0) {
      searchResults.value = response.map((item) => ({
        platform: Platform.BILIBILI,
        roomId: item.room_id,
        webId: item.room_id,
        userName: item.anchor || '未知B站主播',
        roomTitle: item.title || null,
        avatar: proxify(item.avatar),
        liveStatus: item.is_live,
        fansCount: item.watching,
        category: item.area,
      }));
    }
  } catch (e) {
    searchError.value = '搜索服务暂时不可用，请稍后再试。';
  } finally {
    isLoadingSearch.value = false;
    showResults.value = true;
  }
};

const selectAnchor = (anchor: SearchResultItem) => {
  emit('selectAnchor', {
    id: anchor.webId || anchor.roomId,
    platform: anchor.platform,
    nickname: anchor.userName,
    avatarUrl: anchor.avatar,
    currentRoomId: undefined,
  });
  resetSearchState();
};

const tryEnterRoom = (roomId: string) => {
  if (!roomId) return;
  emit('selectAnchor', {
    id: roomId,
    platform: currentPlatform.value,
    nickname: roomId,
    avatarUrl: null,
    currentRoomId: undefined,
  });
  resetSearchState();
};

</script>

<style scoped>
.app-header {
  /* Default: Detail Focused - Night */
  --h-bg: #1a1b1e; /* User-defined Dark Background */
  --h-accent: #1DB954; /* Bright Green for "Live" status (same as day) */
  --h-accent-rgb: 29, 185, 84; /* RGB for #1DB954 */
  --h-accent-text-color: #FFFFFF; /* White text on green accent */
  --h-douyu-platform-color: #FF7F00; /* Deep Orange for Douyu */
  --h-douyu-platform-text-color: #FFFFFF;
  --h-douyin-platform-color: #2A0D2E; /* Deep Purple-Black for Douyin */
  --h-douyin-platform-text-color: #FFFFFF;
  --h-huya-platform-color: #ff4d4f; /* Tiger Red for Huya tag */
  --h-huya-platform-text-color: #FFFFFF;
  --h-text-primary: #E1E3E8; /* Light Grayish White (Main Text) */
  --h-text-secondary: #969BAD; /* Neutral Cool Light Gray (Secondary Text/Icons) */
  --h-border: #2C2E33; /* Darker Gray, slightly lighter than BG (Header Bottom Border) */
  --h-search-bg: #232529; /* Search BG - Lighter than main BG */
  --h-search-border: #383A3F; /* Search Box Normal Border */
  --h-search-focus-border-color: #60687A; /* Search Box Focus Border Color */
  --h-search-focus-shadow: rgba(120, 130, 150, 0.35); /* Subtle Glow for Search Focus */
  --h-btn-hover-bg: #2E3035; /* Button Hover BG */
  --h-results-bg: #1F2124; /* Search Results Dropdown BG */
  --h-results-item-hover-bg: #2A2C30; /* Dropdown Item Hover */
  --h-scroll-thumb: #585E70; /* Scrollbar Thumb */
  --h-scroll-track: #232529; /* Scrollbar Track */
  --h-error-text: #FF6B6B; /* Clear Red for actual errors */
  --h-search-message-text-color: #AEB5C0; /* Grayish white for "no results" */

  display: flex;
  justify-content: center; 
  align-items: center;
  padding: 10px 20px; 
  background-color: var(--h-bg);
  border-bottom: 1px solid var(--h-border);
  height: 64px; 
  box-sizing: border-box;
  position: relative; 
  top: 0;
  z-index: 1000;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

/* Light Theme: Detail Focused - Day */
:root[data-theme="light"] .app-header {
  --h-bg: #FFFFFF; /* Main BG White for better contrast with #f6f6f6 search */
  --h-accent: #1DB954; /* Bright Green for "Live" status */
  --h-accent-rgb: 29, 185, 84;
  --h-accent-text-color: #FFFFFF;
  --h-douyu-platform-color: #F08000; /* Lighter Orange for Douyu */
  --h-douyu-platform-text-color: #FFFFFF;
  --h-douyin-platform-color: #3C003C; /* Lighter Purple-Black for Douyin */
  --h-douyin-platform-text-color: #FFFFFF;
  --h-huya-platform-color: #ff7a45; /* Light Tiger Red */
  --h-huya-platform-text-color: #FFFFFF;
  --h-text-primary: #2c3e50; /* Dark Slate Blue (Main Text) */
  --h-text-secondary: #7f8c8d; /* Neutral Gray (Secondary Text/Icons) */
  --h-border: #e0e0e0; /* Light Gray (Header Bottom Border) */
  --h-search-bg: #f6f6f6; /* User-defined Search BG */
  --h-search-border: #dcdcdc; /* Search Box Normal Border - Light Gray */
  --h-search-focus-border-color: #a0a0a0; /* Search Box Focus Border Color - Medium Gray */
  --h-search-focus-shadow: rgba(160, 160, 160, 0.3); /* Subtle Glow for Search Focus */
  --h-btn-hover-bg: #e9e9e9; /* Button Hover BG */
  --h-results-bg: #FFFFFF; /* White Dropdown BG */
  --h-results-item-hover-bg: #f0f0f0; /* Off-white for item hover */
  --h-scroll-thumb: #bdc3c7; /* Medium-Light Gray (Scrollbar Thumb) */
  --h-scroll-track: #e0e0e0; /* Light Gray (Scrollbar Track) */
  --h-error-text: #e74c3c; /* Standard Error Red for actual errors */
  --h-search-message-text-color: #7f8c8d; /* Gray for "no results" */
}

.search-container {
  width: 400px;
  max-width: 400px;
  position: relative;
}

.header-actions { 
  display: flex;
  align-items: center;
  gap: 10px; 
  position: absolute;
  right: 20px;
  top: 50%;
  transform: translateY(-50%);
}

.theme-btn {
  background-color: transparent; 
  color: var(--h-text-secondary);
  border: none; 
  border-radius: 8px; 
  padding: 8px; 
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s ease, color 0.2s ease, transform 0.35s ease;
  width: 38px; 
  height: 38px; 
}

.theme-btn:hover {
  background-color: var(--h-btn-hover-bg);
  color: var(--h-text-secondary); 
}

.theme-btn.is-animating {
  transform: scale(1.08) rotate(-14deg);
}

.theme-icon {
  width: 20px;
  height: 20px;
  color: currentColor;
}

.theme-icon-enter-active,
.theme-icon-leave-active {
  transition: opacity 0.2s ease, transform 0.28s ease;
}

.theme-icon-enter-from,
.theme-icon-leave-to {
  opacity: 0;
  transform: scale(0.7) rotate(-20deg);
}

.theme-icon-enter-to,
.theme-icon-leave-from {
  opacity: 1;
  transform: scale(1) rotate(0deg);
}

.search-box {
  display: flex;
  align-items: center;
  background-color: var(--h-search-bg);
  border-radius: 8px;
  padding: 0 8px;
  border: 1px solid var(--h-search-border); 
  box-shadow: none; 
  transition: background-color 0.3s ease, border-color 0.3s ease, box-shadow 0.2s ease;
  height: 40px; 
}

.search-box:focus-within {
  border-color: var(--h-search-focus-border-color);
  box-shadow: 0 0 0 3px var(--h-search-focus-shadow);
}

.search-input {
  flex-grow: 1;
  padding: 10px 8px;
  border: none;
  outline: none;
  font-size: 14px;
  background-color: transparent;
  color: var(--h-text-primary);
}

.search-input::placeholder {
  color: var(--h-text-secondary);
  opacity: 1;
}

.search-button {
  background: none;
  border: none;
  padding: 8px;
  cursor: pointer;
  color: var(--h-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.2s ease;
}
.search-button:hover {
  color: var(--h-text-primary); 
}
.search-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.mini-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--h-text-secondary);
  border-top-color: var(--h-text-primary); 
  border-radius: 50%;
  animation: mini-spin 0.8s linear infinite;
}

@keyframes mini-spin {
  to { transform: rotate(360deg); }
}

.search-results-wrapper {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  background-color: var(--h-results-bg);
  border-radius: 8px;
  box-shadow: 0 6px 18px rgba(0,0,0,0.2); 
  max-height: 400px;
  overflow-y: auto;
  z-index: 1001; 
  border: 1px solid var(--h-border); 
  padding: 4px;
}

:root[data-theme="light"] .search-results-wrapper {
    box-shadow: 0 6px 18px rgba(0,0,0,0.1); 
}

.search-loading, .search-error-message, .search-no-results {
  padding: 12px 16px;
  text-align: center;
  /* color will be set by specific classes or message types */
}

.search-loading {
  color: var(--h-text-secondary);
}

.search-error-message,
.search-no-results {
  color: var(--h-search-message-text-color);
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: stretch;
}

.search-fallback-btn {
  width: 100%;
  border: none;
  border-radius: 6px;
  padding: 12px;
  font-size: 15px;
  font-weight: 600;
  color: var(--h-text-primary);
  background: linear-gradient(135deg, rgba(30, 136, 229, 0.16), rgba(30, 82, 229, 0.28));
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease, background 0.2s ease;
}

.search-fallback-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(30, 82, 229, 0.25);
}

.search-fallback-btn:active {
  transform: translateY(0);
  box-shadow: 0 3px 10px rgba(30, 82, 229, 0.18);
}

:root[data-theme="light"] .search-fallback-btn {
  background: linear-gradient(135deg, rgba(64, 158, 255, 0.18), rgba(64, 118, 255, 0.3));
  color: #1f2d3d;
}

.search-results-list {
  display: flex;
  flex-direction: column;
}

.search-result-item {
  display: flex;
  align-items: center;
  padding: 10px 12px;
  cursor: pointer;
  border-radius: 6px;
  transition: background-color 0.15s ease;
  gap: 12px;
  color: var(--h-text-primary);
}

.search-result-item:hover {
  background-color: var(--h-results-item-hover-bg);
}

.result-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  overflow: hidden;
  flex-shrink: 0;
  background-color: var(--h-search-bg); 
  display: flex;
  align-items: center;
  justify-content: center;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  font-size: 18px;
  font-weight: 500;
  color: var(--h-text-primary);
}

.result-main-content {
  flex-grow: 1;
  overflow: hidden; 
}

.result-line-1-main, .result-line-2-main {
  display: flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-name {
  font-weight: 500;
  color: var(--h-text-primary);
  flex-shrink: 0; 
  max-width: 150px; 
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-room-title {
  font-size: 0.8rem;
  color: var(--h-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  flex-grow: 1;
}

.styled-badge {
  padding: 3px 8px; /* Restored larger padding for platform-tag */
  border-radius: 12px; /* Restored larger border-radius for platform-tag */
  font-size: 0.7rem;
  font-weight: 500;
  white-space: nowrap;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  /* line-height: 1.4; Removed from general, will be specific to live-status-badge */
}

.live-status-badge {
  background-color: var(--h-search-bg); 
  color: var(--h-text-secondary);
  /* Specific smaller padding and border-radius for live-status-badge */
  padding-top: 1px;
  padding-bottom: 1px;
  padding-left: 6px;
  padding-right: 6px;
  border-radius: 10px;
  line-height: 1.4; /* For vertical centering in the smaller badge */
}

.live-status-badge.is-live {
  background-color: var(--h-accent);
  color: var(--h-accent-text-color);
}

.result-roomid {
  font-size: 0.7rem;
  display: none !important; 
}

.result-meta-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  flex-shrink: 0;
}

.platform-tag {
  color: white; 
}

.platform-tag.douyu {
  background-color: var(--h-douyu-platform-color);
  color: var(--h-douyu-platform-text-color);
}

.platform-tag.douyin {
  background-color: var(--h-douyin-platform-color);
  color: var(--h-douyin-platform-text-color);
}

.platform-tag.huya {
  background-color: var(--h-huya-platform-color);
  color: var(--h-huya-platform-text-color);
}

.search-results-wrapper::-webkit-scrollbar {
  width: 6px;
}

.search-results-wrapper::-webkit-scrollbar-track {
  background: var(--h-scroll-track); 
  border-radius: 3px;
}

.search-results-wrapper::-webkit-scrollbar-thumb {
  background-color: var(--h-scroll-thumb);
  border-radius: 3px;
  border: 1px solid var(--h-scroll-track); 
}

.search-results-wrapper::-webkit-scrollbar-thumb:hover {
  background-color: var(--h-text-secondary); 
}

</style>
