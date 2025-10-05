<template>
  <div class="huya-home-view-layout">
    <CommonCategory 
      :categories-data="huyaCategoriesData as any"
      class="huya-category-section"
    />
    <div class="huya-streamer-list-section">
      <!-- 预留：虎牙主播列表（后续实现） -->
    </div>
  </div>
</template>

<script setup lang="ts">
import CommonCategory from '../components/CommonCategory/index.vue'
import { huyaCategoriesData } from '../platforms/huya/huyaCategoriesData'
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