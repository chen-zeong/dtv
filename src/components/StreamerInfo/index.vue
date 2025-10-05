<template>
    <div class="streamer-info">
      <div class="streamer-layout">
        <div class="avatar-wrapper">
          <img v-if="avatarUrl && !showAvatarText" :src="avatarUrl" :alt="computedNickname" @error="handleAvatarError" class="avatar-img">
          <div v-else class="avatar-fallback">{{ computedNickname.charAt(0).toUpperCase() }}</div>
        </div>
  
        <div class="streamer-details-main">
          <h3 class="room-title" :title="computedRoomTitle">{{ computedRoomTitle }}</h3>
          <div class="streamer-meta-row">
            <span class="streamer-name">{{ computedNickname }}</span>
            <span :class="['status-tag', statusClass]">{{ getStatusText }}</span>
            <!-- Cookie status button for Bilibili -->
            <span v-if="props.platform === Platform.BILIBILI" class="cookie-status">
              <button class="cookie-status-btn" @click="openCookieModal">
                <span v-if="isCookieConfigured" class="cookie-configured">
                  <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/></svg>
                  已设置 Cookie
                </span>
                <span v-else class="cookie-unset">
                  未设置 Cookie
                </span>
              </button>
            </span>
            <span v-if="computedViewerCount > 0" class="viewers-tag">
              <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5M12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5s5 2.24 5-5s-2.24 5-5 5m0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3s3-1.34 3-3s-1.34-3-3-3"/></svg>
              {{ formattedViewerCount }}
            </span>
          </div>
        </div>
  
        <div class="streamer-actions">
          <div class="id-follow-container" ref="idFollowContainerRef" :class="{ 'highlight-moves-to-id': isFollowing }">
            <span class="streamer-id" ref="streamerIdRef" :class="{ 'text-active-on-highlight': isFollowing }">ID:{{ props.roomId }}</span>
            <button class="follow-btn" ref="followBtnRef" :class="{ 'text-active-on-highlight': !isFollowing, 'is-following': isFollowing }" @click="toggleFollow">
              <span class="follow-icon-wrapper">
                <span class="follow-icon icon-add" v-if="!isFollowing">
                  <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M19 12.998h-6v6h-2v-6H5v-2h6v-6h2v6h6z"/></svg>
                </span>
                <span class="follow-icon icon-check" v-else>
                  <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59L7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12L5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z"/></svg>
                </span>
              </span>
              <span class="follow-text">{{ isFollowing ? '取关' : '关注' }}</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Cookie modal -->
      <div v-if="isCookieModalOpen" class="cookie-modal-backdrop" @click.self="closeCookieModal">
        <div class="cookie-modal">
          <h4>B站 Cookie 设置</h4>
          <textarea v-model="cookieInput" placeholder="在此粘贴你的 B站 Cookie（仅本地存储）"></textarea>
          <div class="cookie-modal-actions">
            <button class="primary" @click="saveCookie">保存</button>
            <button class="danger" @click="clearCookie">清除</button>
            <button @click="closeCookieModal">取消</button>
          </div>
          <p class="cookie-tip">提示：仅保存在本机的 localStorage，用于后端请求头。</p>
        </div>
      </div>
    </div>
  </template>
  
  <style scoped>
  .streamer-info {
    width: 100%;
    padding: 24px 0 32px 0; /* 增加上下边距，特别是下边距 */
  }
  
  .streamer-layout {
    display: flex;
    align-items: center; /* 改为居中对齐，确保所有元素垂直居中 */
    gap: 16px;
    position: relative;
  }
  
  .avatar-wrapper {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    overflow: hidden;
    flex-shrink: 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    border: 2px solid rgba(255, 255, 255, 0.15);
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05));
    transition: transform 0.2s ease, border-color 0.2s ease;
  }
  
  .avatar-wrapper:hover {
    transform: translateY(-1px);
    border-color: rgba(255, 255, 255, 0.25);
  }
  
  .avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  
  .avatar-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    font-weight: 600;
    color: #ffffff;
    background: linear-gradient(135deg, #ff4757, #ff6b81);
  }
  
  .streamer-details-main {
    flex-grow: 1; /* Takes available space, pushing actions to the right */
    display: flex;
    flex-direction: column;
    gap: 8px; /* Space between title and meta-row */
    min-width: 0; /* Prevents overflow issues with long text */
  }
  
  .room-title {
    font-size: 1rem;
    font-weight: 600;
    color: #ffffff;
    margin: 0;
    line-height: 1.4;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2; /* Standard property */
    -webkit-box-orient: vertical;
    letter-spacing: 0.2px;
    text-align: left; /* Ensure room title is left-aligned */
  }
  
  .streamer-meta-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  
  .streamer-name {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.85);
    font-weight: 500;
  }

  /* Cookie status button */
  .cookie-status-btn {
    font-size: 0.75rem;
    padding: 3px 8px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: rgba(255, 255, 255, 0.85);
    background: rgba(255, 255, 255, 0.08);
  }
  .cookie-configured { color: #32b65c; display: inline-flex; align-items: center; gap: 4px; }
  .cookie-unset { color: #888888; }
  
  .streamer-actions {
    display: flex;
    margin-left: auto; 
    flex-shrink: 0; 
    align-items: center; /* 确保垂直居中 */
    align-self: center; /* 在flex容器中自身居中 */
  }
  
  .id-follow-container {
    display: flex;
    align-items: stretch; 
    border-radius: 6px; 
    overflow: hidden; /* Important for clipping the pseudo-element */
    box-shadow: 0 1px 3px rgba(0,0,0,0.15); 
    background-color: #2c2f38; /* Base container background */
    position: relative; /* For the pseudo-element */
    /* CSS variables for dynamic highlight, to be set by JS */
    --id-width: 100px; /* Default/initial value */
    --button-width: 80px; /* Default/initial value */
    --highlight-left: calc(100px + 1px); /* Default to button highlight */
    --highlight-width: calc(80px - 2px); /* Default to button highlight */
  }
  
  /* The sliding highlight pseudo-element */
  .id-follow-container::before {
    content: '';
    position: absolute;
    top: 2px; /* Small inset from container edges */
    bottom: 2px;
    height: calc(100% - 4px); /* Full height within insets */
    background-color: #439ed9; /* MODIFIED - Unified highlight color */
    z-index: 0; /* Behind text and icons */
    border-radius: 4px; /* Rounded corners for the highlight pill itself */
    transition: left 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275), width 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275); 
    /* Dynamic positioning via CSS variables */
    left: var(--highlight-left);
    width: var(--highlight-width);
  }
  
  
  .streamer-id,
  .follow-btn {
    background-color: transparent !important; 
    padding: 6px 10px; 
    font-weight: 500;
    display: flex; 
    align-items: center;
    justify-content: center; 
    position: relative; 
    z-index: 1;
    transition: color 0.2s ease-in-out 0.1s; 
    border: none; 
  }
  
  .follow-btn {
    cursor: pointer;
    /* flex: 2; Removed */
    width: 80px; /* Fixed width */
    min-width: 80px; /* Ensure it doesn't shrink below this */
    white-space: nowrap; 
    color: #9098a3; 
    border-top-right-radius: 6px; 
    border-bottom-right-radius: 6px;
    font-size: 0.8rem; 
  }
  
  .streamer-id {
    color: #9098a3; 
    border-top-left-radius: 6px; 
    border-bottom-left-radius: 6px;
    font-size: 0.75rem; 
    /* flex: 3; Removed */
    flex-grow: 1; /* Allow to grow */
    flex-shrink: 1; /* Allow to shrink */
    min-width: 80px; /* Minimum width same as button */
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap; /* Ensure ID text itself doesn't wrap */
    cursor: default; 
  }
  
  .streamer-id.text-active-on-highlight,
  /* .streamer-id.text-active-on-highlight .room-id-number, Removed as .room-id-number is gone */
  .follow-btn.text-active-on-highlight .follow-text, 
  .follow-btn.text-active-on-highlight .follow-icon-wrapper svg {
    color: white !important; /* Active text color when highlight is underneath */
  }
  
  /* Icon animation styles - preserved */
  .follow-btn .follow-icon-wrapper {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative; 
    width: 16px; 
    height: 16px; 
  }
  
  .follow-btn .follow-icon {
    display: inline-flex;
    align-items: center; 
    justify-content: center;
    transition: opacity 0.2s ease-in-out, transform 0.2s ease-in-out;
    position: absolute; 
    top: 0; left: 0; width: 100%; height: 100%; 
  }
  
  .follow-btn .follow-icon.icon-add {
    opacity: 1;
    transform: scale(1) rotate(0deg);
  }
  .follow-btn.is-following .follow-icon.icon-add {
    opacity: 0;
    transform: scale(0.5) rotate(-90deg);
  }
  
  .follow-btn .follow-icon.icon-check {
    opacity: 0;
    transform: scale(0.5) rotate(90deg);
  }
  .follow-btn.is-following .follow-icon.icon-check {
    opacity: 1;
    transform: scale(1) rotate(0deg);
  }
  
  /* .follow-text transition is now part of the general .follow-btn color transition */
  
  .status-tag {
    font-size: 0.7rem; 
    padding: 2px 7px; 
    border-radius: 5px; 
    color: #ffffff;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    line-height: 1.3; 
  }
  
  .status-tag.live {
    background: #32b65c; /* MODIFIED - Unified live color */
  }
  
  .status-tag.replay {
    background: linear-gradient(135deg, #5352ed, #6c6bff);
  }
  
  .status-tag.looping { /* New style for looping status */
    background: linear-gradient(135deg, #7879f1, #8a8bf8); /* Similar to replay or choose a new one */
  }
  
  .status-tag.offline {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .viewers-tag {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.85);
    display: inline-flex;
    align-items: center;
    gap: 5px; /* Adjusted gap */
    background: rgba(255, 255, 255, 0.08);
    padding: 3px 10px; /* Adjusted padding */
    border-radius: 8px; /* Consistent border radius */
  }
  
  .viewers-tag svg {
    width: 12px;
    height: 12px;
    opacity: 0.9;
  }

  /* Cookie modal styles */
  .cookie-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .cookie-modal {
    width: 520px;
    max-width: 90vw;
    background: #1f232b;
    border: 1px solid rgba(255,255,255,0.1);
    box-shadow: 0 10px 30px rgba(0,0,0,0.35);
    border-radius: 12px;
    padding: 16px;
    color: #fff;
  }
  .cookie-modal h4 { margin: 0 0 10px 0; font-size: 16px; }
  .cookie-modal textarea {
    width: 100%;
    min-height: 120px;
    border-radius: 8px;
    border: 1px solid rgba(255,255,255,0.15);
    background: rgba(255,255,255,0.06);
    color: #fff;
    padding: 8px;
    box-sizing: border-box;
  }
  .cookie-modal-actions {
    display: flex;
    gap: 10px;
    margin-top: 10px;
  }
  .cookie-modal-actions button {
    padding: 6px 12px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    background: rgba(255,255,255,0.12);
    color: #fff;
  }
  .cookie-modal-actions button.primary { background: #439ed9; }
  .cookie-modal-actions button.danger { background: #d94343; }
  .cookie-tip { margin-top: 8px; font-size: 12px; color: rgba(255,255,255,0.7); }
  
  @keyframes idPulse {
    0% { text-shadow: 0 0 2px rgba(251, 114, 153, 0); }
    50% { text-shadow: 0 0 6px rgba(251, 114, 153, 0.7); }
    100% { text-shadow: 0 0 2px rgba(251, 114, 153, 0); }
  }

  :root[data-theme="light"] .avatar-wrapper {
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
    border: 1px solid var(--border-color-light, #e0e0e0);
    background: var(--content-bg-light, #ffffff);
  }

  :root[data-theme="light"] .avatar-wrapper:hover {
    border-color: var(--border-color-hover-light, #cccccc);
  }

  :root[data-theme="light"] .avatar-fallback {
    color: var(--primary-text-light, #333333);
    background: linear-gradient(135deg, #e0e0e0, #f0f0f0);
  }

  :root[data-theme="light"] .room-title {
    color: var(--primary-text-light, #212529);
  }

  :root[data-theme="light"] .streamer-name {
    color: var(--secondary-text-light, #555555);
  }

  :root[data-theme="light"] .id-follow-container {
    background-color: var(--button-group-bg-light, #e9ecef); /* Light gray background for the container */
    box-shadow: 0 1px 2px rgba(0,0,0,0.08);
  }

  :root[data-theme="light"] .id-follow-container::before {
    background-color: #439ed9; /* MODIFIED - Unified highlight color */
  }

  :root[data-theme="light"] .streamer-id,
  :root[data-theme="light"] .follow-btn {
    color: var(--button-group-text-light, #495057); /* Text color for ID and button text before highlight */
  }

  :root[data-theme="light"] .streamer-id.text-active-on-highlight,
  :root[data-theme="light"] .follow-btn.text-active-on-highlight .follow-text,
  :root[data-theme="light"] .follow-btn.text-active-on-highlight .follow-icon-wrapper svg {
    color: white !important; /* Text color when highlighted in day mode */
  }

  :root[data-theme="light"] .status-tag {
    color: #ffffff; /* Text color on status tags is usually white for contrast */
  }

  :root[data-theme="light"] .status-tag.live {
    background: #32b65c; /* MODIFIED - Unified live color */
  }

  :root[data-theme="light"] .status-tag.replay {
    background: linear-gradient(135deg, var(--status-replay-bg-start-light, #007bff), var(--status-replay-bg-end-light, #0056b3)); /* Blue gradient for replay */
  }

  :root[data-theme="light"] .status-tag.looping { /* New style for looping status */
    background: linear-gradient(135deg, var(--status-looping-bg-start-light, #007bff), var(--status-looping-bg-end-light, #0056b3)); /* Blue gradient for looping */
  }

  :root[data-theme="light"] .status-tag.offline {
    background: var(--status-offline-bg-light, #6c757d); /* Gray for offline */
    color: var(--status-offline-text-light, #ffffff);
  }

  :root[data-theme="light"] .viewers-tag {
    color: var(--secondary-text-light, #555555);
    background: var(--tag-bg-light, #f0f0f0);
    /* border: 1px solid var(--border-color-light, #e0e0e0); Optional border */
  }

  :root[data-theme="light"] .viewers-tag svg {
    color: var(--icon-color-light, #888888);
    opacity: 1;
  }
  </style>
  
  <script setup lang="ts">
  import { ref, computed, onMounted, watch, onUpdated, nextTick } from 'vue'
  import { Platform } from '../../platforms/common/types'
  import type { StreamerDetails } from '../../platforms/common/types'
  import { fetchDouyuStreamerDetails } from '../../platforms/douyu/streamerInfoParser'
  import { getDouyinStreamerDetails } from '../../platforms/douyin/streamerInfoParser'
  import { invoke } from '@tauri-apps/api/core'

  // Helper: normalize avatar URL (strip wrappers/backticks, fix protocol)
  const normalizeAvatarUrl = (input: string | null | undefined): string => {
    if (!input) return ''
    let url = String(input).trim()
    // strip wrapping backticks or quotes
    const startsWithWrapper = url.startsWith('`') || url.startsWith('"') || url.startsWith("'")
    const endsWithWrapper = url.endsWith('`') || url.endsWith('"') || url.endsWith("'")
    if (startsWithWrapper && endsWithWrapper) {
      url = url.slice(1, -1).trim()
    }
    // handle protocol-relative URLs
    if (url.startsWith('//')) {
      url = 'https:' + url
    }
    // upgrade http to https
    if (url.startsWith('http://')) {
      url = 'https://' + url.slice('http://'.length)
    }
    // remove any whitespace inside the URL
    url = url.replace(/\s+/g, '')
    return url
  }
  const emit = defineEmits<{
    (e: 'follow', data: { id: string; platform: Platform; nickname: string; avatarUrl: string | null; roomTitle?: string }): void
    (e: 'unfollow', roomId: string): void
  }>()
  
  const props = defineProps<{
    roomId: string
    platform: Platform
    isFollowed: boolean
    title?: string | null
    anchorName?: string | null
    avatar?: string | null
    isLive?: boolean | null
    initialViewerCount?: number | null
  }>()
  
  const roomDetails = ref<StreamerDetails | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const showAvatarText = ref(false)
  
  const computedRoomTitle = computed(() => roomDetails.value?.roomTitle ?? props.title ?? '直播间标题加载中...')
  const computedNickname = computed(() => roomDetails.value?.nickname ?? props.anchorName ?? '主播昵称加载中...')
  const avatarUrl = ref(props.avatar || '')
  const computedViewerCount = computed(() => roomDetails.value?.viewerCount ?? 0)
  const isFollowing = computed(() => props.isFollowed)
  const computedStreamStatus = computed(() => {
    if (roomDetails.value) {
      if (roomDetails.value.isLive && roomDetails.value.isLooping) {
        return 'looping';
      }
      if (roomDetails.value.isLive) {
        return 'live';
      }
    } else if (props.isLive) { 
      return 'live';
    }
    return 'offline';
  });

  const statusClass = computed(() => {
    return computedStreamStatus.value;
  })
  
  const getStatusText = computed(() => {
    if (error.value) return '信息加载失败';
    const status = computedStreamStatus.value;
    if (status === 'live') return '直播中';
    if (status === 'looping') return '轮播中';
    return '未开播';
  })
  
  const formattedViewerCount = computed(() => {
    const count = computedViewerCount.value
    if (count >= 10000) {
      return (count / 10000).toFixed(1) + '万'
    }
    return count.toString()
  })

  // Proxy support for Bilibili avatar images
  const proxyBase = ref<string | null>(null)
  const ensureProxyStarted = async () => {
    if (!proxyBase.value) {
      try {
        const base = await invoke<string>('start_static_proxy_server')
        proxyBase.value = base
      } catch (e) {
        console.error('[StreamerInfo] Failed to start static proxy server', e)
      }
    }
  }
  const proxify = (url?: string): string => {
    if (!url) return ''
    if (proxyBase.value) {
      return `${proxyBase.value}/image?url=${encodeURIComponent(url)}`
    }
    return url
  }

  // Cookie status UI state
  const isCookieModalOpen = ref(false)
  const cookieInput = ref<string>('')
  const isCookieConfigured = computed(() => !!cookieInput.value && cookieInput.value.trim().length > 0)

  const openCookieModal = () => { isCookieModalOpen.value = true }
  const closeCookieModal = () => { isCookieModalOpen.value = false }
  const saveCookie = async () => {
    try {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('bilibili_cookie', cookieInput.value.trim())
      }
      isCookieModalOpen.value = false
      // Refresh details to reflect new cookie usage
      if (props.platform === Platform.BILIBILI) {
        await fetchRoomDetails()
      }
    } catch (e) {
      console.error('[StreamerInfo] saveCookie error:', e)
    }
  }
  const clearCookie = async () => {
    try {
      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('bilibili_cookie')
      }
      cookieInput.value = ''
      isCookieModalOpen.value = false
      if (props.platform === Platform.BILIBILI) {
        await fetchRoomDetails()
      }
    } catch (e) {
      console.error('[StreamerInfo] clearCookie error:', e)
    }
  }
  
  const fetchRoomDetails = async () => {
    if (props.platform === Platform.DOUYIN) {
      showAvatarText.value = !props.avatar;
      isLoading.value = false;
      roomDetails.value = null;
      avatarUrl.value = props.avatar || '';
      return;
    }

    if (props.platform === Platform.HUYA) {
      try {
        isLoading.value = true;
        error.value = null;
        roomDetails.value = null;
        showAvatarText.value = false;

        const res: any = await invoke('get_huya_unified_cmd', { roomId: props.roomId, quality: '原画' });
        const mapped: StreamerDetails = {
          roomId: props.roomId,
          platform: 'huya',
          roomTitle: (res && res.title) ? res.title : (props.title ?? '直播间标题加载中...'),
          nickname: (res && res.nick) ? res.nick : (props.anchorName ?? props.roomId),
          avatarUrl: (res && res.avatar) ? res.avatar : (props.avatar ?? null),
          isLive: !!(res && res.is_live),
        };
        roomDetails.value = mapped;
        avatarUrl.value = normalizeAvatarUrl(mapped.avatarUrl);
        showAvatarText.value = !avatarUrl.value;
      } catch (e: any) {
        console.error(`[StreamerInfo] HUYA fetchRoomDetails error for ${props.roomId}:`, e);
        error.value = e?.message || '获取虎牙房间信息失败';
        roomDetails.value = null;
        avatarUrl.value = props.avatar || '';
        showAvatarText.value = !props.avatar;
      } finally {
        isLoading.value = false;
      }
      return;
    }

    // 新增：B 站主播信息
    if (props.platform === Platform.BILIBILI) {
      try {
        isLoading.value = true;
        error.value = null;
        roomDetails.value = null;
        showAvatarText.value = false;

        const payload = { args: { room_id_str: props.roomId } };
        const savedCookie = (typeof localStorage !== 'undefined') ? (localStorage.getItem('bilibili_cookie') || null) : null;
        const res: any = await invoke('fetch_bilibili_streamer_info', {
          payload,
          cookie: savedCookie,
        });

        const mapped: StreamerDetails = {
          roomId: props.roomId,
          platform: 'bilibili',
          roomTitle: (res && res.title) ? res.title : (props.title ?? '直播间标题加载中...'),
          nickname: (res && res.anchor_name) ? res.anchor_name : (props.anchorName ?? props.roomId),
          avatarUrl: (res && res.avatar) ? res.avatar : (props.avatar ?? null),
          isLive: !!(res && res.status === 1),
        };
        roomDetails.value = mapped;
        await ensureProxyStarted();
        avatarUrl.value = proxify(normalizeAvatarUrl(mapped.avatarUrl));
        showAvatarText.value = !avatarUrl.value;
      } catch (e: any) {
        console.error(`[StreamerInfo] BILIBILI fetchRoomDetails error for ${props.roomId}:`, e);
        error.value = e?.message || '获取 B 站房间信息失败';
        roomDetails.value = null;
        await ensureProxyStarted();
        avatarUrl.value = proxify(normalizeAvatarUrl(props.avatar || ''));
        showAvatarText.value = !props.avatar;
      } finally {
        isLoading.value = false;
      }
      return;
    }

    isLoading.value = true;
    error.value = null;
    roomDetails.value = null;
    showAvatarText.value = false;

    try {
      if (props.platform === Platform.DOUYU) {
        roomDetails.value = await fetchDouyuStreamerDetails(props.roomId);
        avatarUrl.value = normalizeAvatarUrl(roomDetails.value?.avatarUrl || avatarUrl.value);
      } else {
        console.warn(`[StreamerInfo] Unsupported platform: ${props.platform}`);
        throw new Error(`Unsupported platform: ${props.platform}`);
      }

      if (!avatarUrl.value) {
        showAvatarText.value = true
      }

    } catch (e: any) {
      console.error(`[StreamerInfo] Error in fetchRoomDetails for ${props.platform}/${props.roomId}:`, e)
      error.value = e.message || 'Failed to load streamer details'
      showAvatarText.value = true
    } finally {
      isLoading.value = false
    }
  }
  
  const toggleFollow = () => {
    if (isFollowing.value) {
      emit('unfollow', props.roomId)
    } else {
      const followData = {
        id: props.roomId,
        platform: props.platform,
        nickname: computedNickname.value === '主播昵称加载中...' ? props.roomId : computedNickname.value,
        avatarUrl: avatarUrl.value,
        roomTitle: computedRoomTitle.value === '直播间标题加载中...' ? undefined : computedRoomTitle.value,
      }
      emit('follow', followData)
    }
  }
  
  const handleAvatarError = () => {
    console.warn(`[StreamerInfo] Avatar image failed to load for ${computedNickname.value} (URL: ${avatarUrl.value}). Displaying fallback.`);
    showAvatarText.value = true;
  };
  
  const idFollowContainerRef = ref<HTMLElement | null>(null);
  const streamerIdRef = ref<HTMLElement | null>(null);
  const followBtnRef = ref<HTMLElement | null>(null);
  
  const updateHighlightVars = () => {
    if (idFollowContainerRef.value && streamerIdRef.value && followBtnRef.value) {
      const idWidth = streamerIdRef.value.offsetWidth;
      const buttonWidth = followBtnRef.value.offsetWidth;

      idFollowContainerRef.value.style.setProperty('--id-width', `${idWidth}px`);
      idFollowContainerRef.value.style.setProperty('--button-width', `${buttonWidth}px`);

      if (isFollowing.value) {
        idFollowContainerRef.value.style.setProperty('--highlight-left', '2px');
        idFollowContainerRef.value.style.setProperty('--highlight-width', `calc(${idWidth}px - 4px)`);
      } else {
        idFollowContainerRef.value.style.setProperty('--highlight-left', `calc(${idWidth}px + 2px)`);
        idFollowContainerRef.value.style.setProperty('--highlight-width', `calc(${buttonWidth}px - 4px)`);
      }
    }
  };
  
  onMounted(() => {
    // Load saved cookie for status display
    if (typeof localStorage !== 'undefined') {
      cookieInput.value = localStorage.getItem('bilibili_cookie') || ''
    }
    fetchRoomDetails()
    nextTick(() => {
      updateHighlightVars();
    });
  })
  
  watch(() => [props.roomId, props.platform], (newValues, oldValues) => {
    if (newValues[0] !== oldValues[0] || newValues[1] !== oldValues[1]) {
      fetchRoomDetails()
    }
  }, { deep: true })

  watch(() => [props.title, props.anchorName, props.avatar], async (newValues, oldValues) => {
    if (props.platform === Platform.DOUYIN) {
      const hasChanged = newValues.some((val, index) => val !== oldValues[index])
      if (hasChanged) {
        roomDetails.value = await getDouyinStreamerDetails({
          roomId: props.roomId,
          initialTitle: props.title,
          initialAnchorName: props.anchorName,
          initialAvatar: props.avatar,
        })
        avatarUrl.value = normalizeAvatarUrl(roomDetails.value?.avatarUrl || avatarUrl.value)
        showAvatarText.value = !avatarUrl.value
      }
    } else {
      // For non-Douyin platforms, if parent updates avatar prop, reflect it
      if (newValues[2] !== oldValues[2]) {
        avatarUrl.value = normalizeAvatarUrl(props.avatar || '')
        showAvatarText.value = !avatarUrl.value
      }
    }
  })

  watch([() => props.roomId, () => props.platform, isFollowing], () => {
    nextTick(() => {
      updateHighlightVars();
    });
  }, { deep: true })

  watch(() => props.avatar, (newAvatar, oldAvatar) => {
    if (newAvatar !== oldAvatar) {
      showAvatarText.value = false; // Reset error state if avatar URL changes
    }
    if (newAvatar && showAvatarText.value) {
      showAvatarText.value = false;
    }
  });

  onUpdated(() => {
    nextTick(() => {
      updateHighlightVars();
    });
  })
  </script>