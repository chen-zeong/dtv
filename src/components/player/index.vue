<template>
  <div class="player-page" :class="{ 'web-fs': isInWebFullscreen || isInNativePlayerFullscreen }">
    <button v-if="!isInWebFullscreen" @click="$emit('close-player')" class="player-close-btn" title="关闭播放器">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>

    <div class="player-layout">
      <div class="main-content">
        <div v-if="!roomId" class="empty-player">
          <div class="empty-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
               <circle cx="12" cy="12" r="10"></circle>
               <line x1="12" y1="16" x2="12" y2="12"></line>
               <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
          </div>
          <h3>未选择直播间</h3>
          <p>请从首页选择一个直播间开始观看。</p>
        </div>
        <div v-else-if="isLoadingStream" class="loading-player">
          <div class="spinner"></div>
          <p>加载直播流中...</p>
        </div>
        <div v-else-if="isOfflineError" class="offline-player">
          <!-- Display StreamerInfo if room details are available -->
          <StreamerInfo 
            v-if="props.roomId && props.platform"
            :room-id="props.roomId"
            :platform="props.platform"
            :title="playerTitle"
            :anchor-name="playerAnchorName"
            :avatar="playerAvatar"
            :is-live="false"
            :is-followed="props.isFollowed"
            @follow="$emit('follow', $event)"
            @unfollow="$emit('unfollow', $event)"
            class="streamer-info-offline"
          />
          <div class="offline-message">
            <div class="offline-icon">
              <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M16 16.427A4.002 4.002 0 0 0 12.005 20a4 4 0 0 0-3.995-3.573M12 12V2M8.5 7L7 5.5M15.5 7l1.5-1.5M5.562 10.223l-1.842.511M18.438 10.223l1.842.511M12 2a3.5 3.5 0 0 1 3.5 3.5V12H8.5V5.5A3.5 3.5 0 0 1 12 2z"/>
                <line x1="1" y1="1" x2="23" y2="23" stroke-width="2"></line> 
              </svg>
            </div>
            <h3>😴 获取直播流失败了</h3>
            <p>主播当前未开播，请稍后再来。</p>
            <button @click="retryInitialization" class="retry-btn">再试一次</button>
          </div>
        </div>
        <div v-else-if="streamError && !isOfflineError" class="error-player">
          <div class="error-icon">
             <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
          <h3>加载失败</h3>
          <p>{{ streamError }}</p>
          <button @click="retryInitialization" class="retry-btn">再试一次</button>
        </div>
        <div v-else class="player-container">
          <StreamerInfo
            v-if="props.roomId"
            :room-id="props.roomId"
            :platform="props.platform"
            :title="playerTitle"
            :anchor-name="playerAnchorName"
            :avatar="playerAvatar"
            :is-followed="props.isFollowed"
            :is-live="playerIsLive"
            @follow="$emit('follow', $event)"
            @unfollow="$emit('unfollow', $event)"
            class="streamer-info"
            v-show="!isInWebFullscreen"
            :class="{'hidden-panel': isInWebFullscreen}"
          />
          <div class="video-container">
            <div ref="playerContainerRef" class="video-player"></div>
          </div>
        </div>
      </div>

      <DanmuList 
        v-if="roomId && !isLoadingStream && !streamError" 
        :room-id="props.roomId"
        :messages="danmakuMessages"
        v-show="!isFullScreen" 
        class="danmu-panel" 
        :class="{'hidden-panel': isFullScreen}"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted, shallowRef, nextTick } from 'vue';
import Player from 'xgplayer';
import FlvPlugin from 'xgplayer-flv';
import DanmuJs from 'danmu.js';
import 'xgplayer/dist/index.min.css';
import Plugin, { POSITIONS } from 'xgplayer/es/plugin/plugin.js';

import './player.css';

import { Platform as StreamingPlatform } from '../../platforms/common/types';
import type { DanmakuMessage, DanmuOverlayInstance } from './types';

// Platform-specific player helpers
import { getDouyuStreamConfig, startDouyuDanmakuListener, stopDouyuDanmaku, stopDouyuProxy } from '../../platforms/douyu/playerHelper';
import { fetchAndPrepareDouyinStreamConfig, startDouyinDanmakuListener, stopDouyinDanmaku } from '../../platforms/douyin/playerHelper';
import { getHuyaStreamConfig, startHuyaDanmakuListener, stopHuyaDanmaku, stopHuyaProxy } from '../../platforms/huya/playerHelper';
import { getBilibiliStreamConfig, startBilibiliDanmakuListener, stopBilibiliDanmaku } from '../../platforms/bilibili/playerHelper';

