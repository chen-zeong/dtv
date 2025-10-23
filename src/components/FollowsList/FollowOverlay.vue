<template>
  <teleport to="body">
    <transition name="overlay-fade">
      <div v-if="show" class="follow-overlay-backdrop" @click.self="emit('close')">
        <transition name="overlay-pop">
          <div 
            class="follow-overlay-panel" 
            :style="{ top: `${panelTop}px`, left: `${(alignLeft ?? 240)}px`, height: `${panelHeight}px` }"
          >
            <!-- 将关闭按钮移动到面板右上角 -->
            <button class="overlay-close-btn" title="关闭" @click="emit('close')">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
            <div class="overlay-header" ref="headerRef">
              <div class="overlay-header-left">
                <!-- 移除标题：关注列表 -->
                <slot name="filters"></slot>
              </div>
              <div class="overlay-header-actions">
                <button 
                  class="overlay-text-btn manage-action" 
                  :class="{ active: props.isDeleteMode }"
                  @click="emit('toggle-remove')"
                >
                  <span>{{ props.isDeleteMode ? '完成' : '管理' }}</span>
                </button>
                <button 
                  class="overlay-text-btn refresh-action" 
                  :class="{ 'is-refreshing': isRefreshing, 'just-finished': justFinished }" 
                  :disabled="isRefreshing" 
                  @click="emit('refresh')"
                >
                  <span class="refresh-label">刷新</span>
                  <span class="refresh-spinner" aria-hidden="true"></span>
                </button>
                <!-- 原关闭按钮已移除到面板右上角 -->
              </div>
            </div>
            <div class="overlay-content" :style="{ height: `${Math.max(120, panelHeight - headerHeight)}px`, overflow: shouldScroll ? 'auto' : 'hidden' }">
              <div v-if="!items || items.length === 0" class="overlay-empty">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="feather feather-heart">
                  <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"></path>
                </svg>
                <p class="empty-title">暂无关注主播</p>
                <p class="empty-text">当前筛选下暂无关注主播</p>
              </div>
              <ul v-else class="overlay-streamers-list" ref="listEl">
                <li 
                  v-for="s in items" 
                  :key="s.id" 
                  class="overlay-streamer-item"
                  :class="{ 'remove-mode': props.isDeleteMode }"
                  @click="handleSelect(s)"
                >
                  <button 
                    v-if="props.isDeleteMode" 
                    class="overlay-remove-btn" 
                    title="删除"
                    @click.stop="emit('remove', s)"
                  >
                    ×
                  </button>
                  <StreamerItem 
                    :streamer="s" 
                    :getAvatarSrc="getAvatarSrc" 
                    :handleImgError="handleImgError"
                    :getLiveIndicatorClass="getLiveIndicatorClass"
                    :proxyBase="proxyBase"
                    :big="false"
                    :showPlatform="false"
                    @clickItem="() => handleSelect(s)"
                  />
                </li>
              </ul>
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import type { FollowedStreamer } from '../../platforms/common/types';
import StreamerItem from './StreamerItem.vue';

const props = defineProps<{ 
  show: boolean, 
  items: FollowedStreamer[], 
  getAvatarSrc: (s: FollowedStreamer) => string, 
  handleImgError: (ev: Event, s: FollowedStreamer) => void, 
  getLiveIndicatorClass: (s: FollowedStreamer) => string, 
  proxyBase?: string, 
  alignTop?: number, 
  alignLeft?: number,
  isRefreshing?: boolean,
  isDeleteMode?: boolean
}>();
const emit = defineEmits<{ 
  (e: 'close'): void, 
  (e: 'select', s: FollowedStreamer): void, 
  (e: 'refresh'): void,
  (e: 'toggle-remove'): void,
  (e: 'remove', s: FollowedStreamer): void
}>();

// 刷新完成提示：当 isRefreshing 从 true 变为 false 时，短暂展示完成动画
const justFinished = ref(false);
watch(() => props.isRefreshing, (newVal, oldVal) => {
  if (oldVal && !newVal) {
    justFinished.value = true;
    setTimeout(() => { justFinished.value = false; }, 800);
  }
});

// 参考值与动态测量
const PANEL_MIN = 220;
const PANEL_MAX_MARGIN = 120; // 留出顶部/底部边距
const DEFAULT_CARD_H = 76; // 估算：48头像 + 24内边距 + 2边框
const LIST_PAD_TOP = 6;
const LIST_PAD_BOTTOM = 6;
const panelHeight = ref<number>(400);
const panelTop = ref<number>(64);
const headerHeight = ref<number>(56);
const headerRef = ref<HTMLElement | null>(null);
const listEl = ref<HTMLElement | null>(null);
const shouldScroll = ref<boolean>(false);

