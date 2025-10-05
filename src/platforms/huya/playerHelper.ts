import { invoke } from '@tauri-apps/api/core';

export interface HuyaUnifiedEntry { quality: string; bitRate: number; url: string; }

export async function getHuyaStreamConfig(roomId: string, quality: string = '原画'): Promise<{ streamUrl: string, streamType: string | undefined }> {
  try {
    console.log(`[HuyaPlayerHelper] getHuyaStreamConfig start: roomId=${roomId}, quality=${quality}`);
    const res: any = await invoke('get_huya_unified_cmd', { roomId, quality });
    console.log('[HuyaPlayerHelper] get_huya_unified_cmd result:', res);
    const selectedUrl: string | undefined = res?.selected_url || undefined;
    let finalUrl = selectedUrl;
    if (!finalUrl) {
      const entries: HuyaUnifiedEntry[] = Array.isArray(res?.flv_tx_urls) ? res.flv_tx_urls : [];
      finalUrl = pickHuyaUrlByQuality(entries, quality) || entries[0]?.url;
    }
    if (!finalUrl) {
      throw new Error('虎牙直播流地址获取为空。');
    }
    await invoke('set_stream_url_cmd', { url: finalUrl });
    console.log('[HuyaPlayerHelper] set_stream_url_cmd set with url');
    const proxyUrl = await invoke<string>('start_proxy');
    console.log('[HuyaPlayerHelper] start_proxy returned proxy url:', proxyUrl);
    return { streamUrl: proxyUrl, streamType: 'flv' };
  } catch (e: any) {
    const msg = e?.message || String(e);
    console.error('[HuyaPlayerHelper] getHuyaStreamConfig error:', e);
    throw new Error(`获取虎牙直播流失败: ${msg}`);
  }
}

export async function startHuyaProxyFromUrl(directFlvUrl: string): Promise<string> {
  if (!directFlvUrl) throw new Error('无效的虎牙 FLV URL');
  console.log('[HuyaPlayerHelper] startHuyaProxyFromUrl with:', directFlvUrl);
  await invoke('set_stream_url_cmd', { url: directFlvUrl });
  const proxyUrl = await invoke<string>('start_proxy');
  console.log('[HuyaPlayerHelper] start_proxy returned url:', proxyUrl);
  return proxyUrl;
}

export interface HuyaTxStreamEntry { quality: string; bitRate: number; url: string; }

export function pickHuyaUrlByQuality(entries: HuyaTxStreamEntry[], quality: string): string | null {
  if (!Array.isArray(entries) || entries.length === 0) return null;
  // Prefer exact match by quality name, fallback to highest bitRate
  const byName = entries.find(e => e.quality === quality);
  if (byName) return byName.url;
  const sorted = [...entries].sort((a, b) => (b.bitRate || 0) - (a.bitRate || 0));
  return sorted[0]?.url || null;
}

export async function stopHuyaProxy(): Promise<void> {
  try {
    console.log('[HuyaPlayerHelper] stopHuyaProxy invoked');
    await invoke('stop_proxy');
    console.log('[HuyaPlayerHelper] stopHuyaProxy completed');
  } catch (e) {
    console.error('[HuyaPlayerHelper] Error stopping proxy server:', e);
  }
}