import StreamerInfo from '../StreamerInfo/index.vue';
import DanmuList from '../DanmuList/index.vue';
import { platform } from '@tauri-apps/plugin-os';

import { invoke } from '@tauri-apps/api/core';
import { useImageProxy } from '../FollowsList/useProxy';

// Ensure image proxy helpers are available in this component
const { ensureProxyStarted, proxify } = useImageProxy();

class RefreshControl extends Plugin {
  static override pluginName = 'refreshControl';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_LEFT,
    index: 2,
    disable: false,
    onClick: null as (() => void) | null,
  };

  private handleClick: ((event: Event) => void) | null = null;
  private isLoading = false;

  override afterCreate() {
    if (this.config.disable) {
      return;
    }
    this.handleClick = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      if (this.isLoading) {
        return;
      }
      if (typeof this.config.onClick === 'function') {
        this.config.onClick();
      }
    };
    this.bind(['click', 'touchend'], this.handleClick);
  }

  override destroy() {
    if (this.handleClick) {
      this.unbind(['click', 'touchend'], this.handleClick);
      this.handleClick = null;
    }
    this.setLoading(false);
  }

  override render() {
    if (this.config.disable) {
      return '';
    }
    return `<xg-icon class="xgplayer-refresh-control" title="刷新">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4.5 12a7.5 7.5 0 0 1 12.36-5.36L18.5 8" />
        <path d="M19 4v5h-5" />
        <path d="M19.5 12a7.5 7.5 0 0 1-12.36 5.36L5.5 16" />
        <path d="M5 20v-5h5" />
      </svg>
    </xg-icon>`;
  }

  setLoading(isLoading: boolean) {
    this.isLoading = isLoading;
    const root = this.root as HTMLElement | null;
    if (!root) {
      return;
    }
    root.classList.toggle('is-loading', isLoading);
    if (isLoading) {
      root.setAttribute('aria-disabled', 'true');
    } else {
      root.removeAttribute('aria-disabled');
    }
  }
}

