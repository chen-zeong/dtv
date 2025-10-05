<template>
  <div class="douyin-home">
    <div class="douyin-content">
      <div class="left-panel">
        <CommonCategory 
          :categoriesData="categoriesData"
          @category-selected="onCategorySelected" 
        />
      </div>
      <div class="right-panel">
        <CommonStreamerList
          :selectedCategory="selectedCategory"
          :categoriesData="categoriesData"
          platformName="douyin"
          playerRouteName="douyinPlayer"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import { douyinCategoriesData } from '../platforms/douyin/douyinCategoriesData'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes'

const categoriesData = douyinCategoriesData
const selectedCategory = ref<CategorySelectedEvent | null>(null)

function onCategorySelected(evt: CategorySelectedEvent) {
  selectedCategory.value = evt
}
</script>

<style scoped>
.douyin-home {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.douyin-content {
  display: flex;
  flex-direction: column; /* 改为纵向排列，上下布局 */
  height: 100%;
}

.left-panel {
  width: 100%; /* 顶部区域占满宽度 */
  border-bottom: 1px solid var(--border-color-light); /* 改为底部分隔线 */
  overflow: hidden;
}

.right-panel {
  flex: 1; /* 下方列表区域填满剩余空间 */
  overflow: hidden;
}
</style>