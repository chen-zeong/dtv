<template>
  <div class="cate2-container">
    <div
      class="cate2-content"
      :class="{ 'is-expanded': isExpandedInternal, 'scrollable': isExpandedInternal && hasMoreRowsInternal, 'animating': isAnimating }"
      ref="cate2ContentRef"
    >
      <div class="cate2-scroll-wrapper" :class="{ 'allow-scroll': isExpandedInternal && hasMoreRowsInternal }">
        <div class="cate2-grid" ref="cate2GridRef">
          <div
            v-for="cate2 in cate2List"
            :key="cate2.href"
            class="cate2-card"
            :class="{ 'active': selectedCate2Href === cate2.href }"
            @click="$emit('select', cate2)"
          >
            <div class="cate2-name" :title="cate2.title">{{ cate2.title }}</div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="shouldShowExpandButtonInternal" class="expand-button" @click="handleToggleInternalExpand">
      <span>{{ isExpandedInternal ? '收起' : '展开' }}</span>
      <svg
        class="expand-icon"
        :class="{ 'is-expanded': isExpandedInternal }"
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
      >
        <path d="M6 9l6 6 6-6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed, nextTick } from 'vue'
import type { Category2 } from '../../../platforms/common/categoryTypes.ts'

const props = defineProps<{
  cate2List: Category2[]
  selectedCate2Href: string | null
  isExpanded: boolean
}>()

const emit = defineEmits<{
  (e: 'select', cate2: Category2): void
  (e: 'toggle-expand'): void
  (e: 'height-changed'): void
}>()

const CARD_ACTUAL_HEIGHT = 36;
const GRID_VERTICAL_GAP = 12;
const CONTENT_PADDING_BOTTOM = 8;
const GRID_INTERNAL_PADDING_BOTTOM = 18;

const TARGET_CONTENT_HEIGHT_FOR_ONE_ROW = CARD_ACTUAL_HEIGHT + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS = (2 * CARD_ACTUAL_HEIGHT + GRID_VERTICAL_GAP) + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM - 6;
const EXPANDED_CONTENT_MAX_ROWS = 7;
const TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS = 
    (EXPANDED_CONTENT_MAX_ROWS * CARD_ACTUAL_HEIGHT + (EXPANDED_CONTENT_MAX_ROWS - 1) * GRID_VERTICAL_GAP) 
    + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;

const cate2ContentRef = ref<HTMLElement | null>(null)
const cate2GridRef = ref<HTMLElement | null>(null)
const isAnimating = ref(false)
const isExpandedInternal = ref(props.isExpanded)
const actualGridScrollHeight = ref(0)
const hasMoreRowsInternal = ref(false)

const refreshHeightNonAnimated = () => {
  if (cate2ContentRef.value) {
    cate2ContentRef.value.style.height = `${getCurrentTargetHeight(isExpandedInternal.value)}px`;
    nextTick(() => emit('height-changed'));
  }
};

const updateActualGridScrollHeightAndMoreRows = () => {
  nextTick(() => {
    if (cate2GridRef.value) {
      actualGridScrollHeight.value = cate2GridRef.value.scrollHeight;
    } else {
      actualGridScrollHeight.value = GRID_INTERNAL_PADDING_BOTTOM;
    }
    hasMoreRowsInternal.value = requiredHeightForAllGridItemsWithPadding.value > TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS;
    refreshHeightNonAnimated();
  });
};

watch(() => props.cate2List, () => {
  updateActualGridScrollHeightAndMoreRows();
  animateHeightChange(isExpandedInternal.value);
}, { deep: true });

watch(() => props.isExpanded, (newVal) => {
  if (isExpandedInternal.value !== newVal) {
    isExpandedInternal.value = newVal;
    animateHeightChange(newVal);
  }
});

onMounted(() => {
  isExpandedInternal.value = props.isExpanded;
  updateActualGridScrollHeightAndMoreRows();
});

const requiredHeightForAllGridItemsWithPadding = computed(() => {
  return actualGridScrollHeight.value + CONTENT_PADDING_BOTTOM;
});

const shouldShowExpandButtonInternal = computed(() => {
  return requiredHeightForAllGridItemsWithPadding.value > TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS;
});

const getCurrentTargetHeight = (expandedState: boolean) => {
  const naturalContentHeight = requiredHeightForAllGridItemsWithPadding.value;
  if (expandedState) {
    if (hasMoreRowsInternal.value) {
      return TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS;
    }
    return props.cate2List.length > 0 ? naturalContentHeight : GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM; 
  } else {
    if (naturalContentHeight <= TARGET_CONTENT_HEIGHT_FOR_ONE_ROW) {
      return naturalContentHeight;
    }
    return TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS;
  }
};

const animateHeightChange = (targetExpandedState: boolean) => {
  if (!cate2ContentRef.value) return;
  isAnimating.value = true;
  const content = cate2ContentRef.value;
  const targetHeightValue = getCurrentTargetHeight(targetExpandedState);

  if (!targetExpandedState && content.style.height === 'auto') {
    content.style.height = `${content.scrollHeight}px`;
    requestAnimationFrame(() => {
      content.style.height = `${targetHeightValue}px`;
    });
  } else {
    content.style.height = `${targetHeightValue}px`;
  }

  const onTransitionEnd = () => {
    content.removeEventListener('transitionend', onTransitionEnd);
    isAnimating.value = false;
    if (targetExpandedState && !hasMoreRowsInternal.value && props.cate2List.length > 0) {
        const originalTransition = content.style.transition;
        content.style.transition = 'none';
        content.style.height = 'auto';
        requestAnimationFrame(() => {
            content.style.transition = originalTransition;
        });
    } else if (!targetExpandedState && props.cate2List.length === 0) {
        content.style.height = `${GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM}px`;
    }
    emit('height-changed');
  };
  content.addEventListener('transitionend', onTransitionEnd);
  setTimeout(() => { 
    if (isAnimating.value) {
      onTransitionEnd();
    }
  }, 450); 
};