class QualityControl extends Plugin {
  static override pluginName = 'qualityControl';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 5,
    disable: false,
    options: [] as string[],
    getCurrent: (() => '') as () => string,
    onSelect: (async (_value: string) => {}) as (value: string) => Promise<void> | void,
  };

  private dropdown: HTMLElement | null = null;
  private handleToggle: ((event: Event) => void) | null = null;
  private handleDocumentClick: ((event: MouseEvent) => void) | null = null;
  private isSwitching = false;

  override afterCreate() {
    if (this.config.disable) {
      return;
    }

    this.createDropdown();
    this.updateLabel(this.getCurrent());

    this.handleToggle = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      if (this.isSwitching) {
        return;
      }
      this.toggleDropdown();
    };
    this.bind(['click', 'touchend'], this.handleToggle);

    if (typeof document !== 'undefined') {
      this.handleDocumentClick = (event: MouseEvent) => {
        if (!this.root.contains(event.target as Node)) {
          this.hideDropdown();
        }
      };
      document.addEventListener('click', this.handleDocumentClick);
    }
  }

  override destroy() {
    if (this.handleToggle) {
      this.unbind(['click', 'touchend'], this.handleToggle);
      this.handleToggle = null;
    }
    if (this.handleDocumentClick) {
      document.removeEventListener('click', this.handleDocumentClick);
      this.handleDocumentClick = null;
    }
    if (this.dropdown) {
      this.dropdown.remove();
      this.dropdown = null;
    }
    this.setSwitching(false);
  }

  override render() {
    if (this.config.disable) {
      return '';
    }
    const current = this.getCurrent();
    return `<xg-icon class="xgplayer-quality-control" title="画质">
      <span class="quality-label">${current}</span>
      <svg class="quality-caret" width="10" height="10" viewBox="0 0 10 10" fill="none">
        <path d="M2.5 3.5L5 6l2.5-2.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </xg-icon>`;
  }

  updateLabel(label: string) {
    const textEl = this.find('.quality-label') as HTMLElement | null;
    if (textEl) {
      textEl.textContent = label;
    }
    this.updateActiveState(label);
  }

  setOptions(options: string[]) {
    this.config.options = options;
    this.populateDropdown();
  }

  private getCurrent() {
    return typeof this.config.getCurrent === 'function' ? this.config.getCurrent() : '';
  }

  private createDropdown() {
    this.dropdown = document.createElement('div');
    this.dropdown.className = 'xgplayer-quality-dropdown';
    this.root.appendChild(this.dropdown);
    this.populateDropdown();
  }

  private populateDropdown() {
    if (!this.dropdown) {
      return;
    }
    this.dropdown.innerHTML = '';
    const options: string[] = Array.isArray(this.config.options) ? this.config.options : [];
    options.forEach((option) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'xgplayer-quality-item';
      btn.innerHTML = `
        <span class="quality-name">${option}</span>
        <svg class="quality-check" width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M3 6.5l2 2 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      `;
      btn.addEventListener('click', (event) => {
        event.stopPropagation();
        event.preventDefault();
        if (this.isSwitching) {
          return;
        }
        let actionResult: Promise<void> | void;
        try {
          const callback = this.config.onSelect;
          actionResult = typeof callback === 'function' ? callback(option) : undefined;
        } catch (error) {
          console.error('[QualityControl] onSelect error:', error);
          actionResult = undefined;
        }
        Promise.resolve(actionResult).finally(() => {
          this.hideDropdown();
          this.updateLabel(this.getCurrent());
        });
      });
      this.dropdown!.appendChild(btn);
    });
    this.updateActiveState(this.getCurrent());
    this.applySwitchingState();
  }

  private toggleDropdown() {
    if (this.isSwitching) {
      return;
    }
    if (!this.dropdown) {
      return;
    }
    const isOpen = this.dropdown.classList.toggle('show');
    if (isOpen) {
      this.updateActiveState(this.getCurrent());
    }
    this.root.classList.toggle('menu-open', isOpen);
  }

  private hideDropdown() {
    if (this.dropdown) {
      this.dropdown.classList.remove('show');
    }
    this.root.classList.remove('menu-open');
  }

  private updateActiveState(current: string) {
    if (!this.dropdown) {
      return;
    }
    const items = this.dropdown.querySelectorAll<HTMLButtonElement>('.xgplayer-quality-item');
    items.forEach((item) => {
      const label = item.querySelector('.quality-name')?.textContent?.trim();
      item.classList.toggle('active', label === current);
    });
  }

  setSwitching(isSwitching: boolean) {
    this.isSwitching = isSwitching;
    this.applySwitchingState();
    if (isSwitching) {
      this.hideDropdown();
    }
  }

  private applySwitchingState() {
    const root = this.root as HTMLElement | null;
    if (root) {
      root.classList.toggle('is-switching', this.isSwitching);
    }
    if (this.dropdown) {
      this.dropdown.classList.toggle('disabled', this.isSwitching);
      const buttons = this.dropdown.querySelectorAll<HTMLButtonElement>('.xgplayer-quality-item');
      buttons.forEach((button) => {
        button.disabled = this.isSwitching;
      });
    }
  }
}

const props = defineProps<{
  roomId: string | null;
  platform: StreamingPlatform;
  isFollowed?: boolean;
  streamUrl?: string | null; // Primarily for Douyin direct URL
  title?: string | null;
  anchorName?: string | null;
  avatar?: string | null;
  isLive?: boolean | null;
  initialError?: string | null; // Added to accept pre-determined errors like "主播未开播"
  cookie?: string | null; // Optional cookie for platforms like Bilibili
}>();

const emit = defineEmits<{
  (e: 'follow', streamer: any): void;
  (e: 'unfollow', roomId: string): void;
  (e: 'close-player'): void;
  (e: 'fullscreen-change', isFullscreen: boolean): void;
  (e: 'request-refresh-details'): void;
  (e: 'request-player-reload'): void;
}>();

const playerContainerRef = ref<HTMLDivElement | null>(null);
const playerInstance = shallowRef<Player | null>(null);
const refreshControlPlugin = shallowRef<RefreshControl | null>(null);
const qualityControlPlugin = shallowRef<QualityControl | null>(null);
const danmuInstance = shallowRef<DanmuOverlayInstance | null>(null);
const danmakuMessages = ref<DanmakuMessage[]>([]);
const isDanmakuListenerActive = ref(false); // Tracks if a danmaku listener is supposed to be running
let unlistenDanmakuFn: (() => void) | null = null;

const isLoadingStream = ref(true);
const streamError = ref<string | null>(null);
const isOfflineError = ref(false); // Added to track '主播未开播' state

// Reactive state for streamer info, initialized by props, potentially updated by internal fetches (for Douyin)
const playerTitle = ref(props.title);
const playerAnchorName = ref(props.anchorName);
const playerAvatar = ref(props.avatar);
const playerIsLive = ref(props.isLive);

