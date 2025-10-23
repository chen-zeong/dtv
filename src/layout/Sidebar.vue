<template>
  <aside class="app-sidebar">
    <nav class="navigation" ref="navListRef" :data-active-platform="activePlatformKey || undefined">
      <span
        v-if="highlight.visible"
        class="nav-shared-highlight"
        :style="highlightStyles"
        aria-hidden="true"
      ></span>
      <router-link 
        v-for="item in navItems" 
        :key="item.key"
        :to="item.path" 
        custom
        v-slot="{ href, navigate, isActive }"
      >
        <a
          :href="href"
          class="nav-item"
          :data-platform="item.key"
          :class="{ 'is-active': isActive }"
          @click="(event) => handleNavClick(event, navigate, item.path)"
          :ref="(el) => setNavItemRef(item.key, el)"
        >
          <span class="nav-backdrop" aria-hidden="true"></span>
          <span class="nav-glow" aria-hidden="true"></span>
          <div class="nav-content">
            <span class="nav-icon" aria-hidden="true">
              <img :src="item.logo" alt="" loading="lazy" />
            </span>
            <span class="nav-name">{{ item.name }}</span>
          </div>
          <span class="nav-indicator" aria-hidden="true"></span>
        </a>
      </router-link>
    </nav>
    
    <FollowList 
      :followedAnchors="sortedFollowedAnchors"
      @selectAnchor="handleSelectAnchor"
      @unfollow="handleUnfollow"
      @reorderList="handleReorderList"
      class="follow-list-component"
    />
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import type { FollowedStreamer } from '../platforms/common/types';
import FollowList from '../components/FollowsList/index.vue';
import douyuLogo from '../assets/douyu.webp';
import douyinLogo from '../assets/douyin.webp';
import huyaLogo from '../assets/huya.webp';
import bilibiliLogo from '../assets/bilibili.png';

const emit = defineEmits(['selectAnchor', 'unfollow', 'navigate', 'reorderList']);
const router = useRouter();
const route = useRoute();

type PlatformKey = 'douyu' | 'douyin' | 'huya' | 'bilibili';

type NavItem = {
  key: PlatformKey;
  name: string;
  path: string;
  logo: string;
};

const navItems = ref<NavItem[]>([
  {
    key: 'douyu',
    name: '斗鱼直播',
    path: '/',
    logo: douyuLogo,
  },
  {
    key: 'douyin',
    name: '抖音直播',
    path: '/douyin',
    logo: douyinLogo,
  },
  {
    key: 'huya',
    name: '虎牙直播',
    path: '/huya',
    logo: huyaLogo,
  },
  {
    key: 'bilibili',
    name: '哔哩哔哩直播',
    path: '/bilibili',
    logo: bilibiliLogo,
  },
]);

const props = withDefaults(defineProps<{
  followedAnchors?: FollowedStreamer[];
}>(), {
  followedAnchors: () => []
});

const navListRef = ref<HTMLElement | null>(null);
const navItemRefs = new Map<PlatformKey, HTMLElement>();

const highlight = reactive({
  offset: 0,
  height: 0,
  visible: false,
});

const activePlatformKey = computed<PlatformKey | null>(() => {
  const match = navItems.value.find(item => item.path === route.path);
  return match?.key ?? null;
});

const highlightStyles = computed(() => ({
  transform: `translateY(${highlight.offset}px)`,
  height: `${highlight.height}px`,
  opacity: highlight.visible ? 1 : 0,
}));

const updateHighlight = () => {
  nextTick(() => {
    const key = activePlatformKey.value;
    if (!key) {
      highlight.visible = false;
      return;
    }
    const el = navItemRefs.get(key);
    const container = navListRef.value;
    if (!el || !container) {
      highlight.visible = false;
      return;
    }
    highlight.offset = el.offsetTop;
    highlight.height = el.offsetHeight;
    highlight.visible = true;
  });
};

