<template>
  <div v-if="!isLoading && (cate3List.length > 0 || hasAllOption)" class="cate3-list">
    <!-- 全部选项 -->
    <div
      class="cate3-item"
      :class="{ active: selectedCate3Id === null || selectedCate3Id === 'all' }"
      @click="selectAll"
    >
      全部
    </div>
    
    <!-- 其他三级分类 -->
    <div
      v-for="cate3 in cate3List"
      :key="cate3.id"
      class="cate3-item"
      :class="{ active: selectedCate3Id === cate3.id }"
      @click="$emit('select', cate3)"
    >
      {{ cate3.name }}
    </div>
  </div>
  <div v-if="isLoading" class="loading-cate3">正在加载三级分类...</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Category3 } from '../types'

const props = defineProps<{
  cate3List: Category3[]
  selectedCate3Id: string | null
  isLoading: boolean
}>()

const emit = defineEmits<{
  (e: 'select', cate3: Category3): void
}>()

// 计算属性：是否显示全部选项
const hasAllOption = computed(() => {
  return props.cate3List && props.cate3List.length > 0
})

// 选择"全部"
const selectAll = () => {
  // 创建一个特殊的"全部"分类对象
  const allCategory: Category3 = {
    id: 'all',
    name: '全部'
  }
  emit('select', allCategory)
}
</script>

<style scoped>
.cate3-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 2px 16px 0 16px;
  padding-bottom: 4px;
}

.cate3-item {
  /* Shared dimensions and basic properties from cate2-card */
  /* width: 120px; */ /* Cate3 items are usually narrower, let flexbox decide or define custom width if needed */
  padding: 4px 12px;
  height: 30px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
  box-sizing: border-box;
  overflow: hidden;
  /* text-align: center; */ /* Usually left-aligned for list items */
  display: inline-flex; /* To align text vertically like cate2-card */
  align-items: center;
  font-size: 13px; /* Existing font size */

  /* Night mode default (Matches cate2-card night mode default) */
  background: var(--cate2-card-bg-dark, rgba(31, 31, 35, 1)); 
  border: 1px solid var(--cate2-card-border-dark, transparent); 
  color: var(--cate2-card-text-dark, #ccc); 
}

/* Day Mode: Default (Matches cate2-card day mode default) */
:root[data-theme="light"] .cate3-item {
  background-color: var(--content-card-bg-light, #FFFFFF);
  border: 1px solid var(--content-card-border-light, #e0e0e0);
  box-shadow: var(--content-card-shadow-light, 0 1px 3px rgba(0,0,0,0.08));
  color: var(--main-text-primary-light, #212529);
}

/* Night mode default hover (Matches cate2-card night mode default hover) - Added Glow */
.cate3-item:hover {
  background-color: var(--cate2-card-hover-bg-dark, rgba(45, 48, 56, 1));
  border: 1px solid var(--cate2-card-hover-border-dark-glow, rgba(79, 209, 197, 0.5)); /* Cyan glow border, more transparent */
  color: var(--cate2-card-text-dark-hover, #ddd);
  box-shadow: var(--cate2-card-hover-shadow-dark-glow, 0 0 5px rgba(79, 209, 197, 0.2), 0 0 0 1px rgba(79, 209, 197, 0.15)); /* Softer cyan glow */
}

/* Day Mode: Hover style (Matches cate2-card day mode hover - Glow Effect) */
:root[data-theme="light"] .cate3-item:hover {
  background-color: var(--content-card-hover-bg-light, #f8f9fa);
  border: 1px solid transparent;
  box-shadow: var(--douyu-cate2-hover-shadow-light-glow, 0 0 0 2px rgba(50, 150, 255, 0.35), 0 0 9px rgba(50, 150, 255, 0.2));
  /* Text color inherited from day mode .cate3-item or .cate2-card */
}

/* Night Mode Active (Matches cate2-card night mode active - Revived Glow Style) */
.cate3-item.active { 
  background-color: var(--cate2-card-bg-dark, rgba(31, 31, 35, 1)); 
  border: 1px solid transparent; /* Removed border for night mode active */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3)); 
  font-weight: 500; /* Keep existing font-weight for active */
}

:root[data-theme="dark"] .cate3-item.active {
  background-color: var(--cate2-card-bg-dark, rgba(31, 31, 35, 1)); 
  border-color: transparent; /* Removed border for night mode active */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3)); 
  /* Text color for active item name specifically handled below */
}

/* Text color for name inside active card in dark mode (Matches cate2-card night mode active text) */
:root[data-theme="dark"] .cate3-item.active { 
  color: var(--douyu-cate2-active-text-dark-glow-revived, rgb(79, 209, 197)); 
}

/* Night Mode: Hover on ACTIVE card (Matches cate2-card night mode active hover - Intensified Glow) */
:root[data-theme="dark"] .cate3-item.active:hover {
  background-color: var(--cate2-card-bg-dark, rgba(31, 31, 35, 1)); 
  border-color: var(--douyu-cate2-active-border-dark-glow-revived, rgb(79, 209, 197)); 
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived-hover, 0 0 0 2.5px rgba(79, 209, 197, 0.55), 0 0 12px rgba(79, 209, 197, 0.4)); 
  /* Text color inherited from .active night mode style */
}

/* Light Mode Active (Matches cate2-card light mode active - Scheme A: Deep Emphasis) */
:root[data-theme="light"] .cate3-item.active {
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); 
  border-color: transparent; 
  box-shadow: var(--douyu-cate2-active-shadow-light-scheme-a, 0 2px 5px rgba(0,0,0,0.12)); 
  font-weight: 500; /* Keep existing font-weight for active */
  /* Text color for active item name specifically handled below */
}

/* Text color for name inside active card in light mode (Matches cate2-card light mode active text) */
:root[data-theme="light"] .cate3-item.active {
  color: var(--douyu-cate2-active-text-light-scheme-a, #FFFFFF);
}

/* Hover on Active card in Light Mode (Matches cate2-card light mode active hover - Scheme A) */
:root[data-theme="light"] .cate3-item.active:hover {
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); 
  box-shadow: var(--content-card-hover-shadow-light-larger, 0 5px 12px rgba(0,0,0,0.15)); 
  border: 1px solid transparent;
  /* Text color inherited from .active light mode style */
}

.loading-cate3 {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 10px;
  color: var(--cate3-loading-text-dark, rgba(255, 255, 255, 0.5)); 
  font-size: 13px;
}

:root[data-theme="light"] .loading-cate3 {
  color: var(--main-text-secondary-light, #495057); 
}
</style>