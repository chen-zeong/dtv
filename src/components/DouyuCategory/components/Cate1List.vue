<template>
  <div class="cate1-list">
    <div 
      v-for="cate1 in cate1List" 
      :key="cate1.cate1Id"
      class="cate1-item"
      :class="{ active: selectedCate1Id === cate1.cate1Id }"
      @click="$emit('select', cate1.cate1Id)"
    >
      <span class="cate1-name">{{ cate1.cate1Name }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Category1 } from '../types'

defineProps<{
  cate1List: Category1[]
  selectedCate1Id: number | null
}>()

defineEmits<{
  (e: 'select', cate1Id: number): void
}>()
</script>

<style scoped>
.cate1-list {
  padding: 12px;
  display: flex;
  gap: 12px;
  overflow-x: auto;
  background: var(--cate1-list-bg-dark, #1f1f23); /* Night mode default */
  flex-shrink: 0;
  transition: background-color 0.3s ease;
}

:root[data-theme="light"] .cate1-list {
  background-color: var(--primary-bg, #FFFFFF); /* Use --primary-bg */
  color: var(--primary-text); /* Ensure text color is also themed */
}

.cate1-item {
  padding: 6px 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 13px;
  border-radius: 20px;
  white-space: nowrap;
  border: 1px solid transparent; /* Add for consistent box sizing with active state */
  /* Night mode default styles */
  background: var(--cate1-item-bg-dark, rgba(255, 255, 255, 0.1));
  color: var(--cate1-item-text-dark, #e0e0e0); 
}

:root[data-theme="light"] .cate1-item {
  background-color: var(--control-bg-light, #f0f2f5);
  color: var(--control-text-light, #343a40);
  border-color: var(--control-border-light, #ced4da); /* Example light border */
}

.cate1-item:hover {
  /* Night mode hover */
  background: var(--cate1-item-hover-bg-dark, rgba(255, 255, 255, 0.2));
  color: var(--primary-text, #FFFFFF); /* Brighter text on hover for dark */
}

:root[data-theme="light"] .cate1-item:hover {
  background-color: var(--control-hover-bg-light, #e2e6ea);
  border-color: var(--control-hover-border-light, #adb5bd);
}

/* Night Mode Active - Matches current Cate2 active style */
.cate1-item.active {
  background-color: var(--cate2-card-bg-dark, rgba(31, 31, 35, 1)); /* Match Cate2 unselected night bg */
  border: 1px solid transparent; /* Removed border for night mode active */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3));
  color: var(--douyu-cate2-active-text-dark-glow-revived, rgb(79, 209, 197));
  font-weight: 500;
}

/* Light Mode Active - Matches current Cate2 active style */
:root[data-theme="light"] .cate1-item.active {
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); 
  border-color: transparent; 
  box-shadow: var(--douyu-cate2-active-shadow-light-scheme-a, 0 2px 5px rgba(0,0,0,0.12)); 
  color: var(--douyu-cate2-active-text-light-scheme-a, #FFFFFF); 
  font-weight: 500;
}
</style>