const setNavItemRef = (key: PlatformKey, el: Element | null) => {
  if (!el) {
    navItemRefs.delete(key);
    return;
  }
  if (el instanceof HTMLElement) {
    navItemRefs.set(key, el);
    if (key === activePlatformKey.value) {
      updateHighlight();
    }
  }
};

const handleNavClick = (event: MouseEvent, navigate: (e?: MouseEvent) => void, path: string) => {
  navigate(event);
  emit('navigate', path);
};

watch(() => route.path, () => updateHighlight(), { immediate: true });

const handleResize = () => updateHighlight();

onMounted(() => {
  window.addEventListener('resize', handleResize);
  updateHighlight();
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
});

// 保存自定义排序列表
const customSortedAnchors = ref<FollowedStreamer[]>([]);

// 按直播状态排序
const sortedFollowedAnchors = computed(() => {
  if (!props.followedAnchors?.length) return [];
  
  // Use composite key platform:id to avoid collisions across platforms
  const toKey = (a: FollowedStreamer) => `${a.platform}:${a.id}`;
  const currentKeys = new Set(props.followedAnchors.map(toKey));

  const customSortedIsValid = customSortedAnchors.value.length > 0 && 
    customSortedAnchors.value.length === props.followedAnchors.length && 
    customSortedAnchors.value.every(customAnchor => currentKeys.has(toKey(customAnchor)));

  let baseOrder: FollowedStreamer[];
  if (customSortedIsValid) {
    const propsMap = new Map(props.followedAnchors.map(anchor => [toKey(anchor), anchor]));
    baseOrder = customSortedAnchors.value
      .map(customAnchor => propsMap.get(toKey(customAnchor)))
      .filter(Boolean) as FollowedStreamer[];
  } else {
    baseOrder = [...props.followedAnchors];
  }

  // Group by live first (based on liveStatus), then non-live, preserving relative order within each group
  const live = baseOrder.filter(a => a.liveStatus === 'LIVE');
  const notLive = baseOrder.filter(a => a.liveStatus !== 'LIVE');
  return [...live, ...notLive];
});

const handleSelectAnchor = (anchor: FollowedStreamer) => {
  emit('selectAnchor', anchor);
};

const handleUnfollow = (payload: {platform: any, id: string} | string) => {
    if (typeof payload === 'string') {
        emit('unfollow', { platform: undefined, id: payload }); 
    } else {
        emit('unfollow', payload); 
    }
};

const handleReorderList = (reorderedList: FollowedStreamer[]) => {
  customSortedAnchors.value = [...reorderedList];
  emit('reorderList', reorderedList);
};

defineExpose({ router });
</script>

<style scoped>
.app-sidebar {
  width: 240px;
  /* Default to dark mode background variable */
  background-color: var(--sidebar-bg-dark, #18181b); /* Fallback if var not defined */
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color); /* Assuming --border-color is themed */
  padding: 10px 0 12px;
  transition: background-color 0.3s ease, border-color 0.3s ease;
  color: var(--secondary-text); /* Default text color, might need light mode override */
}

/* Light Theme Overrides for Sidebar */
:root[data-theme="light"] .app-sidebar {
  background-color: var(--sidebar-bg-light, #f6f6f6);
  border-right-color: var(--border-color-light, #e2e8f0); /* Define if --border-color isn't fully themed */
  color: var(--sidebar-nav-item-text-light, #4A5568);
}

.navigation {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 12px 12px;
  margin: 0;
  position: relative;
}

.nav-shared-highlight {
  position: absolute;
  left: 14px;
  right: 14px;
  border-radius: 16px;
  background: linear-gradient(135deg, rgba(24, 28, 48, 0.45), rgba(18, 22, 38, 0.18));
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 10px 22px rgba(12, 16, 35, 0.28);
  backdrop-filter: blur(22px);
  -webkit-backdrop-filter: blur(22px);
  transition: transform 0.45s cubic-bezier(0.2, 0.8, 0.2, 1), height 0.45s cubic-bezier(0.2, 0.8, 0.2, 1), opacity 0.28s ease, background 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
  pointer-events: none;
  z-index: 0;
}

:root[data-theme="light"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.9), rgba(236, 241, 255, 0.8));
  border-color: rgba(148, 163, 184, 0.3);
  box-shadow: 0 12px 26px rgba(203, 213, 225, 0.26);
}

.navigation[data-active-platform="douyu"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 130, 28, 0.32), rgba(191, 78, 6, 0.52));
  border-color: rgba(255, 183, 122, 0.28);
  box-shadow: 0 12px 34px rgba(255, 120, 16, 0.28);
}