const isInNativePlayerFullscreen = ref(false); // New: Tracks Artplayer element's native fullscreen
const isInWebFullscreen = ref(false);
const isFullScreen = ref(false); // True if EITHER native player OR web fullscreen is active

// OS specific states
const osName = ref<string>('');

// 画质切换相关
const qualityOptions = ['原画', '高清', '标清'] as const;

const resolveStoredQuality = (platform?: StreamingPlatform | null): string => {
  if (!platform) {
    return '原画';
  }
  if (typeof window === 'undefined') {
    return '原画';
  }
  try {
    const saved = window.localStorage.getItem(`${platform}_preferred_quality`);
    if (saved && qualityOptions.includes(saved as (typeof qualityOptions)[number])) {
      return saved;
    }
  } catch (error) {
    console.warn('[Player] Failed to read stored quality preference:', error);
  }
  return '原画';
};

const currentQuality = ref<string>(resolveStoredQuality(props.platform));
const isQualitySwitching = ref(false);
const isRefreshingStream = ref(false);

function resetFullscreenState() {
  isInNativePlayerFullscreen.value = false;
  isInWebFullscreen.value = false;
  isFullScreen.value = false;
  try {
    document.documentElement.classList.remove('web-fs-active');
  } catch (error) {
    console.warn('[Player] Failed to reset web fullscreen flag:', error);
  }
}

function updateFullscreenFlag() {
  isFullScreen.value = isInNativePlayerFullscreen.value || isInWebFullscreen.value;
  emit('fullscreen-change', isFullScreen.value);
}

function destroyPlayerInstance() {
  const player = playerInstance.value;
  if (player) {
    try {
      player.destroy();
    } catch (error) {
      console.error('[Player] Error destroying xgplayer instance:', error);
    }
    const overlayHost = player.root?.querySelector('.player-danmu-overlay') as HTMLElement | null;
    overlayHost?.remove();
  }
  playerInstance.value = null;

  const danmu = danmuInstance.value;
  if (danmu) {
    try {
      danmu.stop?.();
    } catch (error) {
      console.error('[Player] Error stopping danmu overlay:', error);
    }
    danmuInstance.value = null;
  }

  refreshControlPlugin.value = null;
  qualityControlPlugin.value = null;

  resetFullscreenState();
}

function ensureDanmuOverlayHost(player: Player): HTMLElement | null {
  const root = player.root as HTMLElement | undefined;
  if (!root) {
    return null;
  }

  let host = root.querySelector('.player-danmu-overlay') as HTMLElement | null;
  if (!host) {
    host = document.createElement('div');
    host.className = 'player-danmu-overlay';
  }

  const videoContainer = root.querySelector('xg-video-container');
  if (videoContainer && host.parentElement !== videoContainer) {
    videoContainer.appendChild(host);
  } else if (!videoContainer && host.parentElement !== root) {
    root.appendChild(host);
  } else if (!host.parentElement) {
    root.appendChild(host);
  }

  return host;
}

function createDanmuOverlay(player: Player | null) {
  if (!player) {
    return null;
  }

  const overlayHost = ensureDanmuOverlayHost(player);
  if (!overlayHost) {
    return null;
  }

  overlayHost.innerHTML = '';

  try {
    const overlay = new DanmuJs({
      container: overlayHost,
      player: player.video || player.media || undefined,
      comments: [],
      mouseControl: false,
      defaultOff: false,
      channelSize: 36,
      containerStyle: {
        pointerEvents: 'none',
      },
    });

    overlay.start?.();
    danmuInstance.value = overlay;
    return overlay;
  } catch (error) {
    console.error('[Player] Failed to initialize danmu.js overlay:', error);
    danmuInstance.value = null;
    return null;
  }
}

