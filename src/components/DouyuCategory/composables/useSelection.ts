import { ref } from 'vue'
import type { Category2, Category3 } from '../types'

export function useSelection(emit: (event: 'category-selected', ...args: any[]) => void) {
  const selectedCate1Id = ref<number | null>(null)
  const selectedCate2Id = ref<number | null>(null)
  const selectedCate3Id = ref<string | null>(null)
  const selectedCate2 = ref<Category2 | null>(null)

  const selectCate1 = (cate1Id: number) => {
    if (selectedCate1Id.value === cate1Id) {
      return
    }
    
    selectedCate1Id.value = cate1Id
    selectedCate2Id.value = null
    selectedCate2.value = null
    selectedCate3Id.value = null
  }

  const handleCate2Click = (cate2: Category2) => {
    if (selectedCate2Id.value === cate2.cate2Id) {
      return
    }
    
    selectedCate2Id.value = cate2.cate2Id
    selectedCate2.value = cate2
    selectedCate3Id.value = null // 默认选择"全部"
    
    emit('category-selected', {
      type: 'cate2',
      cate2Id: cate2.cate2Id,
      shortName: cate2.shortName,
      cate2Name: cate2.cate2Name
    })
  }

  const handleCate3Click = (cate3: Category3) => {
    if (!selectedCate2.value) {
      console.error('未选择二级分类，无法选择三级分类')
      return
    }
    
    // "全部"选项特殊处理
    if (cate3.id === 'all') {
      if (selectedCate3Id.value === null) {
        return
      }
      selectedCate3Id.value = null
      emit('category-selected', {
        type: 'cate2',
        cate2Id: selectedCate2.value.cate2Id,
        shortName: selectedCate2.value.shortName,
        cate2Name: selectedCate2.value.cate2Name
      })
      return
    }
    
    if (selectedCate3Id.value === cate3.id) {
      return
    }
    
    selectedCate3Id.value = cate3.id
    emit('category-selected', {
      type: 'cate3',
      cate2Id: selectedCate2.value.cate2Id,
      shortName: selectedCate2.value.shortName,
      cate2Name: selectedCate2.value.cate2Name,
      cate3Id: cate3.id,
      cate3Name: cate3.name
    })
  }

  // 提供一个重置选择的方法
  const resetSelection = () => {
    selectedCate1Id.value = null
    selectedCate2Id.value = null
    selectedCate2.value = null
    selectedCate3Id.value = null
  }

  return {
    selectedCate1Id,
    selectedCate2Id,
    selectedCate3Id,
    selectCate1,
    handleCate2Click,
    handleCate3Click,
    resetSelection
  }
}