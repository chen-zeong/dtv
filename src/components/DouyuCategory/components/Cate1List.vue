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
  background: var(--cate1-list-bg-dark, #1f1f23);
  flex-shrink: 0;
  transition: background-color 0.3s ease;
}

:root[data-theme="light"] .cate1-list {
  background-color: var(--primary-bg, #FFFFFF);
  color: var(--primary-text);
}

.cate1-item {
  padding: 6px 14px;
  cursor: pointer;
  transition: background 0.22s ease, border-color 0.22s ease, box-shadow 0.22s ease, color 0.22s ease;
  font-size: 13px;
  border-radius: 20px;
  white-space: nowrap;
  border: 1px solid rgba(90, 176, 255, 0.14);
  background: linear-gradient(165deg, rgba(22, 27, 36, 0.96), rgba(18, 22, 31, 0.92));
  color: rgba(227, 236, 255, 0.82);
}

:root[data-theme="light"] .cate1-item {
  background: rgba(248, 250, 255, 0.92);
  border-color: rgba(203, 213, 225, 0.6);
  color: #334155;
  box-shadow: none;
}

.cate1-item:hover {
  background: linear-gradient(165deg, rgba(32, 39, 50, 0.94), rgba(24, 29, 40, 0.92));
  border-color: rgba(125, 211, 252, 0.3);
  color: rgba(236, 244, 255, 0.92);
}

:root[data-theme="light"] .cate1-item:hover {
  background: rgba(241, 245, 255, 0.96);
  border-color: rgba(148, 163, 184, 0.65);
  color: #1f2f4d;
}

.cate1-item.active {
  background: linear-gradient(150deg, rgba(79, 209, 197, 0.35), rgba(59, 130, 246, 0.28));
  border: 1px solid rgba(125, 211, 252, 0.45);
  box-shadow: 0 12px 32px rgba(14, 116, 144, 0.32);
  color: rgba(222, 255, 250, 0.95);
  font-weight: 600;
}

:root[data-theme="light"] .cate1-item.active {
  background: #dbe7ff;
  border-color: #85a5ff;
  box-shadow: 0 0 0 2px rgba(133, 165, 255, 0.18);
  color: #1f3f85;
  font-weight: 600;
}
</style>