async function mountXgPlayer(
  streamUrl: string,
  platformCode: StreamingPlatform,
  roomId: string,
) {
  await nextTick();

  if (!playerContainerRef.value) {
    streamError.value = '播放器容器初始化失败。';
    return;
  }

  playerContainerRef.value.innerHTML = '';

  const player = new Player({
    el: playerContainerRef.value,
    url: streamUrl,
    isLive: true,
    autoplay: true,
    playsinline: true,
    lang: 'zh-cn',
    width: '100%',
    height: '100%',
    videoFillMode: 'contain',
    closeVideoClick: false,
    keyShortcut: true,
    volume: {
      position: POSITIONS.CONTROLS_LEFT,
      index: 3,
    },
    pip: {
      position: POSITIONS.CONTROLS_RIGHT,
      index: 3,
      showIcon: true,
    },
    cssFullscreen: {
      index: 2,
    },
    playbackRate: false,
    controls: {
      mode: 'normal',
    },
    plugins: [FlvPlugin],
    flv: {
      isLive: true,
      cors: true,
      autoCleanupSourceBuffer: true,
      enableWorker: true,
      stashInitialSize: 128,
      lazyLoad: true,
      lazyLoadMaxDuration: 30,
      deferLoadAfterSourceOpen: true,
    },
  });

  playerInstance.value = player;

  refreshControlPlugin.value = player.registerPlugin(RefreshControl, {
    position: POSITIONS.CONTROLS_LEFT,
    index: 2,
    onClick: () => {
      void reloadCurrentStream('refresh');
    },
  }) as RefreshControl;

  qualityControlPlugin.value = player.registerPlugin(QualityControl, {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 5,
    options: [...qualityOptions],
    getCurrent: () => currentQuality.value,
    onSelect: async (option: string) => {
      if (option === currentQuality.value) {
        return;
      }
      await switchQuality(option);
    },
  }) as QualityControl;
  qualityControlPlugin.value?.setOptions([...qualityOptions]);
  qualityControlPlugin.value?.updateLabel(currentQuality.value);

  let overlayInstance = createDanmuOverlay(player);

  player.on('ready', async () => {
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    try {
      if (roomId) {
        await startCurrentDanmakuListener(platformCode, roomId, overlayInstance);
      }
    } catch (error) {
      console.error('[Player] Failed starting danmaku listener after ready:', error);
    }
    overlayInstance?.play?.();
    updateFullscreenFlag();
  });

  player.on('play', () => {
    overlayInstance?.play?.();
  });

  player.on('pause', () => {
    overlayInstance?.pause?.();
  });

  player.on('destroy', () => {
    overlayInstance?.stop?.();
    overlayInstance = null;
  });

  player.on('error', (error: any) => {
    console.error('[Player] xgplayer error:', error);
    streamError.value = `播放器错误: ${error?.message || error}`;
  });

  player.on('enterFullscreen', () => {
    isInNativePlayerFullscreen.value = true;
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    overlayInstance?.play?.();
    updateFullscreenFlag();
  });

  player.on('exitFullscreen', () => {
    isInNativePlayerFullscreen.value = false;
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    updateFullscreenFlag();
  });

  player.on('enterFullscreenWeb', () => {
    isInWebFullscreen.value = true;
    try {
      document.documentElement.classList.add('web-fs-active');
    } catch (error) {
      console.warn('[Player] Failed to set web fullscreen flag:', error);
    }
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    overlayInstance?.play?.();
    updateFullscreenFlag();
  });

  player.on('exitFullscreenWeb', () => {
    isInWebFullscreen.value = false;
    try {
      document.documentElement.classList.remove('web-fs-active');
    } catch (error) {
      console.warn('[Player] Failed to clear web fullscreen flag:', error);
    }
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    updateFullscreenFlag();
  });
}


