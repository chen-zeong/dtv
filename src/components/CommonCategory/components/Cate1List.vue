<template>
  <div class="cate1-list-container">
    <ul class="cate1-list">
      <li
        v-for="cate1 in cate1List"
        :key="cate1.href"
        class="cate1-item"
        :class="{ selected: cate1.href === selectedCate1Href }"
        @click="$emit('select', cate1)"
      >
        <span class="cate1-name">{{ cate1.title }}</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import type { Category1 } from '../../../platforms/common/categoryTypes.ts'

defineProps<{
  cate1List: Category1[]
  selectedCate1Href: string | null
}>()
</script>

<style scoped>
.cate1-list-container {
  overflow-x: auto;
  background: var(--cate1-list-bg-dark, #1f1f23);
  flex-shrink: 0;
  transition: background-color 0.3s ease;
}

:root[data-theme="light"] .cate1-list-container {
  background-color: var(--primary-bg, #FFFFFF);
}

.cate1-list {
  list-style: none;
  margin: 0;
  padding: 12px;
  display: flex;
  gap: 12px;
  flex-wrap: nowrap;
}

.cate1-item {
  padding: 6px 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 13px;
  border-radius: 20px;
  white-space: nowrap;
  border: 1px solid rgba(125, 211, 252, 0.08);
  background-color: #1f222c;
  color: var(--cate1-item-text-dark, #e0e0e0);
}

:root[data-theme="light"] .cate1-item {
  background-color: #f3f5fc;
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

.cate1-item.selected {
  background-color: rgba(125, 211, 252, 0.16);
  border: 1px solid rgba(125, 211, 252, 0.28);
  box-shadow: 0 0 0 1px rgba(125, 211, 252, 0.22), 0 0 12px rgba(125, 211, 252, 0.28);
  color: var(--douyu-cate2-active-text-dark-glow-revived, rgb(79, 209, 197));
  font-weight: 500;
}

:root[data-theme="light"] .cate1-item.selected {
  background-color: #d7e4ff;
  border-color: #96b7ff;
  box-shadow: 0 6px 16px rgba(67, 103, 190, 0.24);
  color: #1f3b7a;
  font-weight: 500;
}

.cate1-name {
  display: inline-block;
}
</style>
