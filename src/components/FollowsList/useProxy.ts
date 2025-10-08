import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useImageProxy() {
  const proxyBase = ref('')

  async function ensureProxyStarted(): Promise<void> {
    try {
      if (!proxyBase.value) {
        const base = await invoke<string>('start_static_proxy_server')
        proxyBase.value = base || ''
      }
    } catch (e) {
      console.warn('[useImageProxy] ensureProxyStarted failed:', e)
    }
  }

  function proxify(url: string | null | undefined): string {
    const u = (url || '').trim()
    if (!u) return ''
    try {
      const parsed = new URL(u)
      if (parsed.hostname === '127.0.0.1' || parsed.hostname === 'localhost') {
        return u
      }
    } catch {}
    if (!proxyBase.value) return u
    const base = proxyBase.value.endsWith('/') ? proxyBase.value.slice(0, -1) : proxyBase.value
    return `${base}/image?url=${encodeURIComponent(u)}`
  }

  function getAvatarSrc(platform: string, avatarUrl?: string | null) {
    const u = avatarUrl || ''
    if (!u) return ''
    // Platform enum string names: expect 'BILIBILI'
    if (platform === 'BILIBILI') {
      return proxify(u)
    }
    return u
  }

  return { proxyBase, ensureProxyStarted, proxify, getAvatarSrc }
}