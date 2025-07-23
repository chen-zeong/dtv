import { invoke } from '@tauri-apps/api/core';
import type { FollowedStreamer, LiveStreamInfo, LiveStatus } from '../common/types';

export async function refreshDouyinFollowedStreamer(
  streamer: FollowedStreamer
): Promise<Partial<FollowedStreamer>> {
  try {
    // The payload for 'get_douyin_live_stream_url' expects { payload: { args: { room_id_str: string } } }
    const payloadData = { args: { room_id_str: streamer.id } };
    const data = await invoke<LiveStreamInfo>('fetch_douyin_streamer_info', {
      payload: payloadData,
    });

    // Check if data is valid and there are no errors from the backend
    if (data && !data.error_message) {
      const isLive = data.status === 2; // Correctly determines if live
      const liveStatus: LiveStatus = isLive ? 'LIVE' : 'OFFLINE'; // Set liveStatus based on isLive

      return {
        isLive: isLive, 
        liveStatus: liveStatus, // Add/Update liveStatus field
        nickname: data.anchor_name || streamer.nickname, // Map anchor_name to nickname
        roomTitle: data.title || streamer.roomTitle,      // Map title to roomTitle
        avatarUrl: data.avatar || streamer.avatarUrl,    // Map avatar to avatarUrl
      };
    } else {
      if (data && data.error_message) {
        console.warn(
          `[DouyinFollowHelper] Error fetching Douyin room ${streamer.id}: ${data.error_message}`
        );
      } else {
        console.warn(
          `[DouyinFollowHelper] Received no/invalid data for Douyin room ${streamer.id}`,
          data
        );
      }
      return { isLive: false, liveStatus: 'OFFLINE' }; // Ensure these are set on error too
    }
  } catch (e) {
    console.error(
      `[DouyinFollowHelper] Failed to refresh Douyin streamer ${streamer.id}:`,
      e
    );
    return { isLive: false, liveStatus: 'OFFLINE' }; // Ensure these are set on error too
  }
} 