function clamp(n: number, min: number, max: number) { return Math.max(min, Math.min(max, n)); }
function computePanelMetrics() {
  nextTick(() => {
    // 测量 header 实际高度
    headerHeight.value = Math.ceil(headerRef.value?.getBoundingClientRect().height || 56);
    // 测量首个卡片高度
    let cardH = DEFAULT_CARD_H;
    const firstItem = listEl.value?.querySelector('.overlay-streamer-item') as HTMLElement | null;
    if (firstItem) {
      cardH = Math.ceil(firstItem.getBoundingClientRect().height);
    }
    // 读取 grid gap（如果可用）
    let gapPx = 14;
    if (listEl.value) {
      const cs = window.getComputedStyle(listEl.value);
      const gapStr = (cs as any).gap || cs.rowGap;
      const parsed = parseFloat(gapStr || '');
      if (!isNaN(parsed)) gapPx = Math.round(parsed);
    }
    const count = Array.isArray(props.items) ? props.items.length : 0;
    const columns = 4;
    const rows = Math.max(1, Math.ceil(count / columns));
    const contentHeight = rows * cardH + (rows - 1) * gapPx + LIST_PAD_TOP + LIST_PAD_BOTTOM;
    const desired = headerHeight.value + contentHeight + 8 + 10; // overlay-content padding: 上8 下10
    const maxH = (typeof window !== 'undefined') ? (window.innerHeight - PANEL_MAX_MARGIN) : desired;
    panelHeight.value = clamp(desired, PANEL_MIN, maxH);
    shouldScroll.value = desired > maxH;
    const vh = (typeof window !== 'undefined') ? window.innerHeight : panelHeight.value;
    panelTop.value = Math.max(16, Math.round((vh - panelHeight.value) / 2));
  });
}

onMounted(() => {
  computePanelMetrics();
  const onResize = () => computePanelMetrics();
  window.addEventListener('resize', onResize);
  resizeListener = onResize;
});
let resizeListener: ((this: Window, ev: UIEvent) => any) | null = null;
onUnmounted(() => { if (resizeListener) window.removeEventListener('resize', resizeListener); });
watch(() => props.items, () => computePanelMetrics(), { deep: true });
watch(() => props.show, (v) => { if (v) computePanelMetrics(); });
watch(() => props.isDeleteMode, () => computePanelMetrics());

const handleSelect = (s: FollowedStreamer) => {
  if (props.isDeleteMode) return;
  emit('select', s);
};
</script>

