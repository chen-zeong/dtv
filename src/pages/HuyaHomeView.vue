<template>
  <div class="huya-home-view-layout">
    <CommonCategory 
      :categories-data="huyaCategoriesData as any"
      @category-selected="onCategorySelected"
      class="huya-category-section"
    />
    <CommonStreamerList 
      :selected-category="currentSelectedCategory"
      :categories-data="huyaCategoriesData as any"
      :default-page-size="120"
      playerRouteName="huyaPlayer"
      class="huya-streamer-list-section"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import { huyaCategoriesData } from '../platforms/huya/huyaCategoriesData'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes.ts'

const currentSelectedCategory = ref<CategorySelectedEvent | null>(null)
const onCategorySelected = (categoryEvent: CategorySelectedEvent) => {
  currentSelectedCategory.value = categoryEvent
}
</script>

<style scoped>
.huya-home-view-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.huya-category-section {
  flex-shrink: 0;
}

.huya-streamer-list-section {
  flex-grow: 1;
  overflow-y: auto;
  min-height: 0;
}
</style>