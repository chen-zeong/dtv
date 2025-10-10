import { invoke } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';
import Artplayer from 'artplayer';
import { Ref } from 'vue';
import type { DanmakuMessage } from '../../components/player/types';
import { v4 as uuidv4 } from 'uuid';

export interface HuyaUnifiedEntry { quality: string; bitRate: number; url: string; }

export async function getHuyaStreamConfig(roomId: string, quality: string = '原画'): Promise<{ streamUrl: string, streamType: string | undefined }> {
  console.log('[HuyaPlayerHelper] getHuyaStreamConfig called with roomId:', roomId, 'quality:', quality);
  try {
    const result = await invoke<any>('get_huya_unified_cmd', { roomId: roomId, quality });
    console.log('[HuyaPlayerHelper] getHuyaStreamConfig got result:', result);
    
    if (result && result.flv_tx_urls && Array.isArray(result.flv_tx_urls)) {  
      const streamUrl = pickHuyaUrlByQuality(result.flv_tx_urls, quality) || result.flv_tx_urls[0]?.url;
      if (streamUrl) {
        const proxy = await startHuyaProxyFromUrl(streamUrl);
        return proxy;
      } else {
        // 无地址按未开播处理
        throw new Error('主播未开播或无法获取直播流');
      }
    } else {
      // 数据异常或为空，一般意味着未开播或房间详情获取失败
      throw new Error('主播未开播或获取虎牙房间详情失败');
    }
  } catch (error: any) {
    console.error('[HuyaPlayerHelper] getHuyaStreamConfig error:', error);
    // 若后端明确返回未开播文案，直接透传；否则统一按未开播处理
    const msg = (error?.message || '').trim();
    if (msg.includes('未开播')) {
      throw new Error(msg);
    }
    throw new Error('主播未开播或无法获取直播流');
  }
}

export async function startHuyaProxyFromUrl(directFlvUrl: string): Promise<{ streamUrl: string, streamType: string | undefined }> {
  try {
    // 先将原始流地址写入到后端的 StreamUrlStore
    await invoke('set_stream_url_cmd', { url: directFlvUrl });
    // 再启动代理，返回带有 /live.flv 的本地地址
    const localProxyUrl = await invoke<string>('start_proxy');
    console.log('[HuyaPlayerHelper] Proxy started for Huya:', localProxyUrl, 'from', directFlvUrl);
    return { streamUrl: localProxyUrl, streamType: 'flv' };
  } catch (error) {
    console.error('[HuyaPlayerHelper] Error starting Huya proxy:', error);
    throw error;
  }
}

export async function stopHuyaProxy(): Promise<void> {
  try {
    await invoke('stop_proxy');
    console.log('[HuyaPlayerHelper] Proxy stopped successfully');
  } catch (error) {
    console.error('[HuyaPlayerHelper] Error stopping proxy:', error);
    throw error;
  }
}

// 统一的 Rust 弹幕事件负载（与 Douyin/Douyu 保持一致）
interface UnifiedRustDanmakuPayload {
  room_id: string;
  user: string;
  content: string;
  user_level: number;
  fans_club_level: number;
}
let currentHuyaRoomId: string | null = null;

export async function startHuyaDanmakuListener(
  roomId: string,
  artInstance: Artplayer,
  danmakuMessagesRef: Ref<DanmakuMessage[]>
): Promise<() => void> {
  console.log('[HuyaPlayerHelper] Starting Huya danmaku listener for room:', roomId);
  currentHuyaRoomId = roomId;
  
  try {
    // 调用后端虎牙弹幕监听命令
    await invoke('start_huya_danmaku_listener', { payload: { args: { room_id_str: roomId } } });
    console.log('[HuyaPlayerHelper] Backend Huya danmaku listener started');
  } catch (error) {
    console.error('[HuyaPlayerHelper] Failed to start backend Huya danmaku listener:', error);
    throw error;
  }

  // 监听弹幕事件
  const eventName = 'danmaku-message';
  
  const unlisten = await listen<UnifiedRustDanmakuPayload>(eventName, (event: TauriEvent<UnifiedRustDanmakuPayload>) => {
    console.log('[HuyaPlayerHelper] Received danmaku event:', event.payload);
    
    // 只处理当前房间的弹幕（后端 payload 字段为 room_id/user/content/...）
    if (!event.payload || event.payload.room_id !== roomId) {
      return;
    }

    const frontendDanmaku: DanmakuMessage = {
      id: uuidv4(),
      nickname: event.payload.user || '未知用户',
      content: event.payload.content,
      level: String(event.payload.user_level ?? 0),
      badgeLevel: event.payload.fans_club_level != null ? String(event.payload.fans_club_level) : undefined,
      room_id: roomId,
    };

    // 添加到 Artplayer 弹幕插件
    if (artInstance && (artInstance as any).plugins && (artInstance as any).plugins.artplayerPluginDanmuku) {
      (artInstance as any).plugins.artplayerPluginDanmuku.emit({ 
        text: frontendDanmaku.content, 
        color: (frontendDanmaku as any).color || '#FFFFFF' 
      });
    }

    // 添加到弹幕消息列表
    danmakuMessagesRef.value.push(frontendDanmaku);
    if (danmakuMessagesRef.value.length > 200) {
      danmakuMessagesRef.value.splice(0, danmakuMessagesRef.value.length - 200);
    }
  });

  console.log('[HuyaPlayerHelper] Event listener registered for:', eventName);
  
  return unlisten;
}

export async function stopHuyaDanmaku(currentUnlistenFn: (() => void) | null): Promise<void> {
  if (currentUnlistenFn) {
    try { 
      currentUnlistenFn(); 
      console.log('[HuyaPlayerHelper] Event listener unregistered');
    } catch (e) { 
      console.warn('[HuyaPlayerHelper] stopHuyaDanmaku cleanup error:', e); 
    }
  }
  
  // 停止后端虎牙弹幕监听
  try {
    const roomIdToStop = currentHuyaRoomId || '';
    await invoke('stop_huya_danmaku_listener', { roomId: roomIdToStop });
  } catch (e) {
    console.warn('[HuyaPlayerHelper] stopHuyaDanmaku: backend stop encountered error (ignored):', e);
  }
  currentHuyaRoomId = null;
  console.log('[HuyaPlayerHelper] Huya danmaku stopped');
}

function pickHuyaUrlByQuality(entries: HuyaUnifiedEntry[], quality: string): string | undefined {
  const target = entries.find((e) => e.quality === quality);
  return target?.url;
}