const handleToggleInternalExpand = () => {
  if (isAnimating.value) return;
  emit('toggle-expand'); 
};
</script>

<style scoped>
.cate2-container {
  padding: 14px 12px;
  display: flex;
  flex-direction: column;
  flex: 1;
  position: relative;
  overflow: visible;
  background: var(--cate2-grid-area-bg-dark, #18181b);
}

:root[data-theme="light"] .cate2-container {
  background-color: var(--primary-bg, #FFFFFF);
}

.cate2-content {
  position: relative;
  height: 0;
  padding-bottom: 8px;
  overflow: hidden;
  transition: height 0.4s cubic-bezier(0.33, 0.66, 0.66, 1);
  will-change: height;
  box-sizing: border-box;
}

.cate2-scroll-wrapper {
  max-height: 100%;
  height: 100%;
  overflow: hidden;
}

.cate2-content.is-expanded .cate2-scroll-wrapper.allow-scroll {
  overflow-y: auto !important;
  -ms-overflow-style: none !important;
  scrollbar-width: none !important;
}

.cate2-content.is-expanded .cate2-scroll-wrapper.allow-scroll::-webkit-scrollbar {
  display: none !important;
  width: 0 !important;
  height: 0 !important;
}

.cate2-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 120px));
  gap: 12px;
  justify-content: flex-start;
  padding-top: 2px;
  padding-left: 2px;
  padding-right: 2px;
  padding-bottom: 18px;
}

.cate2-card {
  width: 120px;
  height: var(--cate2-card-height, 36px);
  padding: 6px 8px;
  border-radius: 12px;
  cursor: pointer;
  transition: background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease, color 0.2s ease;
  box-sizing: border-box;
  overflow: hidden;
  text-align: center;
  display: flex;
  align-items: center;
  gap: 8px;
  background: #1e2022;
  border: 1px solid rgba(90, 176, 255, 0.12);
  box-shadow: 0 8px 22px rgba(3, 8, 20, 0.42);
  color: rgba(226, 232, 240, 0.86);
}

.cate2-card:hover {
  background: #25282b;
  border-color: rgba(125, 211, 252, 0.32);
  color: rgba(241, 245, 249, 0.95);
  box-shadow: 0 12px 26px rgba(10, 20, 38, 0.44);
}

.cate2-card.active {
  background: linear-gradient(150deg, rgba(79, 209, 197, 0.35), rgba(59, 130, 246, 0.28));
  border-color: rgba(125, 211, 252, 0.45);
  color: rgba(222, 255, 250, 0.95);
  box-shadow: 0 14px 34px rgba(15, 118, 110, 0.35);
}

.cate2-card.active .cate2-name {
  color: rgba(222, 255, 250, 0.95);
}

:root[data-theme="light"] .cate2-card {
  background: rgba(248, 250, 255, 0.9);
  border: 1px solid rgba(203, 213, 225, 0.65);
  box-shadow: none;
  color: #334155;
}

:root[data-theme="light"] .cate2-card:hover {
  background: rgba(241, 245, 255, 0.96);
  border-color: rgba(148, 163, 184, 0.7);
  color: #1f2f4d;
}

:root[data-theme="light"] .cate2-card.active {
  background: #dbe7ff;
  border-color: #85a5ff;
  box-shadow: 0 0 0 2px rgba(133, 165, 255, 0.18);
  color: #1f3f85;
}

:root[data-theme="light"] .cate2-card:not(.active) .cate2-name {
  color: #31415d;
}
:root[data-theme="light"] .cate2-card.active .cate2-name {
  color: #1f3f85;
}
.cate2-name {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
  flex: 1;
  text-align: left;
}

.expand-button {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  min-width: 88px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 6px 14px;
  font-size: 12px;
  height: 28px;
  box-sizing: border-box;
  cursor: pointer;
  transition: color 0.2s ease, background-color 0.3s ease, border-color 0.3s ease;
  background: rgba(26, 29, 37, 0.92);
  color: rgba(226, 232, 240, 0.72);
  border: 1px solid rgba(125, 211, 252, 0.12);
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(8, 16, 28, 0.32);
  z-index: 10;
}

.expand-button:hover {
  color: rgba(248, 250, 252, 0.95);
  background: rgba(32, 37, 48, 0.96);
  border-color: rgba(125, 211, 252, 0.3);
}

:root[data-theme="light"] .expand-button {
  background: #f4f7fd;
  color: var(--douyu-cate2-expand-btn-text-light, var(--secondary-text, #4b5563));
  border: 1px solid rgba(148, 163, 184, 0.28);
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.1);
}

:root[data-theme="light"] .expand-button:hover {
  color: var(--douyu-cate2-expand-btn-hover-text-light, var(--primary-text, #1f2937));
  background: #e5ecfe;
}

.expand-icon {
  margin-left: 4px;
  transition: transform 0.4s cubic-bezier(0.33, 0.66, 0.66, 1);
  width: 12px;
  height: 12px;
}

.expand-icon.is-expanded {
  transform: rotate(180deg);
}

.cate2-card img {
  width: 24px;
  height: 24px;
  object-fit: cover;
  border-radius: 4px;
  transition: filter 0.2s ease;
}
.cate2-content.animating {
  overflow: hidden !important;
}
</style>
