<template>
  <div class="cate1-list-container">
    <ul class="cate1-list">
      <li
        v-for="cate1 in cate1List"
        :key="cate1.href" 
        class="cate1-item"
        :class="{ selected: cate1.href === selectedCate1Href }"
        @click="selectCate1(cate1)"
      >
        <span class="cate1-name">{{ cate1.title }}</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import type { DouyinCategory1 } from '../types'

defineProps<{
  cate1List: DouyinCategory1[]
  selectedCate1Href: string | null
}>()

const emit = defineEmits<{
  (e: 'select', cate1: DouyinCategory1): void
}>()

const selectCate1 = (cate1: DouyinCategory1) => {
  emit('select', cate1)
}
</script>

<style scoped>
/* Styles for DouyinCate1List.vue, adapted from Douyu */
.cate1-list-container { /* Original Douyin container */
  overflow-x: auto; /* Keep this */
  background: var(--cate1-list-bg-dark, #1f1f23); /* Match Douyu's .cate1-list background */
  flex-shrink: 0; /* Keep this */
  transition: background-color 0.3s ease; /* Add this */
}

:root[data-theme="light"] .cate1-list-container {
  background-color: var(--primary-bg, #FFFFFF); /* Match Douyu's .cate1-list light background */
}

.cate1-list { /* This is the ul element in Douyin */
  list-style: none;
  margin: 0;
  padding: 12px; /* From Douyu's .cate1-list */
  display: flex; /* From Douyu's .cate1-list */
  gap: 12px; /* From Douyu's .cate1-list */
  flex-wrap: nowrap; /* From original Douyin .cate1-list */
}

.cate1-item {
  padding: 6px 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 13px;
  border-radius: 20px;
  white-space: nowrap;
  border: 1px solid transparent; 
  background: var(--cate1-item-bg-dark, rgba(255, 255, 255, 0.1));
  color: var(--cate1-item-text-dark, #e0e0e0); 
}

:root[data-theme="light"] .cate1-item {
  background-color: var(--control-bg-light, #f0f2f5);
  color: var(--control-text-light, #343a40);
  border-color: var(--control-border-light, #ced4da); 
}

.cate1-item:hover {
  background: var(--cate1-item-hover-bg-dark, rgba(255, 255, 255, 0.2));
  color: var(--primary-text, #FFFFFF); 
}

:root[data-theme="light"] .cate1-item:hover {
  background-color: var(--control-hover-bg-light, #e2e6ea);
  border-color: var(--control-hover-border-light, #adb5bd);
}

/* Styles for .selected item (Douyin uses .selected, Douyu styles used .active) */
.cate1-item.selected {
  background-color: var(--cate2-card-bg-dark, rgba(31, 31, 35, 1)); 
  border: 1px solid transparent; /* Removed border for night mode selected */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3));
  color: var(--douyu-cate2-active-text-dark-glow-revived, rgb(79, 209, 197));
  font-weight: 500;
}

:root[data-theme="light"] .cate1-item.selected {
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); 
  border-color: transparent; 
  box-shadow: var(--douyu-cate2-active-shadow-light-scheme-a, 0 2px 5px rgba(0,0,0,0.12)); 
  color: var(--douyu-cate2-active-text-light-scheme-a, #FFFFFF); 
  font-weight: 500;
}

.cate1-name { /* From original Douyin, seems fine */
  display: inline-block;
}
</style> 