async function initializePlayerAndStream(
  pRoomId: string, 
  pPlatform: StreamingPlatform,
  _pStreamUrlProp?: string | null, 
  isRefresh: boolean = false,
  oldRoomIdForCleanup?: string | null,
  oldPlatformForCleanup?: StreamingPlatform | null
) {
  isLoadingStream.value = true;
  streamError.value = null;
  isOfflineError.value = false;

  // Detect OS (synchronous call)
  osName.value = platform();

  if (!isRefresh) {
    danmakuMessages.value = [];
  }

  if (props.initialError && props.initialError.includes('主播未开播')) {
    streamError.value = props.initialError;
    isOfflineError.value = true;
    playerTitle.value = props.title;
    playerAnchorName.value = props.anchorName;
    playerAvatar.value = props.avatar;
    playerIsLive.value = false;
    destroyPlayerInstance();
    isLoadingStream.value = false;
    return;
  }

  if (oldRoomIdForCleanup && oldPlatformForCleanup !== undefined && oldPlatformForCleanup !== null) {
    await stopCurrentDanmakuListener(oldPlatformForCleanup, oldRoomIdForCleanup);
    if (oldPlatformForCleanup === StreamingPlatform.DOUYU) {
      await stopDouyuProxy();
    }
    if (oldPlatformForCleanup === StreamingPlatform.HUYA) {
      await stopHuyaProxy();
    }
  } else {
    await stopCurrentDanmakuListener();
  }

  destroyPlayerInstance();

  try {
    let streamConfig: { streamUrl: string; streamType: string | undefined };

    if (pPlatform === StreamingPlatform.DOUYU) {
      if (playerIsLive.value === false) {
        streamError.value = streamError.value || '主播未开播。';
        isOfflineError.value = true;
        isLoadingStream.value = false;
        return;
      }
      streamConfig = await getDouyuStreamConfig(pRoomId, currentQuality.value);
    } else if (pPlatform === StreamingPlatform.DOUYIN) {
      const douyinConfig = await fetchAndPrepareDouyinStreamConfig(pRoomId, currentQuality.value);
      playerTitle.value = douyinConfig.title;
      playerAnchorName.value = douyinConfig.anchorName;
      playerAvatar.value = douyinConfig.avatar;
      playerIsLive.value = douyinConfig.isLive;

      if (douyinConfig.initialError || !douyinConfig.isLive || !douyinConfig.streamUrl) {
        streamError.value = douyinConfig.initialError || '主播未开播或无法获取直播流。';
        isOfflineError.value = true;
        playerIsLive.value = false;
        isLoadingStream.value = false;
        console.warn(`[Player] Douyin config error or not live: ${streamError.value}`);
        return;
      }

      streamConfig = { streamUrl: douyinConfig.streamUrl, streamType: douyinConfig.streamType };
    } else if (pPlatform === StreamingPlatform.HUYA) {
      streamConfig = await getHuyaStreamConfig(pRoomId, currentQuality.value);
    } else if (pPlatform === StreamingPlatform.BILIBILI) {
      streamConfig = await getBilibiliStreamConfig(pRoomId, currentQuality.value, props.cookie || undefined);
    } else {
      throw new Error(`不支持的平台: ${pPlatform}`);
    }

    isLoadingStream.value = false;
    await mountXgPlayer(streamConfig.streamUrl, pPlatform, pRoomId);
  } catch (error: any) {
    console.error(`[Player] Error initializing stream for ${pPlatform} room ${pRoomId}:`, error);
    destroyPlayerInstance();

    const errorMessage = error?.message || '加载直播流失败，请稍后再试。';

    if (errorMessage.includes('主播未开播')) {
      streamError.value = errorMessage;
      isOfflineError.value = true;

      try {
        if (pPlatform === StreamingPlatform.HUYA) {
          const result: any = await invoke('get_huya_unified_cmd', { roomId: pRoomId, quality: currentQuality.value });
          await ensureProxyStarted();
          playerTitle.value = result?.title ?? props.title;
          playerAnchorName.value = result?.nick ?? props.anchorName;
          playerAvatar.value = proxify((result?.avatar ?? props.avatar ?? '') as string);
        } else if (pPlatform === StreamingPlatform.BILIBILI) {
          const payload = { args: { room_id_str: pRoomId } };
          const savedCookie = (typeof localStorage !== 'undefined') ? (localStorage.getItem('bilibili_cookie') || null) : null;
          const res: any = await invoke('fetch_bilibili_streamer_info', { payload, cookie: savedCookie });
          await ensureProxyStarted();
          playerTitle.value = res?.title ?? props.title;
          playerAnchorName.value = res?.anchor_name ?? props.anchorName;
          playerAvatar.value = proxify((res?.avatar ?? props.avatar ?? '') as string);
        }
      } catch (infoError) {
        console.warn('[Player] Failed to fetch basic streamer info for offline page:', infoError);
      }
    } else {
      streamError.value = errorMessage;
      isOfflineError.value = false;
    }

    isLoadingStream.value = false;
  }
}
async function startCurrentDanmakuListener(platform: StreamingPlatform, roomId: string, danmuOverlay: DanmuOverlayInstance | null) {
  if (!roomId) {
    return;
  }
  if (isDanmakuListenerActive.value) {
    return;
  }

  isDanmakuListenerActive.value = true;
  if (!danmuOverlay) {
    console.warn('[Player] Danmu overlay instance missing, incoming danmaku will not render on video but list will update.');
  }

  try {
    let stopFn: (() => void) | null = null;
    if (platform === StreamingPlatform.DOUYU) {
      stopFn = await startDouyuDanmakuListener(roomId, danmuOverlay, danmakuMessages); 
    } else if (platform === StreamingPlatform.DOUYIN) {
      stopFn = await startDouyinDanmakuListener(roomId, danmuOverlay, danmakuMessages);
    } else if (platform === StreamingPlatform.HUYA) {
      stopFn = await startHuyaDanmakuListener(roomId, danmuOverlay, danmakuMessages);
    } else if (platform === StreamingPlatform.BILIBILI) {
      stopFn = await startBilibiliDanmakuListener(roomId, danmuOverlay, danmakuMessages, props.cookie || undefined);
    }

    if (stopFn) {
      unlistenDanmakuFn = stopFn;
      const successMessage: DanmakuMessage = {
        id: `system-conn-${Date.now()}`,
        nickname: '系统消息',
        content: '弹幕连接成功！',
        isSystem: true,
        type: 'success',
        color: '#28a745'
      };
      danmakuMessages.value.push(successMessage);

    } else {
      console.warn(`[Player] Danmaku listener for ${platform}/${roomId} did not return a stop function.`);
      isDanmakuListenerActive.value = false; 
    }
  } catch (error) {
    console.error(`[Player] Failed to start danmaku listener for ${platform}/${roomId}:`, error);
    isDanmakuListenerActive.value = false; 

    const errorMessage: DanmakuMessage = {
      id: `system-err-${Date.now()}`,
      nickname: '系统消息',
      content: '弹幕连接失败，请尝试刷新播放器。',
      isSystem: true,
      type: 'error',
      color: '#dc3545'
    };
    danmakuMessages.value.push(errorMessage);
  }
}

