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
  padding: 6px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-color);
  background: var(--card-bg, rgba(255,255,255,0.04));
  color: var(--secondary-text);
  cursor: pointer;
}
.filter-chip.active {
  background: var(--card-hover-bg, rgba(255,255,255,0.08));
  color: var(--primary-text);
}
</style>