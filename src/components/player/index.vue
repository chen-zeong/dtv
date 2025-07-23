<template>
  <div class="player-page">
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
            <h3>😴 {{ streamError }}</h3>
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
import { ref, onMounted, watch, onUnmounted, shallowRef, nextTick, computed } from 'vue';
import Artplayer from 'artplayer';
import artplayerPluginDanmuku from 'artplayer-plugin-danmuku';

import './player.css';

import { Platform as StreamingPlatform } from '../../platforms/common/types';
import type { DanmakuMessage } from './types';

// Platform-specific player helpers
import { getDouyuStreamConfig, startDouyuDanmakuListener, stopDouyuDanmaku, stopDouyuProxy } from '../../platforms/douyu/playerHelper';
import { fetchAndPrepareDouyinStreamConfig, startDouyinDanmakuListener, stopDouyinDanmaku } from '../../platforms/douyin/playerHelper';

import StreamerInfo from '../StreamerInfo/index.vue';
import DanmuList from '../DanmuList/index.vue';

// OS detection
import { platform } from '@tauri-apps/plugin-os';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'; // Import WebviewWindow

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
const art = shallowRef<Artplayer | null>(null);
const flvPlayerInstance = shallowRef<any>(null);
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
const isMacOS = computed(() => osName.value === 'macos');