async function stopCurrentDanmakuListener(platform?: StreamingPlatform, roomId?: string | null | undefined) {
  if (platform) {
    if (platform === StreamingPlatform.DOUYU) {
      await stopDouyuDanmaku(roomId!, unlistenDanmakuFn); 
    } else if (platform === StreamingPlatform.DOUYIN) {
      await stopDouyinDanmaku(unlistenDanmakuFn);
    } else if (platform === StreamingPlatform.HUYA) {
      await stopHuyaDanmaku(unlistenDanmakuFn);
    } else if (platform === StreamingPlatform.BILIBILI) {
      await stopBilibiliDanmaku(unlistenDanmakuFn);
    }
    if (unlistenDanmakuFn) { 
        unlistenDanmakuFn = null;
    }

  } else if (unlistenDanmakuFn) {
    console.warn('[Player] stopCurrentDanmakuListener called without platform, but a global unlistenDanmakuFn exists. Calling it now.');
    try {
      unlistenDanmakuFn();
      unlistenDanmakuFn = null; // Nullify after successful call
    } catch (e) {
      console.error('[Player] Error executing fallback unlistenDanmakuFn:', e);
      // Still nullify to prevent repeated errors with a bad function ref
      unlistenDanmakuFn = null; 
    }
  }

  isDanmakuListenerActive.value = false;
}

const retryInitialization = async () => {
  await reloadCurrentStream('refresh');
};

// 画质切换函数
const switchQuality = async (quality: string) => {
  if (isQualitySwitching.value) {
    return;
  }
  if (!qualityOptions.includes(quality as (typeof qualityOptions)[number])) {
    return;
  }
  if (!props.roomId || props.platform == null) {
    emit('request-player-reload');
    return;
  }
  if (quality === currentQuality.value) {
    return;
  }

  isQualitySwitching.value = true;
  const previousQuality = currentQuality.value;

  try {
    currentQuality.value = quality;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(`${props.platform}_preferred_quality`, quality);
    }
    await reloadCurrentStream('quality');
    console.log(`[Player] 画质切换完成: ${quality}`);
  } catch (error) {
    console.error('[Player] 画质切换失败:', error);
    currentQuality.value = previousQuality;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(`${props.platform}_preferred_quality`, previousQuality);
    }
  } finally {
    isQualitySwitching.value = false;
  }
};

// 初始化画质偏好
const initializeQualityPreference = () => {
  currentQuality.value = resolveStoredQuality(props.platform);
};

async function reloadCurrentStream(trigger: 'refresh' | 'quality' = 'refresh') {
  if (isLoadingStream.value) {
    return;
  }
  if (!props.roomId || props.platform == null) {
    emit('request-player-reload');
    return;
  }
  const isRefreshAction = trigger === 'refresh';
  if (isRefreshAction) {
    isRefreshingStream.value = true;
  }
  try {
    await initializePlayerAndStream(
      props.roomId,
      props.platform,
      props.streamUrl ?? null,
      true,
      props.roomId,
      props.platform,
    );
  } finally {
    if (isRefreshAction) {
      isRefreshingStream.value = false;
    }
  }
  if (trigger === 'quality') {
    qualityControlPlugin.value?.updateLabel(currentQuality.value);
  }
}

watch(isRefreshingStream, (isLoading) => {
  refreshControlPlugin.value?.setLoading(isLoading);
});

watch(refreshControlPlugin, (plugin) => {
  plugin?.setLoading(isRefreshingStream.value);
});

