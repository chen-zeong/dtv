import { invoke } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';
import Artplayer from 'artplayer';
import type { LiveStreamInfo } from '../common/types';
import type { Ref } from 'vue';
import type { DanmakuMessage } from '../../components/player/types';

export async function getBilibiliStreamConfig(
  roomId: string,
  quality: string = '原画',
  cookie?: string,
): Promise<{ streamUrl: string, streamType: string | undefined }> {
  if (!roomId) {
    throw new Error('房间ID未提供');
  }
  const payloadData = { args: { room_id_str: roomId } };
  const result = await invoke<LiveStreamInfo>('get_bilibili_live_stream_url_with_quality', {
    payload: payloadData,
    quality,
    cookie: cookie || null,
  });
  if (result.error_message) {
    throw new Error(result.error_message);
  }
  if (!result.stream_url) {
    throw new Error('未获取到直播流地址');
  }
  return { streamUrl: result.stream_url, streamType: 'flv' };
}

// 统一的 Rust 弹幕事件负载（与 Douyin/Douyu/Huya 保持一致）
interface UnifiedRustDanmakuPayload {
  room_id: string;
  user: string;
  content: string;
  user_level: number;
  fans_club_level: number;
}

export async function startBilibiliDanmakuListener(
  roomId: string,
  artInstance: Artplayer,
  danmakuMessagesRef: Ref<DanmakuMessage[]>,
  cookie?: string,
): Promise<() => void> {
  // 启动后端 B 站弹幕监听（cookie 可选）
  await invoke('start_bilibili_danmaku_listener', {
    payload: { args: { room_id_str: roomId } },
    cookie: cookie || null,
  });

  const eventName = 'danmaku-message';
  const unlisten = await listen<UnifiedRustDanmakuPayload>(eventName, (event: TauriEvent<UnifiedRustDanmakuPayload>) => {
    if (!event.payload || event.payload.room_id !== roomId) return;

    const frontendDanmaku: DanmakuMessage = {
      nickname: event.payload.user || '未知用户',
      content: event.payload.content,
      level: String(event.payload.user_level ?? 0),
      badgeLevel: event.payload.fans_club_level != null ? String(event.payload.fans_club_level) : undefined,
      room_id: roomId,
    };

    if (artInstance && (artInstance as any).plugins && (artInstance as any).plugins.artplayerPluginDanmuku) {
      (artInstance as any).plugins.artplayerPluginDanmuku.emit({ 
        text: frontendDanmaku.content, 
        color: (frontendDanmaku as any).color || '#FFFFFF' 
      });
    }

    danmakuMessagesRef.value.push(frontendDanmaku);
    if (danmakuMessagesRef.value.length > 200) {
      danmakuMessagesRef.value.splice(0, danmakuMessagesRef.value.length - 200);
    }
  });
  return unlisten;
}

export async function stopBilibiliDanmaku(currentUnlistenFn: (() => void) | null): Promise<void> {
  if (currentUnlistenFn) {
    try { currentUnlistenFn(); } catch {}
  }
  try {
    await invoke('stop_bilibili_danmaku_listener');
  } catch {}
}