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
  transition: background 0.22s ease, border-color 0.22s ease, box-shadow 0.22s ease, color 0.22s ease;
  font-size: 13px;
  border-radius: 20px;
  white-space: nowrap;
  border: 1px solid rgba(90, 176, 255, 0.14);
  background: #1e2022;
  color: rgba(227, 236, 255, 0.82);
}

:root[data-theme="light"] .cate1-item {
  background: rgba(248, 250, 255, 0.92);
  border-color: rgba(203, 213, 225, 0.6);
  color: #334155;
  box-shadow: none;
}

.cate1-item:hover {
  background: #272a2d;
  border-color: rgba(125, 211, 252, 0.3);
  color: rgba(236, 244, 255, 0.92);
}

:root[data-theme="light"] .cate1-item:hover {
  background: rgba(241, 245, 255, 0.96);
  border-color: rgba(148, 163, 184, 0.65);
  color: #1f2f4d;
}

.cate1-item.selected {
  background: linear-gradient(150deg, rgba(79, 209, 197, 0.35), rgba(59, 130, 246, 0.28));
  border: 1px solid rgba(125, 211, 252, 0.45);
  box-shadow: 0 12px 32px rgba(14, 116, 144, 0.32);
  color: rgba(222, 255, 250, 0.95);
  font-weight: 600;
}

:root[data-theme="light"] .cate1-item.selected {
  background: linear-gradient(140deg, rgba(229, 236, 255, 0.96), rgba(204, 219, 255, 0.94));
  border-color: rgba(114, 136, 255, 0.6);
  box-shadow: 0 12px 24px rgba(108, 130, 255, 0.18), inset 0 0 0 1px rgba(255, 255, 255, 0.48);
  color: #162a63;
  font-weight: 600;
}

.cate1-name {
  display: inline-block;
}
</style>
