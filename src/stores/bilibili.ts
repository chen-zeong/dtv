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
      // 防抖：避免重复初始化；但如果之前失败过，允许重试
      if (this.initialized && this.wWebId) return
      try {
        const id = await invoke<string>('generate_bilibili_w_webid')
        this.wWebId = id
        this.initialized = true
        this.error = null
      } catch (e: any) {
        // 保持错误信息，但不要阻塞之后的调用；后端已做兜底自动生成
        this.error = typeof e === 'string' ? e : (e?.message || '初始化 B 站 w_webid 失败')
        // 不设置 initialized=true，允许后续视图或列表请求触发后端兜底
        this.initialized = false
      }
    },
  },
})