import { ref, computed, Ref, watch } from 'vue'
import type { Category2 } from '../types'

const EXPANDED_ROWS = 7   // 展开状态下显示的行数

export function useExpand(sortedCate2List: Ref<Category2[]>) {
  const isExpanded = ref(false)
  
  // 计算是否有足够多的分类，使得展开时可能需要滚动（基于总行数估算）
  const hasMoreRows = computed(() => {
    const estimatedItemsPerRow = 5; // 估计每行平均显示5个 (This is a rough estimate)
    return sortedCate2List.value.length > EXPANDED_ROWS * estimatedItemsPerRow
  })
  
  // 切换展开状态
  const toggleExpand = () => {
    isExpanded.value = !isExpanded.value
  }
  
  // 重置展开状态为折叠
  const resetExpand = () => {
    isExpanded.value = false
  }
  
  // 监听分类列表变化，如果项目数变得很少，可能不再需要展开状态
  watch(sortedCate2List, (newList) => {
    if (newList.length === 0 && isExpanded.value) {
      isExpanded.value = false
    }
  })
  
  return {
    isExpanded,
    hasMoreRows,
    toggleExpand,
    resetExpand,
    EXPANDED_ROWS
  }
}