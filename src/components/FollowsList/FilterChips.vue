<template>
  <div class="filter-group">
    <button 
      class="filter-chip" 
      :class="{ active: activeFilter === 'ALL' }" 
      @click="$emit('update:activeFilter', 'ALL')"
    >全部</button>
    <button 
      v-for="p in visiblePlatforms" 
      :key="p" 
      class="filter-chip" 
      :class="{ active: activeFilter === p }" 
      @click="$emit('update:activeFilter', p)"
    >{{ platformLabel(p) }}</button>
  </div>
</template>

<script setup lang="ts">
import { Platform } from '../../platforms/common/types';

const props = defineProps<{
  visiblePlatforms: Platform[],
  activeFilter: 'ALL' | Platform,
}>();

const platformLabel = (p: Platform): string => {
  switch (p) {
    case Platform.DOUYU: return '斗鱼';
    case Platform.DOUYIN: return '抖音';
    case Platform.HUYA: return '虎牙';
    case Platform.BILIBILI: return 'B站';
    default: return '未知';
  }
};
</script>

<style scoped>
.filter-group { display: flex; align-items: center; gap: 8px; }
.filter-chip {
  padding: 6px 12px; /* 稍微加大触控面积 */
  border-radius: 10px;
  border: 1px solid var(--border-color);
  background: var(--card-bg, rgba(255,255,255,0.04));
  color: var(--secondary-text);
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease, transform 0.2s ease;
  font-weight: 500;
  backdrop-filter: saturate(130%) blur(4px);
}
.filter-chip:hover {
  background: var(--card-hover-bg, rgba(255,255,255,0.08));
  transform: translateY(-1px);
}
.filter-chip.active {
  /* 选中态：强调背景与字体变化 */
  background: linear-gradient(180deg, rgba(0,218,198,0.22), rgba(0,218,198,0.15));
  border-color: rgba(0,218,198,0.45);
  color: var(--primary-text);
  box-shadow: 0 6px 16px rgba(0,218,198,0.25);
  font-weight: 600;
}
:root[data-theme="light"] .filter-chip.active {
  background: linear-gradient(180deg, rgba(0,218,198,0.12), rgba(0,218,198,0.08));
  border-color: rgba(0,218,198,0.35);
  box-shadow: 0 4px 10px rgba(0,218,198,0.18);
}
</style>