async function initializePlayerAndStream(
  pRoomId: string, 
  pPlatform: StreamingPlatform,
  _pStreamUrlProp?: string | null, 
  isRefresh: boolean = false,
  oldRoomIdForCleanup?: string | null,
  oldPlatformForCleanup?: StreamingPlatform | null // Use renamed Platform
) {
  isLoadingStream.value = true;
  streamError.value = null;
  isOfflineError.value = false;

  // Detect OS (synchronous call)
  osName.value = platform();

  if (!isRefresh) {
    danmakuMessages.value = [];
  }

  // Handle initialError from props (e.g., Douyin pre-check says "主播未开播")
  if (props.initialError && props.initialError.includes('主播未开播')) {
    streamError.value = props.initialError;
    isOfflineError.value = true;
    // Update reactive player info based on props if offline state is from props
    playerTitle.value = props.title;
    playerAnchorName.value = props.anchorName;
    playerAvatar.value = props.avatar;
    playerIsLive.value = false; // Explicitly set to false if error indicates offline

    isLoadingStream.value = false;
    return; // Skip further initialization if streamer is known to be offline
  }

  if (art.value) {
    // Stop danmaku for the *old* room if IDs are valid
    if (oldRoomIdForCleanup && oldPlatformForCleanup !== undefined && oldPlatformForCleanup !== null) {
        await stopCurrentDanmakuListener(oldPlatformForCleanup, oldRoomIdForCleanup);
    } else {
        console.warn("[Player] Cleanup in init: Old room/platform for danmaku stop not provided or invalid.");
    }
    
    // Stop Douyu proxy if the *old* platform was Douyu
    if (oldPlatformForCleanup === StreamingPlatform.DOUYU) {
        await stopDouyuProxy();
    }

    // Attempt to unload media from Artplayer before destroying it
    if (art.value.playing) {
      art.value.pause();
    }
    try {
      art.value.url = ''; 
    } catch (e) {
      console.error('[Player] Error setting old Artplayer URL to empty during cleanup:', e);
    }

    art.value.destroy(true);
    art.value = null;
    await nextTick();
  }

  try {
    let streamConfig: { streamUrl: string, streamType: string | undefined };

    if (pPlatform === StreamingPlatform.DOUYU) {
      if (playerIsLive.value === false) { // Explicitly check for false, as null/undefined might mean info not yet loaded
        streamError.value = streamError.value || '主播未开播。'; // Preserve specific error if already set by initialError prop
        isOfflineError.value = true;
        isLoadingStream.value = false;
        return; // Stop further execution for stream fetching and player init
      }
      streamConfig = await getDouyuStreamConfig(pRoomId);

    } else if (pPlatform === StreamingPlatform.DOUYIN) {
      const douyinConfig = await fetchAndPrepareDouyinStreamConfig(pRoomId);
      
      // Update internal reactive state with fetched Douyin info
      playerTitle.value = douyinConfig.title;
      playerAnchorName.value = douyinConfig.anchorName;
      playerAvatar.value = douyinConfig.avatar;
      playerIsLive.value = douyinConfig.isLive;
      
      if (douyinConfig.initialError || !douyinConfig.isLive || !douyinConfig.streamUrl) {
        streamError.value = douyinConfig.initialError || '主播未开播或无法获取直播流。';
        isOfflineError.value = true; // Assume offline or error state
        isLoadingStream.value = false;
        // Ensure playerIsLive is false if there's an error making it unplayable
        playerIsLive.value = false; 
        console.warn(`[Player] Douyin config error or not live: ${streamError.value}`);
        return; // Stop if not playable
      }
      streamConfig = { streamUrl: douyinConfig.streamUrl, streamType: douyinConfig.streamType };
    } else {
      throw new Error(`不支持的平台: ${pPlatform}`);
    }
    
    isLoadingStream.value = false;
    await nextTick();

    if (!playerContainerRef.value) {
      console.error('[Player] playerContainerRef is null AFTER nextTick. Cannot init Artplayer. This is unexpected.');
      streamError.value = '播放器容器初始化失败。';
      return;
    }

    // 播放器类型将直接使用 streamConfig.streamType，如果未定义则默认为 'flv'
    const artPlayerOptions: any = {
        container: playerContainerRef.value, 
        url: streamConfig.streamUrl,
        type: streamConfig.streamType || 'flv',
        isLive: true, pip: true, autoplay: true, autoSize: false, aspectRatio: false,
        fullscreen: true, // Player element native fullscreen - THIS IS THE KEY FOR OS FULLSCREEN ON NON-MAC
        fullscreenWeb: true, // Player web fullscreen (takes over viewport)
        miniProgressBar: true, mutex: true,
        backdrop: false, playsInline: true, autoPlayback: true, theme: '#FB7299', lang: 'zh-cn',
        moreVideoAttr: { playsInline: true },
        plugins: [
            artplayerPluginDanmuku({
            danmuku: [], speed: 7, opacity: 1, fontSize: 20, color: '#FFFFFF',
            mode: 0, margin: [10, '2%'], antiOverlap: true, synchronousPlayback: false, emitter:false
            }),
        ],
        controls: [
          {
            name: 'streamRefresh', 
            position: 'left',     
            index: 15,
            html: '<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M17.65 6.35A7.958 7.958 0 0 0 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0 1 12 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"></path></svg>',
            tooltip: '刷新',
            click: async () => {
              emit('request-player-reload');
            }
          }
        ],
        customType: {
            flv: function(video: HTMLVideoElement, url: string) {
                const platformForLog = pPlatform; 
                import('mpegts.js').then(mpegts => {
                    if (mpegts.default.isSupported()) {
                        // 如果之前有实例，再次检查并销毁 (双重保险，主要销毁点在 Artplayer 销毁前)
                        if (flvPlayerInstance.value) {
                            try {
                                flvPlayerInstance.value.destroy();
                            } catch (e) {
                                console.error(`[Player ${platformForLog}] Error destroying previous mpegts.js player in customType.flv:`, e);
                            }
                            flvPlayerInstance.value = null;
                        }
                        const flvPlayer = mpegts.default.createPlayer(
                            { type: 'flv', url: url, isLive: true, cors: true, hasAudio: true, hasVideo: true }, 
                            {}
                        );
                        flvPlayerInstance.value = flvPlayer; // <--- 保存新实例
                        flvPlayer.attachMediaElement(video);
                        flvPlayer.load();
                        video.play().catch(e => console.error(`[Player ${platformForLog}] FLV Auto-play error:`, e));
                        flvPlayer.on('error', (errType, errInfo) => {
                            console.error(`[mpegts ${platformForLog}] Error:`, errType, errInfo);
                            streamError.value = `FLV组件错误: ${errInfo.msg}`;
                        });
                    } else {
                        console.error(`[Player ${platformForLog}] Browser does not support FLV playback (mpegts.js).`);
                        streamError.value = '浏览器不支持FLV播放。';
                    }
                }).catch((e) => { 
                    console.error(`[Player ${platformForLog}] Failed to load mpegts.js component:`, e);
                    streamError.value = '加载FLV播放组件失败。'; 
                });
            }
        },
    };
    art.value = new Artplayer(artPlayerOptions);

    art.value.on('ready', async () => {
      if (pRoomId && pPlatform && art.value) { 
        await startCurrentDanmakuListener(pPlatform, pRoomId, art.value);
      }
    });
    art.value.on('error', (error: any, _reconnectTime: number) => { 
        console.error('[Player] Artplayer error:', error);
        streamError.value = `播放器错误: ${error.message || error}`; 
    });

    // Listener for Artplayer's NATIVE element fullscreen changes
    art.value.on('fullscreen', async (nativeActive: boolean) => {
      isInNativePlayerFullscreen.value = nativeActive;
      isFullScreen.value = isInNativePlayerFullscreen.value || isInWebFullscreen.value;

      if (!isMacOS.value) {
        try {
          await WebviewWindow.getCurrent().setFullscreen(nativeActive);
        } catch (e) {
          console.error('Tauri setFullscreen error triggered by player native fullscreen:', e);
        }
      }
      emit('fullscreen-change', isFullScreen.value);
    });

    // Listener for Artplayer's WEB fullscreen changes
    art.value.on('fullscreenWeb', (webActive: boolean) => {
      isInWebFullscreen.value = webActive;
      isFullScreen.value = isInNativePlayerFullscreen.value || isInWebFullscreen.value;
      // No OS fullscreen call here, this is just for player's web fullscreen state.
      emit('fullscreen-change', isFullScreen.value);
    });

  } catch (error: any) {
    console.error(`[Player] Error initializing stream for ${pPlatform} room ${pRoomId}:`, error);
    const errorMessage = error.message || '加载直播流失败，请稍后再试。';
    if (errorMessage.includes('主播未开播')) {
      streamError.value = errorMessage; // Store the specific "主播未开播" message
      isOfflineError.value = true;       // Set the flag for custom display
    } else {
      streamError.value = errorMessage;
      isOfflineError.value = false;
    }
    isLoadingStream.value = false;
  }
}