watch(isQualitySwitching, (isSwitching) => {
  qualityControlPlugin.value?.setSwitching(isSwitching);
});

watch(qualityControlPlugin, (plugin) => {
  plugin?.setSwitching(isQualitySwitching.value);
});

watch(currentQuality, (quality) => {
  qualityControlPlugin.value?.updateLabel(quality);
});

watch([() => props.roomId, () => props.platform, () => props.streamUrl, () => props.avatar, () => props.title, () => props.anchorName, () => props.isLive], 
  async ([newRoomId, newPlatform, newStreamUrl, _newAvatar, _newTitle, _newAnchorName, _newIsLive], [oldRoomId, oldPlatform, _oldStreamUrl, _oldAvatar, _oldTitle, _oldAnchorName, _oldIsLive]) => {
    // Update internal reactive streamer info when props change
    if (newPlatform === StreamingPlatform.DOUYU) { // Only update from props if Douyu
      playerTitle.value = _newTitle;
      playerAnchorName.value = _newAnchorName;
      playerAvatar.value = _newAvatar;
      if (_newIsLive !== undefined) {
          playerIsLive.value = _newIsLive;
      }
    }

    // Initial error from props (e.g., DouyuPlayerView determined offline before MainPlayer rendered)
    if (newRoomId && newPlatform) {
      // Always reset isOfflineError when props change significantly unless initialError prop dictates it
      if (!(props.initialError && props.initialError.includes('主播未开播'))) {
        isOfflineError.value = false; 
      }

      // Determine if re-initialization is needed
      const isInitialCall = oldRoomId === undefined && oldPlatform === undefined;
      const hasSwitchedStream = newRoomId !== oldRoomId || newPlatform !== oldPlatform;
      // Douyin might also re-init if its specific stream URL prop changes (though less likely with current proxy setup)
      const douyinStreamUrlChanged = newPlatform === StreamingPlatform.DOUYIN && newStreamUrl !== _oldStreamUrl;

      const needsReInit = hasSwitchedStream || isInitialCall || douyinStreamUrlChanged;

      if (needsReInit) {
        // 在重新初始化时更新画质偏好
        initializeQualityPreference();
        // Pass oldRoomId and oldPlatform (which might be undefined on initial call)
        // initializePlayerAndStream will handle undefined cleanup IDs gracefully.
        initializePlayerAndStream(newRoomId, newPlatform, newStreamUrl, false, oldRoomId, oldPlatform);
      }
    } else if (!newRoomId) { 
      if (oldRoomId && oldPlatform !== null && oldPlatform !== undefined) { 
        await stopCurrentDanmakuListener(oldPlatform, oldRoomId);
        if (oldPlatform === StreamingPlatform.DOUYU) {
          await stopDouyuProxy();
        }
        if (oldPlatform === StreamingPlatform.HUYA) {
          await stopHuyaProxy();
        }
      } else {
        await stopCurrentDanmakuListener();
      }

      destroyPlayerInstance();

      isLoadingStream.value = false;
      danmakuMessages.value = [];
      streamError.value = null;
      isOfflineError.value = false; 
    }
    if (!props.roomId || props.platform == null) {
      if (props.initialError) {
        if (props.initialError.includes('主播未开播')) {
            streamError.value = props.initialError;
            isOfflineError.value = true;
        } else {
            streamError.value = props.initialError;
            isOfflineError.value = false; // Ensure it's not marked as offline for other errors
        }
      }
      isLoadingStream.value = false;
    }
}, 
{ immediate: true }
);

onMounted(async () => {
  // 初始化画质偏好
  initializeQualityPreference();
  
  if (!props.roomId || props.platform == null) {
    if (props.initialError) {
      if (props.initialError.includes('主播未开播')) {
          streamError.value = props.initialError;
          isOfflineError.value = true;
      } else {
          streamError.value = props.initialError;
          isOfflineError.value = false; // Ensure it's not marked as offline for other errors
      }
    }
    isLoadingStream.value = false;
  }
});

onUnmounted(async () => {
  const platformToStop: StreamingPlatform = props.platform;
  const roomIdToStop: string | null = props.roomId;
  await stopCurrentDanmakuListener(platformToStop, roomIdToStop);

  if (props.platform === StreamingPlatform.DOUYU) {
    await stopDouyuProxy();
  }
  if (props.platform === StreamingPlatform.HUYA) {
    await stopHuyaProxy();
  }

  destroyPlayerInstance();
  danmakuMessages.value = []; 
});

</script>
