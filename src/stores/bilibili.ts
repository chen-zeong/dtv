import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useBilibiliStore = defineStore('bilibili', {
  state: () => ({
    initialized: false as boolean,
    error: null as string | null,
    wWebId: null as string | null,
  }),
  actions: {
    async initWebid() {
      if (this.initialized) return
      try {
        const id = await invoke<string>('generate_bilibili_w_webid')
        this.wWebId = id
        this.initialized = true
        this.error = null
      } catch (e: any) {
        this.error = typeof e === 'string' ? e : (e?.message || '初始化 B 站 w_webid 失败')
        this.initialized = false
      }
    },
  },
})