<style scoped>
.follow-overlay-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.45);
  backdrop-filter: blur(2px);
  z-index: 1000;
}
.follow-overlay-panel {
  position: fixed;
  width: min(1160px, 94vw);
  border-radius: 14px;
  background: var(--primary-bg);
  color: var(--primary-text);
  border: 1px solid var(--border-color);
  box-shadow: 0 16px 40px rgba(0,0,0,0.38);
  overflow: visible;
  transform: translateZ(0);
}
.overlay-header {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  padding: 12px 16px;
  padding-left: 18px; /* 与下方 overlay-content 左边距对齐 */
  border-bottom: 1px solid var(--border-color);
}
.overlay-header-left {
  display: flex;
  align-items: center;
  gap: 8px; /* 更紧凑 */
  flex: 1;
  min-width: 0;
}
.overlay-header-actions { 
  display: flex; 
  align-items: center; 
  gap: 6px;
  margin-left: auto; /* 靠右显示 */
  position: relative; /* 保持分隔线伪元素定位 */
}
.overlay-header-actions::before {
  content: '';
  position: absolute;
  left: -10px;
  top: 8px;
  bottom: 8px;
  width: 1px;
  background: var(--border-color);
  opacity: 0.6;
}
.overlay-text-btn {
  background: var(--card-bg, rgba(255,255,255,0.06));
  border: 1px solid var(--border-color);
  color: var(--primary-text);
  padding: 6px 10px;
  border-radius: 8px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  transition: background 0.2s ease, border-color 0.2s ease;
}
.overlay-text-btn:hover { background: var(--card-hover-bg, rgba(255,255,255,0.08)); border-color: var(--border-color-strong, #4b5563); }
.overlay-text-btn:disabled { opacity: 0.6; cursor: default; }
.manage-action.active {
  background: rgba(220, 38, 38, 0.18);
  border-color: rgba(248, 113, 113, 0.45);
  color: #fca5a5;
}
.manage-action.active:hover {
  background: rgba(248, 113, 113, 0.24);
  border-color: rgba(248, 113, 113, 0.6);
}
.manage-action span { letter-spacing: 0.02em; }
:root[data-theme="light"] .manage-action.active {
  background: rgba(248, 113, 113, 0.2);
  border-color: rgba(248, 113, 113, 0.5);
  color: #dc2626;
}
:root[data-theme="light"] .manage-action.active:hover {
  background: rgba(248, 113, 113, 0.28);
  border-color: rgba(248, 113, 113, 0.65);
}

.refresh-action { min-width: 64px; }
.refresh-action .refresh-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  margin-left: 4px;
  visibility: hidden;
}
.refresh-action.is-refreshing .refresh-spinner { visibility: visible; animation: spin 0.9s linear infinite; }
.refresh-action.just-finished { position: relative; }
.refresh-action.just-finished::after {
  content: '✓';
  font-weight: 700;
  color: #22c55e;
  margin-left: 4px;
  animation: pop 0.35s ease;
}
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
@keyframes pop { 0% { transform: scale(0.6); opacity: 0; } 100% { transform: scale(1); opacity: 1; } }

.overlay-content {
  overflow: auto;
  padding: 8px 18px 10px; /* 减小上下 padding，减少出现滚动条的概率 */
  will-change: scroll-position;
  transform: translateZ(0);
}
.overlay-content::-webkit-scrollbar { width: 3px; height: 3px; }
.overlay-content::-webkit-scrollbar-track { background: var(--scrollbar-track-bg, #18181b); border-radius: 3px; }
.overlay-content::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb-bg, #4b5563); border-radius: 3px; }
:root[data-theme="light"] .overlay-content::-webkit-scrollbar-track { background: var(--scrollbar-track-bg-light, #e9ecef); }
:root[data-theme="light"] .overlay-content::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb-bg-light, #adb5bd); }

.overlay-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: var(--secondary-text);
}

.overlay-streamers-list {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 14px;
  list-style: none;
  margin: 0;
  padding: 6px 0; /* 上下各加一点 padding，已纳入高度计算 */
  contain: paint;
  transform: translateZ(0);
}
.overlay-streamer-item {
  padding: 5px 8px; /* 与侧边栏 .streamer-item 保持一致，更紧凑高度 */
  border-radius: 12px;
  border: 1px solid var(--border-color);
  background: var(--card-bg, rgba(255,255,255,0.04));
  cursor: pointer;
  transition: transform 0.18s ease, background 0.2s ease, box-shadow 0.2s ease;
  overflow: hidden;
  will-change: transform, opacity;
  backface-visibility: hidden;
  transform: translateZ(0);
  position: relative;
}
.overlay-streamer-item:hover {
  background: var(--card-hover-bg, rgba(255,255,255,0.08));
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(0,0,0,0.24);
}
.overlay-streamer-item.remove-mode {
  cursor: default;
  transform: none;
  box-shadow: none;
  background: var(--card-bg, rgba(255,255,255,0.04));
}
.overlay-remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  border: none;
  background: transparent;
  color: rgba(248, 113, 113, 0.82);
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  line-height: 1;
  padding: 0;
  z-index: 2;
  transition: color 0.2s ease, transform 0.2s ease;
}
.overlay-remove-btn:hover {
  color: rgba(248, 113, 113, 1);
  transform: scale(1.1);
}

.overlay-fade-enter-active,
.overlay-fade-leave-active { transition: opacity 0.2s ease; }
.overlay-fade-enter-from,
.overlay-fade-leave-to { opacity: 0; }
.overlay-pop-enter-active,
.overlay-pop-leave-active { transition: transform 0.22s ease, opacity 0.22s ease; }
.overlay-pop-enter-from { transform: translateY(8px); opacity: 0; }
.overlay-pop-leave-to { transform: translateY(8px); opacity: 0; }
.overlay-close-btn {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.9);
  width: 32px;
  height: 32px;
  border-radius: 16px; /* 改为圆角方形，和关注列表按钮一致 */
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  padding: 0;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  position: absolute; /* 固定在面板右上角 */
  top: -16px; /* 上移半个高度，使其一半在外一半在内 */
  right: -16px; /* 右移半个宽度，使其一半在外一半在内 */
  z-index: 100;
}
.overlay-close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.08);
  border-color: rgba(255, 255, 255, 0.3);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
}
.overlay-close-btn:active { transform: scale(0.95); }
.overlay-close-btn svg { width: 16px; height: 16px; }
:root[data-theme="light"] .overlay-close-btn {
  background: var(--button-bg-light, rgba(255, 255, 255, 0.9));
  border: 1px solid var(--button-border-light, rgba(0, 0, 0, 0.1));
  color: var(--button-text-light, #333333);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}
:root[data-theme="light"] .overlay-close-btn:hover {
  background: var(--button-hover-bg-light, rgba(245, 245, 245, 0.95));
  border-color: var(--button-hover-border-light, rgba(0, 0, 0, 0.15));
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.12);
}
</style>