async function startCurrentDanmakuListener(platform: StreamingPlatform, roomId: string, artInstance: Artplayer | null) {
  if (!roomId) {
    return;
  }
  if (!artInstance) {
    return;
  }
  if (isDanmakuListenerActive.value) {
    return;
  }

  isDanmakuListenerActive.value = true;

  try {
    let stopFn: (() => void) | null = null;
    if (platform === StreamingPlatform.DOUYU) {
      stopFn = await startDouyuDanmakuListener(roomId, artInstance, danmakuMessages); 
    } else if (platform === StreamingPlatform.DOUYIN) {
      stopFn = await startDouyinDanmakuListener(roomId, artInstance, danmakuMessages);
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

const retryInitialization = () => {
  emit('request-player-reload');
};

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
        // Pass oldRoomId and oldPlatform (which might be undefined on initial call)
        // initializePlayerAndStream will handle undefined cleanup IDs gracefully.
        initializePlayerAndStream(newRoomId, newPlatform, newStreamUrl, false, oldRoomId, oldPlatform);
      }
    } else if (!newRoomId && art.value) { 
      // This block handles clearing the player when roomId becomes null (e.g. navigating away from player)
      // It correctly uses oldRoomId and oldPlatform for cleanup as these are from the watcher.
      if (oldRoomId && oldPlatform !== null && oldPlatform !== undefined) { 
          await stopCurrentDanmakuListener(oldPlatform, oldRoomId);
          if (oldPlatform === StreamingPlatform.DOUYU) {
              await stopDouyuProxy();
          }
      }
      
      // Player instance (art and flv) destruction is now handled by onUnmounted.
      // We only reset component state here.

      isLoadingStream.value = false;
      danmakuMessages.value = [];
      streamError.value = null;
      isOfflineError.value = false; 
    }
    if (!props.roomId || !props.platform) {
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
  if (!props.roomId || !props.platform) {
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

  if (art.value) {
    if (art.value.playing) {
      art.value.pause();
    }
    
    // Attempt to unload media from Artplayer before flv instance and artplayer itself are destroyed
    try {
      art.value.url = ''; 
    } catch (e) {
      console.error('[Player] Error setting Artplayer URL to empty on unmount:', e);
    }

    // More aggressively stop and destroy mpegts.js instance if it exists
    if (flvPlayerInstance.value) {
        try {
            if (typeof flvPlayerInstance.value.unload === 'function') {
                flvPlayerInstance.value.unload();
            }
            if (typeof flvPlayerInstance.value.detachMediaElement === 'function') {
                flvPlayerInstance.value.detachMediaElement();
            }
            flvPlayerInstance.value.destroy();
        } catch (e) {
            console.error('[Player] Error destroying mpegts.js player on unmount:', e);
        }
        flvPlayerInstance.value = null;
    }

    try {
      art.value.destroy(true); // true to remove video element and all listeners
    } catch (e) {
      console.error('[Player] Error destroying Artplayer instance on unmount:', e);
    }
    art.value = null;
  } else { // art.value was already null
      if (flvPlayerInstance.value) { // But flv instance might exist
          try { flvPlayerInstance.value.destroy(); } catch (e) { console.error('[Player] Error destroying orphaned mpegts.js player on unmount:', e); }
          flvPlayerInstance.value = null;
      }
  }
  danmakuMessages.value = []; 
});

</script>