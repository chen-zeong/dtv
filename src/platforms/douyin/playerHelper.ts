import { invoke } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';
import Artplayer from 'artplayer';
import { Ref } from 'vue';
import { Platform } from '../common/types';
import type { DanmakuMessage, RustGetStreamUrlPayload } from '../../components/player/types';
import type { LiveStreamInfo } from '../common/types';


export interface DouyinRustDanmakuPayload {
  room_id?: string; 
  user: string;      // Nickname from Rust's DanmakuFrontendPayload
  content: string;
  user_level: number; // from Rust's i64
  fans_club_level: number; // from Rust's i32
}

export async function fetchAndPrepareDouyinStreamConfig(roomId: string): Promise<{ 
  streamUrl: string | null;
  streamType: string | undefined; 
  title?: string | null; 
  anchorName?: string | null; 
  avatar?: string | null; 
  isLive: boolean; 
  initialError: string | null; // Made non-optional, will always be string or null
}> {
  if (!roomId) {
    return { streamUrl: null, streamType: undefined, title: null, anchorName: null, avatar: null, isLive: false, initialError: '房间ID未提供' };
  }

  try {
    const payloadData = { args: { room_id_str: roomId } };
    const result = await invoke<LiveStreamInfo>('get_douyin_live_stream_url', { payload: payloadData });

    if (result.error_message) {
      console.error(`[DouyinPlayerHelper] Error from backend for room ${roomId}: ${result.error_message}`);
      return {
        streamUrl: null,
        streamType: undefined,
        title: result.title,
        anchorName: result.anchor_name,
        avatar: result.avatar,
        isLive: result.status === 2,
        initialError: result.error_message, // string | null from Rust
      };
    }

    const streamAvailable = result.status === 2 && !!result.stream_url;
    let streamType: string | undefined = undefined;
    let uiMessage: string | null = null; 

    if (streamAvailable && result.stream_url) {
      if (result.stream_url.startsWith('http://127.0.0.1') && result.stream_url.endsWith('/live.flv')) {
        streamType = 'flv';
      } else if (result.stream_url.includes('pull-hls') || result.stream_url.endsWith('.m3u8')) {
        console.warn(`[DouyinPlayerHelper] Received HLS-like stream URL (${result.stream_url}), but expected flv. Overriding to flv.`);
        streamType = 'flv';
      } else if (result.stream_url.includes('pull-flv') || result.stream_url.includes('.flv')) {
        streamType = 'flv';
      } else {
        console.warn(`[DouyinPlayerHelper] Could not determine stream type for URL: ${result.stream_url}. Defaulting to flv.`);
        streamType = 'flv';
      }
      // uiMessage remains null if stream is available and no prior error.
    } else {
      if (result.status !== 2) {
        uiMessage = result.title ? `主播 ${result.anchor_name || ''} 未开播。` : '主播未开播或房间不存在。';
      } else {
        uiMessage = '主播在线，但获取直播流失败。';
      }
    }

    return {
      streamUrl: streamAvailable ? (result.stream_url !== undefined ? result.stream_url : null) : null,
      streamType: streamType,
      title: result.title,
      anchorName: result.anchor_name,
      avatar: result.avatar,
      isLive: streamAvailable,
      initialError: uiMessage, // uiMessage is definitely string or null here.
    };

  } catch (e: any) {
    console.error(`[DouyinPlayerHelper] Exception while fetching Douyin stream details for ${roomId}:`, e);
    return { 
        streamUrl: null, 
        streamType: undefined, 
        title: null, 
        anchorName: null, 
        avatar: null, 
        isLive: false, 
        initialError: e.message || '获取直播信息失败: 未知错误' // Ensure string here
    };
  }
}

export async function startDouyinDanmakuListener(
  roomId: string,
  artInstance: Artplayer, // For emitting danmaku to player
  danmakuMessagesRef: Ref<DanmakuMessage[]> // For updating DanmuList
): Promise<() => void> {
  
  const rustPayload: RustGetStreamUrlPayload = { 
    args: { room_id_str: roomId }, 
    platform: Platform.DOUYIN, 
  };
  await invoke('start_douyin_danmu_listener', { payload: rustPayload });
  
  const eventName = 'danmaku-message';

  const unlisten = await listen<DouyinRustDanmakuPayload>(eventName, (event: TauriEvent<DouyinRustDanmakuPayload>) => {
    

    if (artInstance && artInstance.plugins && artInstance.plugins.artplayerPluginDanmuku && event.payload) {
      const rustP = event.payload;
      const frontendDanmaku: DanmakuMessage = {
        nickname: rustP.user || '未知用户',
        content: rustP.content || '',
        level: String(rustP.user_level || 0),
        badgeLevel: rustP.fans_club_level > 0 ? String(rustP.fans_club_level) : undefined,
        room_id: rustP.room_id || roomId, // Ensure room_id is present
      };

      artInstance.plugins.artplayerPluginDanmuku.emit({
        text: frontendDanmaku.content,
        color: frontendDanmaku.color || '#FFFFFF', 
      });
      danmakuMessagesRef.value.push(frontendDanmaku);
      if (danmakuMessagesRef.value.length > 200) { // Manage danmaku array size
        danmakuMessagesRef.value.splice(0, danmakuMessagesRef.value.length - 200);
      }
    }
  });
  return unlisten;
}

export async function stopDouyinDanmaku(currentUnlistenFn: (() => void) | null): Promise<void> {
  if (currentUnlistenFn) {
    currentUnlistenFn();
  }
  try {
    const rustPayload: RustGetStreamUrlPayload = { 
      args: { room_id_str: "stop_listening" }, 
      platform: Platform.DOUYIN, 
    };
    await invoke('start_douyin_danmu_listener', { payload: rustPayload });
  } catch (error) {
    console.error('[DouyinPlayerHelper] Error stopping Douyin danmaku listener:', error);
  }
} 