import { invoke } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';
import Artplayer from 'artplayer';
import { Ref } from 'vue';
import { Platform } from '../common/types';
import type { DanmakuMessage } from '../../components/player/types'; // Corrected path
import { fetchStreamPlaybackDetails } from '../common/apiService';
import { parseDouyuDanmakuMessage } from './parsers'; // <-- Import the parser

// Specific type for Douyu's raw danmaku payload from Rust event
export interface DouyuRustDanmakuPayload {
  type: "chatmsg" | "uenter";
  room_id: string;
  nickname: string;
  content: string; // only for chatmsg
  level: string;
  badgeName?: string;
  badgeLevel?: string;
  color?: string;
  uid?: string; // only for uenter
}

export async function getDouyuStreamConfig(roomId: string): Promise<{ streamUrl: string, streamType: string | undefined }> {
  let finalStreamUrl: string | null = null;
  let streamType: string | undefined = undefined;
  const MAX_STREAM_FETCH_ATTEMPTS = 1; // Changed to 1 attempt

  for (let attempt = 1; attempt <= MAX_STREAM_FETCH_ATTEMPTS; attempt++) {
    try {
      const playbackDetails = await fetchStreamPlaybackDetails(roomId, Platform.DOUYU);
      if (playbackDetails && playbackDetails.primaryUrl) {
        finalStreamUrl = playbackDetails.primaryUrl;
        streamType = playbackDetails.format; // 直接使用后端返回的 format
        if (streamType === 'm3u8') {
            console.warn('[DouyuPlayerHelper] Received m3u8 format, but expected flv. Overriding to flv.');
            streamType = 'flv';
        }
        break; 
      } else {
        // This case might be redundant if fetchStreamPlaybackDetails throws an error for empty/null URLs
        throw new Error('斗鱼直播流地址获取为空。');
      }
    } catch (e: any) {
      console.error(`[DouyuPlayerHelper] 获取斗鱼直播流失败 (尝试 ${attempt}/${MAX_STREAM_FETCH_ATTEMPTS}):`, e.message);
      // Check for specific error messages indicating streamer is offline or room doesn't exist
      // These are examples; actual messages from your Rust backend might differ.
      const offlineOrInvalidRoomMessages = [
        "主播未开播", // Generic offline message
        "房间不存在", // Generic room not found
        "error: 1",   // Example: Douyu API error code 1 (often offline or invalid)
        "error: 102", // Example: Douyu API error code 102 (often room not found or offline)
        "error code 1", // More flexible matching for error codes
        "error code 102"
        // Add other known patterns here from your Rust error messages
      ];

      const errorMessageLowerCase = e.message?.toLowerCase() || '';
      const isDefinitivelyOffline = offlineOrInvalidRoomMessages.some(msg => errorMessageLowerCase.includes(msg.toLowerCase()));

      if (isDefinitivelyOffline) {
        console.warn(`[DouyuPlayerHelper] Streamer for room ${roomId} is definitively offline or room is invalid. Aborting retries.`);
        throw e; // Re-throw the specific error to stop retries
      }

      if (attempt === MAX_STREAM_FETCH_ATTEMPTS) {
        throw new Error(`获取斗鱼直播流失败 (尝试 ${MAX_STREAM_FETCH_ATTEMPTS} 次后): ${e.message}`);
      }
      // Exponential backoff might be better, but simple delay for now
      await new Promise(resolve => setTimeout(resolve, 1000 * attempt)); 
    }
  }

  if (!finalStreamUrl) {
    throw new Error('未能获取有效的斗鱼直播流地址。');
  }

  try {
    // Assuming stopProxy is handled separately or before calling this if needed
    await invoke('set_stream_url_cmd', { url: finalStreamUrl });
    const proxyUrl = await invoke<string>('start_proxy')
    return { streamUrl: proxyUrl, streamType };
  } catch (e: any) {
    throw new Error(`设置斗鱼代理失败: ${e.message}`);
  }
}

export async function startDouyuDanmakuListener(
  roomId: string,
  artInstance: Artplayer, // For emitting danmaku to player
  danmakuMessagesRef: Ref<DanmakuMessage[]> // For updating DanmuList
): Promise<() => void> {

  await invoke('start_danmaku_listener', { roomId });
  
  const eventName = `danmaku-${roomId}`;

  const unlisten = await listen<DouyuRustDanmakuPayload>(eventName, (event: TauriEvent<DouyuRustDanmakuPayload>) => {

    if (artInstance && artInstance.plugins && artInstance.plugins.artplayerPluginDanmuku && event.payload) {

      const commonDanmaku = parseDouyuDanmakuMessage(event.payload);
      
      if (commonDanmaku) {

        artInstance.plugins.artplayerPluginDanmuku.emit({
          text: commonDanmaku.content, // Use content from parsed message
          color: commonDanmaku.color || '#FFFFFF', // Use color from parsed message
        });

       
        const displayDanmaku: DanmakuMessage = { // Adapting to DanmuList's expected DanmakuMessage type
            nickname: commonDanmaku.sender.nickname,
            content: commonDanmaku.content,
            level: commonDanmaku.sender.level ? String(commonDanmaku.sender.level) : '0',
            badgeName: commonDanmaku.sender.badgeName,
            badgeLevel: commonDanmaku.sender.badgeLevel ? String(commonDanmaku.sender.badgeLevel) : undefined,
            color: commonDanmaku.color,
            uid: commonDanmaku.sender.uid,
            room_id: roomId, // roomId is available in this scope
            // id and timestamp are part of CommonDanmakuMessage but might not be directly used by DanmuList's item display
        };
        danmakuMessagesRef.value.push(displayDanmaku);
        
        if (danmakuMessagesRef.value.length > 200) { // Manage danmaku array size
            danmakuMessagesRef.value.splice(0, danmakuMessagesRef.value.length - 200);
        }
      }
    }
  });
  
  return unlisten;
}

export async function stopDouyuDanmaku(roomId: string, currentUnlistenFn: (() => void) | null): Promise<void> {
  if (currentUnlistenFn) {
    currentUnlistenFn();
  }
  try {
    if (roomId) { 
        await invoke('stop_danmaku_listener', { roomId: roomId });
    }
  } catch (error) {
    console.error('[DouyuPlayerHelper] Error invoking stop_danmaku_listener for Douyu:', error);
  }
}

export async function stopDouyuProxy(): Promise<void> {
  try {
    await invoke('stop_proxy');
  } catch (e) {
    console.error('[DouyuPlayerHelper] Error stopping proxy server:', e);
  }
} 