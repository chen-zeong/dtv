import { invoke } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';
import Artplayer from 'artplayer';
import type { LiveStreamInfo, StreamVariant } from '../common/types';
import type { Ref } from 'vue';
import type { DanmakuMessage } from '../../components/player/types';
import { v4 as uuidv4 } from 'uuid';

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

  // 调试输出：真实上游地址与所有可用地址
  if (result.upstream_url) {
    console.info('[Bilibili] 上游真实地址（可用于 VLC 测试）:', result.upstream_url);
  }
  if (result.available_streams && Array.isArray(result.available_streams)) {
    console.info(`[Bilibili] 可用播放地址（共 ${result.available_streams.length} 条）:`);
    (result.available_streams as StreamVariant[]).forEach((v, idx) => {
      const meta = [v.format, v.desc, v.qn?.toString(), v.protocol].filter(Boolean).join(' | ');
      console.info(`  [${idx + 1}] ${v.url}${meta ? `  <<< ${meta}` : ''}`);
    });
  }

  // 根据 available_streams 决定播放类型：优先 m3u8，其次 flv
  let streamType: string | undefined = undefined;
  if (Array.isArray(result.available_streams)) {
    const hasM3U8 = (result.available_streams as StreamVariant[]).some(v => (v.format || '').toLowerCase() === 'ts' || v.url.includes('.m3u8'));
    const hasFlv = (result.available_streams as StreamVariant[]).some(v => (v.format || '').toLowerCase() === 'flv' || v.url.endsWith('.flv'));
    if (hasM3U8) streamType = 'm3u8';
    else if (hasFlv) streamType = 'flv';
  }

  return { streamUrl: result.stream_url, streamType };
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
      id: uuidv4(),
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