.navigation[data-active-platform="douyin"] .nav-shared-highlight {
  background: linear-gradient(140deg, rgba(124, 59, 255, 0.38), rgba(13, 13, 18, 0.82));
  border-color: rgba(145, 125, 255, 0.36);
  box-shadow: 0 12px 32px rgba(120, 66, 255, 0.36);
}

.navigation[data-active-platform="huya"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 193, 37, 0.36), rgba(177, 115, 11, 0.44));
  border-color: rgba(255, 215, 128, 0.34);
  box-shadow: 0 12px 32px rgba(255, 193, 37, 0.3);
}

.navigation[data-active-platform="bilibili"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 77, 109, 0.36), rgba(160, 27, 49, 0.52));
  border-color: rgba(255, 139, 155, 0.36);
  box-shadow: 0 12px 34px rgba(255, 77, 109, 0.3);
}

:root[data-theme="light"] .navigation[data-active-platform="douyu"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 183, 122, 0.44), rgba(255, 133, 52, 0.3));
  border-color: rgba(255, 160, 96, 0.4);
  box-shadow: 0 14px 30px rgba(255, 163, 101, 0.26);
}

:root[data-theme="light"] .navigation[data-active-platform="douyin"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(203, 173, 255, 0.46), rgba(148, 163, 184, 0.38));
  border-color: rgba(180, 160, 255, 0.36);
  box-shadow: 0 14px 30px rgba(168, 139, 255, 0.24);
}

:root[data-theme="light"] .navigation[data-active-platform="huya"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 221, 133, 0.5), rgba(255, 200, 92, 0.34));
  border-color: rgba(255, 202, 110, 0.4);
  box-shadow: 0 14px 30px rgba(255, 205, 115, 0.26);
}

:root[data-theme="light"] .navigation[data-active-platform="bilibili"] .nav-shared-highlight {
  background: linear-gradient(135deg, rgba(255, 164, 178, 0.48), rgba(255, 110, 133, 0.34));
  border-color: rgba(255, 138, 157, 0.42);
  box-shadow: 0 14px 30px rgba(255, 128, 149, 0.26);
}

.nav-item {
  --nav-accent: #6c7bff;
  --nav-accent-secondary: #4650d6;
  --nav-accent-soft: rgba(108, 123, 255, 0.18);
  --nav-icon-bg: rgba(108, 123, 255, 0.14);
  --nav-glow-color: rgba(108, 123, 255, 0.42);
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  border-radius: 16px;
  text-decoration: none;
  overflow: hidden;
  color: var(--secondary-text, #a0aec0);
  background: linear-gradient(135deg, rgba(24, 26, 35, 0.78), rgba(15, 17, 25, 0.7));
  border: 1px solid rgba(255, 255, 255, 0.03);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.015);
  transition: background 0.45s ease, box-shadow 0.45s ease, border-color 0.4s ease, color 0.3s ease;
  isolation: isolate;
}

.nav-item[data-platform="douyu"] {
  --nav-accent: #ff7a1c;
  --nav-accent-secondary: #ffb35c;
  --nav-accent-soft: rgba(255, 133, 50, 0.26);
  --nav-icon-bg: rgba(255, 133, 50, 0.16);
  --nav-glow-color: rgba(255, 122, 28, 0.5);
}

