<template>
  <div class="bili-home-view-layout">
    <CommonCategory 
      :categories-data="biliCategoriesData as any"
      @category-selected="onCategorySelected"
      class="bili-category-section"
    />
    <CommonStreamerList 
      :selected-category="currentSelectedCategory"
      :categories-data="biliCategoriesData as any"
      platformName="bilibili"
      playerRouteName="bilibiliPlayer"
      class="bili-streamer-list-section"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import { biliCategoriesData } from '../platforms/bilibili/biliCategoriesData'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes.ts'

const currentSelectedCategory = ref<CategorySelectedEvent | null>(null)
const onCategorySelected = (categoryEvent: CategorySelectedEvent) => {
  currentSelectedCategory.value = categoryEvent
}
</script>

<style scoped>
.bili-home-view-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.bili-category-section {
  flex-shrink: 0;
}

.bili-streamer-list-section {
  flex-grow: 1;
  overflow-y: auto;
  min-height: 0;
}
</style>