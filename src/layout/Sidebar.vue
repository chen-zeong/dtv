<template>
  <aside class="app-sidebar">
    <nav class="navigation">
      <router-link 
        v-for="item in navItems" 
        :key="item.name"
        :to="item.path" 
        class="nav-item"
        :class="{ 'is-active': $route.path === item.path }"
        @click="() => emit('navigate', item.path)"
      >
        <span class="nav-label">{{ item.name }}</span>
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
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import type { FollowedStreamer } from '../platforms/common/types';
import FollowList from '../components/FollowsList/index.vue';

const emit = defineEmits(['selectAnchor', 'unfollow', 'navigate', 'reorderList']);
const router = useRouter();

const navItems = ref([
  { name: '斗鱼直播', path: '/' },
  { name: '抖音直播', path: '/douyin' },
  { name: '虎牙直播', path: '/huya' },
  { name: 'B站直播', path: '/bilibili' },
]);

const props = withDefaults(defineProps<{
  followedAnchors?: FollowedStreamer[];
}>(), {
  followedAnchors: () => []
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
  padding: 16px 0;
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
  padding: 0 8px;
  margin-bottom: 8px;
  gap: 12px;
  margin-top: 12px;
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding: 8px 16px;
  border-radius: 10px;
  text-decoration: none;
  transition: all 0.25s ease;
  font-size: 14px;
  font-weight: 500;
  position: relative;
  overflow: hidden;

  /* Dark Mode Default Styles (mostly from original) */
  color: var(--secondary-text, #a0aec0);
  background: rgba(255, 255, 255, 0.03); /* Subtle bg for dark */
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.nav-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  background: transparent; /* Default transparent */
  transition: all 0.25s ease;
}

/* Dark Mode Hover/Active for ::before (using new colors) */
:root[data-theme="dark"] .nav-item:hover::before {
  background: var(--sidebar-nav-item-hover-border-dark, rgba(0, 218, 198, 0.3));
}
:root[data-theme="dark"] .nav-item.is-active::before {
  background: var(--sidebar-nav-item-active-border-dark, #00DAC6);
  box-shadow: 0 0 10px var(--sidebar-nav-item-active-border-shadow-dark, rgba(0, 218, 198, 0.4));
}
/* Dark Mode active/inactive backgrounds */
:root[data-theme="dark"] .nav-item { background: #242427; }
:root[data-theme="dark"] .nav-item.is-active {
  background: #25272f;
  color: #FFFFFF;
  font-weight: 600;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
:root[data-theme="dark"] .nav-item:hover {
  background: var(--card-hover-bg, rgba(255,255,255,0.06)); /* Use themed or original fallback */
  color: var(--primary-text, #e2e8f0); /* Use themed or original fallback */
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1); /* Original hover shadow */
}


/* Light Theme Overrides for Nav Item */
:root[data-theme="light"] .nav-item {
  background: var(--sidebar-nav-item-bg-light, #FFFFFF);
  color: #4A5568; /* Updated non-active light mode text color - lighter gray */
  box-shadow: var(--sidebar-nav-item-shadow-light, 0 2px 4px rgba(0,0,0,0.06));
}

:root[data-theme="light"] .nav-item:hover {
  background: var(--sidebar-nav-item-hover-bg-light, #F7FAFC);
  color: #2D3748; /* Updated hover light mode text color - darker gray, but not black */
  transform: translateY(-1px);
  box-shadow: var(--sidebar-nav-item-hover-shadow-light, 0 5px 10px rgba(0,0,0,0.08));
}

:root[data-theme="light"] .nav-item.is-active {
  background: var(--sidebar-nav-item-active-bg-light, #FFFFFF);
  color: #1A202C; /* Active color remains deep black */
  font-weight: 600;
  box-shadow: var(--sidebar-nav-item-active-shadow-light, 0 5px 12px rgba(0,0,0,0.1));
}

:root[data-theme="light"] .nav-item:hover::before {
  background: var(--sidebar-nav-item-hover-border-light, rgba(66, 153, 225, 0.4));
}

:root[data-theme="light"] .nav-item.is-active::before {
  background: var(--sidebar-nav-item-active-border-light, #4299E1);
  box-shadow: 0 0 8px var(--sidebar-nav-item-active-border-light, rgba(66, 153, 225, 0.5)); /* Shadow for light active border */
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