.nav-item[data-platform="douyin"] {
  --nav-accent: #7c3aed;
  --nav-accent-secondary: #0f172a;
  --nav-accent-soft: rgba(124, 58, 237, 0.28);
  --nav-icon-bg: rgba(124, 58, 237, 0.18);
  --nav-glow-color: rgba(124, 58, 237, 0.52);
}

.nav-item[data-platform="huya"] {
  --nav-accent: #fbbf24;
  --nav-accent-secondary: #f97316;
  --nav-accent-soft: rgba(251, 191, 36, 0.28);
  --nav-icon-bg: rgba(251, 191, 36, 0.18);
  --nav-glow-color: rgba(251, 191, 36, 0.5);
}

.nav-item[data-platform="bilibili"] {
  --nav-accent: #ff4d6d;
  --nav-accent-secondary: #d72638;
  --nav-accent-soft: rgba(255, 77, 109, 0.26);
  --nav-icon-bg: rgba(255, 77, 109, 0.18);
  --nav-glow-color: rgba(255, 77, 109, 0.52);
}

.nav-backdrop,
.nav-glow {
  position: absolute;
  pointer-events: none;
  border-radius: inherit;
  inset: 0;
  opacity: 0;
  transition: opacity 0.45s ease, transform 0.45s ease;
}

.nav-backdrop {
  background: linear-gradient(135deg, var(--nav-accent-soft), rgba(255, 255, 255, 0.05));
  transform: scale(0.92);
}

.nav-glow {
  background: radial-gradient(ellipse at 20% 30%, var(--nav-glow-color), transparent 70%);
  filter: blur(12px);
  mix-blend-mode: screen;
  transform: scale(0.8);
}

.nav-indicator {
  position: absolute;
  top: 50%;
  right: 18px;
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: linear-gradient(145deg, var(--nav-accent), var(--nav-accent-secondary));
  box-shadow: 0 0 12px rgba(255, 255, 255, 0.35);
  opacity: 0;
  transform: translateY(-50%) scale(0.2);
  transition: opacity 0.32s ease, transform 0.32s ease;
}

.nav-content {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  position: relative;
  z-index: 1;
}

.nav-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), var(--nav-icon-bg));
  color: var(--nav-accent);
  box-shadow: inset 0 -1px 4px rgba(0, 0, 0, 0.2), 0 8px 14px rgba(0, 0, 0, 0.18);
  transition: transform 0.35s ease, color 0.35s ease, box-shadow 0.35s ease;
}

.nav-icon svg {
  width: 22px;
  height: 22px;
}

.nav-icon img {
  width: 72%;
  height: 72%;
  object-fit: contain;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.22));
}

