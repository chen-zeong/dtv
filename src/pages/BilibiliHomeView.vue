<template>
  <div class="bili-home-view-layout">
    <CommonCategory 
      :categories-data="biliCategoriesData as any"
      class="bili-category-section"
    />
    <div class="bili-streamer-list-section">
      <!-- 预留：B站主播列表（后续实现） -->
    </div>
  </div>
</template>

<script setup lang="ts">
import CommonCategory from '../components/CommonCategory/index.vue'
import { biliCategoriesData } from '../platforms/bilibili/biliCategoriesData'
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