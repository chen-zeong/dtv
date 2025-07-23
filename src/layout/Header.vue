<template>
  <header class="app-header">
    <div class="search-container">
      <div class="search-box">
        <input 
          v-model="searchQuery" 
          placeholder="搜索斗鱼主播 / 抖音房间ID" 
          @input="handleSearch"
          @focus="showResults = true"
          @blur="handleBlur"
          class="search-input"
        />
        <button class="search-button" @click="doSearch" :disabled="isLoadingSearch">
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
                  ID: {{ anchor.roomId }}
                </span>
              </div>
            </div>

            <div class="result-meta-right">
              <span class="platform-tag styled-badge" 
                    :class="[anchor.platform.toLowerCase(), { 
                      'douyu': anchor.platform === Platform.DOUYU, 
                      'douyin': anchor.platform === Platform.DOUYIN 
                    }]"
              >
                {{ anchor.platform === Platform.DOUYU ? '斗鱼' : (anchor.platform === Platform.DOUYIN ? '抖音' : anchor.platform) }}
              </span>
            </div>

          </div>
        </div>
        <div v-else-if="searchQuery.trim() && !isLoadingSearch && !searchError" class="search-no-results">
            无匹配结果。
        </div>
      </div>
    </div>

    <div class="header-actions">
      <button 
        @click="toggleTheme" 
        class="theme-btn"
        :title="effectiveTheme === 'dark' ? '切换到日间模式' : '切换到夜间模式'"
      >
        <svg v-if="effectiveTheme === 'dark'" width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z" />
        </svg>
        <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21c1.33 0 2.597-.266 3.752-.748z" />
        </svg>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Platform } from '../platforms/common/types';
import { useThemeStore } from '../stores/theme';

interface DouyinApiStreamInfo {
  title?: string | null;
  anchor_name?: string | null;
  avatar?: string | null;
  status?: number | null;
  error_message?: string | null;
}

interface SearchResultItem {
  platform: Platform;
  roomId: string;
  userName: string;
  roomTitle?: string | null;
  avatar: string | null;
  liveStatus: boolean;
  fansCount?: string;
  category?: string;
  rawStatus?: number | null;
}

const searchQuery = ref('');
const searchResults = ref<SearchResultItem[]>([]);
const showResults = ref(false);
const searchError = ref<string | null>(null);
const isLoadingSearch = ref(false);

const emit = defineEmits(['selectAnchor']);

const themeStore = useThemeStore();

const effectiveTheme = computed(() => themeStore.getEffectiveTheme());

const toggleTheme = () => {
  const currentTheme = themeStore.getEffectiveTheme();
  if (currentTheme === 'light') {
    themeStore.setUserPreference('dark');
  } else {
    themeStore.setUserPreference('light');
  }
};

let searchTimeout: number | null = null;

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
  const query = searchQuery.value.trim();
  if (!query) {
    searchResults.value = [];
    showResults.value = false;
    isLoadingSearch.value = false;
    return;
  }

  const douyinIdRegex = /^\d{10,}$/;

  if (douyinIdRegex.test(query)) {
    await performDouyinIdSearch(query);
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
      if (douyinInfo.error_message) {
        searchError.value = '没有搜索到主播。';
      } else if (douyinInfo.anchor_name) {
        const isLive = douyinInfo.status === 2;
        searchResults.value = [{
          platform: Platform.DOUYIN,
          roomId: userInputRoomId,
          userName: douyinInfo.anchor_name || '未知抖音主播',
          roomTitle: douyinInfo.title || null,
          avatar: douyinInfo.avatar || null,
          liveStatus: isLive,
          rawStatus: douyinInfo.status,
        }];
      } else {
        searchError.value = '没有搜索到主播。';
      }
    } else {
      searchError.value = '没有搜索到主播。';
    }
  } catch (e: any) {
    isLoadingSearch.value = false;
    searchError.value = '没有搜索到主播。';
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
      if (searchResults.value.length === 0) {
        searchError.value = '没有搜索到主播。';
      } else {
        searchError.value = null;
      }
    } else {
      searchError.value = '没有搜索到主播。';
    }
  } catch (e) {
    isLoadingSearch.value = false;
    searchError.value = '没有搜索到主播。';
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

const selectAnchor = (anchor: SearchResultItem) => {
  emit('selectAnchor', {
    id: anchor.roomId,
    platform: anchor.platform,
    nickname: anchor.userName,
    avatarUrl: anchor.avatar,
  });
  searchQuery.value = '';
  searchResults.value = [];
  searchError.value = null;
  showResults.value = false;
  isLoadingSearch.value = false;
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
  transition: background-color 0.2s ease, color 0.2s ease;
  width: 38px; 
  height: 38px; 
}

.theme-btn:hover {
  background-color: var(--h-btn-hover-bg);
  color: var(--h-text-secondary); 
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