.nav-name {
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--primary-text, #e2e8f0);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-item:hover,
.nav-item:focus-visible {
  color: var(--primary-text, #e2e8f0);
  border-color: rgba(255, 255, 255, 0.09);
  background: linear-gradient(135deg, rgba(28, 31, 43, 0.84), rgba(18, 21, 32, 0.78));
  box-shadow: 0 12px 26px rgba(4, 6, 16, 0.24);
}

.nav-item:hover .nav-backdrop,
.nav-item:focus-visible .nav-backdrop {
  opacity: 0.6;
  transform: scale(1);
}

.nav-item:hover .nav-glow,
.nav-item:focus-visible .nav-glow {
  opacity: 0.85;
  transform: scale(0.95);
  animation: navGlowHover 1.6s ease-in-out infinite alternate;
}

.nav-item:hover .nav-icon,
.nav-item:focus-visible .nav-icon {
  color: var(--nav-accent-secondary);
  box-shadow: inset 0 -1px 4px rgba(0, 0, 0, 0.18), 0 10px 18px rgba(0, 0, 0, 0.25);
}

.nav-item.is-active {
  color: var(--primary-text, #f8fafc);
  border-color: rgba(255, 255, 255, 0.14);
  background: linear-gradient(135deg, rgba(18, 21, 33, 0.78), rgba(10, 11, 18, 0.66));
  box-shadow: 0 16px 28px rgba(3, 7, 18, 0.32);
  animation: navActivate 0.65s ease;
}

.nav-item.is-active .nav-backdrop {
  opacity: 0.75;
  transform: scale(1);
}

.nav-item.is-active .nav-glow {
  opacity: 1;
  transform: scale(1);
  animation: navGlowPulse 3.2s ease-in-out infinite;
}

.nav-item.is-active .nav-indicator {
  opacity: 1;
  transform: translateY(-50%) scale(1);
}

.nav-item.is-active .nav-icon {
  color: var(--nav-accent-secondary);
  transform: scale(1.05);
  box-shadow: inset 0 -1px 4px rgba(0, 0, 0, 0.2), 0 12px 22px rgba(0, 0, 0, 0.28);
  animation: navIconPop 0.4s ease;
}

:root[data-theme="light"] .nav-item {
  color: #4a5568;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.92), rgba(240, 245, 255, 0.86));
  border: 1px solid rgba(148, 163, 184, 0.18);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.24), 0 8px 18px rgba(148, 163, 184, 0.16);
}

:root[data-theme="light"] .nav-item .nav-name {
  color: #1a202c;
}

:root[data-theme="light"] .nav-item .nav-icon {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.96), var(--nav-icon-bg));
  box-shadow: inset 0 -1px 3px rgba(148, 163, 184, 0.22), 0 6px 10px rgba(148, 163, 184, 0.2);
}

:root[data-theme="light"] .nav-item:hover,
:root[data-theme="light"] .nav-item:focus-visible {
  color: #1a202c;
  border-color: rgba(99, 102, 241, 0.25);
  box-shadow: 0 14px 24px rgba(148, 163, 184, 0.22);
}

:root[data-theme="light"] .nav-item.is-active {
  box-shadow: 0 18px 30px rgba(148, 163, 184, 0.26);
}

@keyframes navActivate {
  0% {
    box-shadow: 0 0 0 rgba(0, 0, 0, 0);
    border-color: rgba(255, 255, 255, 0.04);
  }
  60% {
    box-shadow: 0 26px 38px rgba(0, 0, 0, 0.38);
  }
  100% {
    box-shadow: 0 20px 36px rgba(0, 0, 0, 0.32);
  }
}

@keyframes navGlowPulse {
  0% {
    transform: scale(0.96);
    opacity: 0.85;
  }
  50% {
    transform: scale(1.06);
    opacity: 1;
  }
  100% {
    transform: scale(0.98);
    opacity: 0.9;
  }
}

@keyframes navGlowHover {
  0% {
    transform: scale(0.92);
    opacity: 0.65;
  }
  100% {
    transform: scale(1.02);
    opacity: 0.9;
  }
}

@keyframes navIconPop {
  0% {
    transform: scale(0.92);
  }
  60% {
    transform: scale(1.08);
  }
  100% {
    transform: scale(1.05);
  }
}


.follow-list-component {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0; /* Original padding was 0 */
}

/* Apply themed scrollbar for follow list */
.follow-list-component::-webkit-scrollbar {
  width: 4px;
}
.follow-list-component::-webkit-scrollbar-track {
  background: transparent;
  margin-right: 4px; /* Original margin */
}
.follow-list-component::-webkit-scrollbar-thumb {
  background: var(--border-color); /* Themed border color */
  border-radius: 2px;
}
.follow-list-component::-webkit-scrollbar-thumb:hover {
  background: var(--secondary-text); /* Themed secondary text color */
}

/* Light theme specific scrollbar if needed, otherwise it uses themed vars above */
:root[data-theme="light"] .follow-list-component::-webkit-scrollbar-thumb {
  background: var(--border-color-light, #d1d5db); /* Example light scrollbar thumb */
}
:root[data-theme="light"] .follow-list-component::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary-light, #9ca3af); /* Example light scrollbar hover */
}

</style>
