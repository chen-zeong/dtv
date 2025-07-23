<template>
  <div class="category-list" :class="{ 'is-expanded': isExpanded }" ref="categoryListRootRef">
    <template v-if="cate1List.length > 0">
      <DouyinCate1List
        :cate1-list="cate1List"
        :selected-cate1-href="selectedCate1Href"
        @select="selectCate1"
      />
      <DouyinCate2Grid
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
import DouyinCate1List from './components/DouyinCate1List.vue'
import DouyinCate2Grid from './components/DouyinCate2Grid.vue'
import { douyinCategoriesData } from '../../platforms/douyin/douyinCategoriesData'
import type { DouyinCategory1, DouyinCategory2, DouyinCategorySelectedEvent } from './types'

const emit = defineEmits<{
  (e: 'category-selected', category: DouyinCategorySelectedEvent): void
  (e: 'expanded-state-changed', isExpanded: boolean): void
  (e: 'category-section-height-settled'): void
}>()

const categoryListRootRef = ref<HTMLElement | null>(null)

const cate1List = ref<DouyinCategory1[]>([])
const selectedCate1Href = ref<string | null>(null)
const selectedCate2Href = ref<string | null>(null)

const isExpanded = ref(false) // Default to collapsed

// Simulate fetching categories
onMounted(() => {
  cate1List.value = douyinCategoriesData as DouyinCategory1[]
  if (cate1List.value.length > 0) {
    // Auto-select the first category1 if not already selected
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
  const selectedCate1 = cate1List.value.find(c1 => c1.href === selectedCate1Href.value)
  return selectedCate1 ? selectedCate1.subcategories : []
})

const selectCate1 = (cate1: DouyinCategory1) => {
  if (selectedCate1Href.value === cate1.href) return; // Avoid re-selecting the same category
  selectedCate1Href.value = cate1.href
  selectedCate2Href.value = null // Reset cate2 selection

  // Auto-select the first cate2 if available
  if (currentCate2List.value.length > 0) {
    handleCate2SelectAndCollapse(currentCate2List.value[0])
  }
  
  // If the list was expanded, collapse it when a new C1 is selected.
  if (isExpanded.value) {
    toggleExpand()
  }
  nextTick(() => {
    emit('category-section-height-settled')
  })
}

const handleCate2Select = (cate2: DouyinCategory2) => {
  selectedCate2Href.value = cate2.href
  const selectedCate1 = cate1List.value.find(c1 => c1.href === selectedCate1Href.value)
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

const handleCate2SelectAndCollapse = (cate2: DouyinCategory2) => {
  handleCate2Select(cate2)
  if (isExpanded.value) {
    toggleExpand() // Collapse after selection if expanded
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
  emit('category-section-height-settled'); // Propagate this event if HomeView needs it
}

onActivated(() => {
  // When component is activated from keep-alive,
  // re-emit the current selection so parent can re-render list if necessary.
  const currentSelectedCate1 = cate1List.value.find(c1 => c1.href === selectedCate1Href.value);
  // Find in currentCate2List as it's already filtered for the selectedCate1Href
  const currentSelectedCate2 = currentCate2List.value.find(c2 => c2.href === selectedCate2Href.value);

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
        handleCate2SelectAndCollapse(currentCate2List.value[0]); // This will emit 'category-selected'
    }
  }
  // Ensure layout recalculates if necessary
  nextTick(() => {
    emit('category-section-height-settled');
  });
});

</script>

<style scoped>
.category-list {
  display: flex;
  flex-direction: column;
  background: var(--component-bg); /* Night mode default */
  color: var(--text-color); /* Night mode default */
  max-height: 280px; /* Default max height from Douyu */
  min-height: 200px; /* Default min height from Douyu */
  overflow: hidden;
  transition: max-height 0.4s cubic-bezier(0.33, 0.66, 0.66, 1), background-color 0.3s ease, color 0.3s ease; /* Douyu transition */
  width: 100%;
}

:root[data-theme="light"] .category-list {
  background-color: var(--main-bg-light, #FFFFFF);
  color: var(--main-text-primary-light, #212529);
}

.category-list.is-expanded {
  max-height: 500px; /* Douyu expanded max height */
}

.loading-state {
  padding: 40px 20px;
  text-align: center;
  color: var(--secondary-text); /* Use theme variable */
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

:root[data-theme="light"] .loading-state {
  color: var(--main-text-secondary-light, #495057);
}

/* Re-add loading spinner styles if you have one, or adapt from Douyu */
.loading-spinner {
  width: 30px;
  height: 30px;
  border: 3px solid rgba(128, 128, 128, 0.2); /* Adjusted for theming */
  border-top-color: var(--primary-text); /* Use theme variable for spinner highlight */
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 15px;
}

:root[data-theme="light"] .loading-spinner {
  border-color: var(--border-color-light-softer, rgba(0,0,0,0.1)); /* Softer track for light */
  border-top-color: var(--accent-color-light, #007bff); /* Accent for spinner highlight */
}

.loading-text {
  font-size: 14px;
  color: var(--secondary-text); /* Use theme variable */
}

:root[data-theme="light"] .loading-text {
  color: var(--main-text-secondary-light, #495057);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Styles for error state can be added if needed, similar to Douyu's .error-state */

</style> 