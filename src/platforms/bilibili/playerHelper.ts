import { invoke } from '@tauri-apps/api/core';
import type { LiveStreamInfo } from '../common/types';

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