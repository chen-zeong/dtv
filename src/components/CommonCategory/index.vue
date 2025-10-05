<template>
  <div class="category-list" :class="{ 'is-expanded': isExpanded }" ref="categoryListRootRef">
    <template v-if="cate1List.length > 0">
      <Cate1List
        :cate1-list="cate1List"
        :selected-cate1-href="selectedCate1Href"
        @select="selectCate1"
      />
      <Cate2Grid
        v-if="currentCate2List.length > 0"
        :cate2-list="currentCate2List"
        :selected-cate2-href="selectedCate2Href"
        :is-expanded="isExpanded"
        @select="handleCate2SelectAndCollapse"
        @toggle-expand="toggleExpand"
        @height-changed="handleCate2GridHeightChanged"
      />
    </template>
    <div v-else class="loading-state">
      <div class="loading-text">正在加载分类数据...</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick, onActivated } from 'vue'
import Cate1List from './components/Cate1List.vue'
import Cate2Grid from './components/Cate2Grid.vue'
import type { Category1 as CommonCategory1, Category2 as CommonCategory2, CategorySelectedEvent } from '../../platforms/common/categoryTypes.ts'

const props = defineProps<{ categoriesData: CommonCategory1[] }>()

const emit = defineEmits<{
  (e: 'category-selected', category: CategorySelectedEvent): void
  (e: 'expanded-state-changed', isExpanded: boolean): void
  (e: 'category-section-height-settled'): void
}>()

const categoryListRootRef = ref<HTMLElement | null>(null)

const cate1List = ref<CommonCategory1[]>([])
const selectedCate1Href = ref<string | null>(null)
const selectedCate2Href = ref<string | null>(null)

const isExpanded = ref(false)

onMounted(() => {
  cate1List.value = Array.isArray(props.categoriesData) ? props.categoriesData : []
  if (cate1List.value.length > 0) {
    if (!selectedCate1Href.value) {
      selectCate1(cate1List.value[0])
    }
  }
  nextTick(() => {
    emit('category-section-height-settled')
  })
})

const currentCate2List = computed(() => {
  if (!selectedCate1Href.value) return []
  const selectedCate1 = cate1List.value.find((c1: CommonCategory1) => c1.href === selectedCate1Href.value)
  return selectedCate1 ? selectedCate1.subcategories : []
})

const selectCate1 = (cate1: CommonCategory1) => {
  if (selectedCate1Href.value === cate1.href) return;
  selectedCate1Href.value = cate1.href
  selectedCate2Href.value = null

  if (currentCate2List.value.length > 0) {
    handleCate2SelectAndCollapse(currentCate2List.value[0])
  }
  if (isExpanded.value) {
    toggleExpand()
  }
  nextTick(() => {
    emit('category-section-height-settled')
  })
}

const handleCate2Select = (cate2: CommonCategory2) => {
  selectedCate2Href.value = cate2.href
  const selectedCate1 = cate1List.value.find((c1: CommonCategory1) => c1.href === selectedCate1Href.value)
  if (selectedCate1) {
    emit('category-selected', {
      type: 'cate2',
      cate1Href: selectedCate1.href,
      cate2Href: cate2.href,
      cate1Name: selectedCate1.title,
      cate2Name: cate2.title,
    })
  }
}

onActivated(() => {
  const currentSelectedCate1 = cate1List.value.find((c1: CommonCategory1) => c1.href === selectedCate1Href.value);
  const currentSelectedCate2 = currentCate2List.value.find((c2: CommonCategory2) => c2.href === selectedCate2Href.value);

  if (currentSelectedCate1 && currentSelectedCate2) {
    emit('category-selected', {
      type: 'cate2',
      cate1Href: currentSelectedCate1.href,
      cate2Href: currentSelectedCate2.href,
      cate1Name: currentSelectedCate1.title,
      cate2Name: currentSelectedCate2.title,
    });
  } else if (currentSelectedCate1 && !selectedCate2Href.value) {
    if (currentCate2List.value.length > 0) {
        handleCate2SelectAndCollapse(currentCate2List.value[0]);
    }
  }
  nextTick(() => {
    emit('category-section-height-settled');
  });
})
const handleCate2SelectAndCollapse = (cate2: CommonCategory2) => {
  handleCate2Select(cate2)
  if (isExpanded.value) {
    toggleExpand()
  }
}

const toggleExpand = () => {
  isExpanded.value = !isExpanded.value
  emit('expanded-state-changed', isExpanded.value)
  nextTick(() => {
    emit('category-section-height-settled')
  })
}

const handleCate2GridHeightChanged = () => {
  emit('category-section-height-settled')
}
</script>

<style scoped>
.category-list {
  display: flex;
  flex-direction: column;
  background: var(--component-bg);
  color: var(--text-color);
  max-height: 280px;
  min-height: 200px;
  overflow: hidden;
  transition: max-height 0.4s cubic-bezier(0.33, 0.66, 0.66, 1), background-color 0.3s ease, color 0.3s ease;
  width: 100%;
}

:root[data-theme="light"] .category-list {
  background-color: var(--main-bg-light, #FFFFFF);
  color: var(--main-text-primary-light, #212529);
}

.category-list.is-expanded {
  max-height: 500px;
}

.loading-state {
  padding: 40px 20px;
  text-align: center;
  color: var(--secondary-text);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

:root[data-theme="light"] .loading-state {
  color: var(--main-text-secondary-light, #495057);
}

.loading-spinner {
  width: 30px;
  height: 30px;
  border: 3px solid rgba(128, 128, 128, 0.2);
  border-top-color: var(--primary-text);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 15px;
}

:root[data-theme="light"] .loading-spinner {
  border-color: var(--border-color-light-softer, rgba(0,0,0,0.1));
  border-top-color: var(--accent-color-light, #007bff);
}

.loading-text {
  font-size: 14px;
  color: var(--secondary-text);
}

:root[data-theme="light"] .loading-text {
  color: var(--main-text-secondary-light, #495057);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>