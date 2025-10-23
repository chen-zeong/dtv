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
  padding: 4px 12px;
  height: 30px;
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease, color 0.2s ease;
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  font-size: 13px;
  background: linear-gradient(170deg, rgba(23, 28, 38, 0.95), rgba(17, 21, 30, 0.9));
  border: 1px solid rgba(90, 176, 255, 0.12);
  color: rgba(226, 232, 240, 0.86);
}

.cate3-item:hover {
  background: linear-gradient(170deg, rgba(30, 37, 50, 0.95), rgba(23, 29, 41, 0.9));
  border-color: rgba(125, 211, 252, 0.32);
  color: rgba(241, 245, 249, 0.95);
  box-shadow: 0 10px 20px rgba(10, 20, 38, 0.38);
}

.cate3-item.active {
  background: linear-gradient(150deg, rgba(79, 209, 197, 0.35), rgba(59, 130, 246, 0.28));
  border-color: rgba(125, 211, 252, 0.45);
  color: rgba(222, 255, 250, 0.95);
  box-shadow: 0 12px 28px rgba(15, 118, 110, 0.32);
  font-weight: 600;
}

:root[data-theme="light"] .cate3-item {
  background: rgba(248, 250, 255, 0.9);
  border: 1px solid rgba(203, 213, 225, 0.65);
  color: #334155;
  box-shadow: none;
}

:root[data-theme="light"] .cate3-item:hover {
  background: rgba(241, 245, 255, 0.96);
  border-color: rgba(148, 163, 184, 0.7);
  color: #1f2f4d;
}

:root[data-theme="light"] .cate3-item.active {
  background: #dbe7ff;
  border-color: #85a5ff;
  box-shadow: 0 0 0 2px rgba(133, 165, 255, 0.18);
  color: #1f3f85;
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
