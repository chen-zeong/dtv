<template>
  <div class="cate2-container-dy">
    <div
      class="cate2-content-dy"
      :class="{ 'is-expanded': isExpandedInternal, 'scrollable': isExpandedInternal && hasMoreRowsInternal }"
      ref="cate2ContentRef"
    >
      <div class="cate2-scroll-wrapper-dy" :class="{ 'allow-scroll': isExpandedInternal && hasMoreRowsInternal }">
        <div class="cate2-grid-dy" ref="cate2GridRef">
          <div
            v-for="cate2 in cate2List"
            :key="cate2.href" 
            class="cate2-card-dy"
            :class="{ 'active': selectedCate2Href === cate2.href }"
            @click="selectCate2(cate2)"
          >
            <div class="cate2-name-dy" :title="cate2.title">{{ cate2.title }}</div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="shouldShowExpandButtonInternal" class="expand-button-dy" @click="handleToggleInternalExpand">
      <span>{{ isExpandedInternal ? '收起' : '展开' }}</span>
      <svg
        class="expand-icon-dy"
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
import type { DouyinCategory2 } from '../types'

const props = defineProps<{
  cate2List: DouyinCategory2[]
  selectedCate2Href: string | null
  isExpanded: boolean // Prop from parent (DouyinCategory/index.vue)
  // hasMoreRows: boolean // This will be determined internally now
}>()

const emit = defineEmits<{
  (e: 'select', cate2: DouyinCategory2): void
  (e: 'toggle-expand'): void // To inform parent to update its isExpanded state
  (e: 'height-changed'): void // To inform parent if its layout might need adjustment
}>()

// Constants adapted from Douyu Cate2Grid, adjusted for new styles
const CARD_ACTUAL_HEIGHT = 36; // px, from .cate2-card-dy height
const GRID_VERTICAL_GAP = 12;  // Reverted: px, from .cate2-grid-dy gap
const CONTENT_PADDING_BOTTOM = 8; // px, from .cate2-content-dy padding-bottom
const GRID_INTERNAL_PADDING_BOTTOM = 16; // px, from .cate2-grid-dy padding-bottom

const TARGET_CONTENT_HEIGHT_FOR_ONE_ROW = CARD_ACTUAL_HEIGHT + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS = (2 * CARD_ACTUAL_HEIGHT + GRID_VERTICAL_GAP) + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const EXPANDED_CONTENT_MAX_ROWS = 7; // Match Douyu
const TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS = 
    (EXPANDED_CONTENT_MAX_ROWS * CARD_ACTUAL_HEIGHT + (EXPANDED_CONTENT_MAX_ROWS - 1) * GRID_VERTICAL_GAP) 
    + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;

const cate2ContentRef = ref<HTMLElement | null>(null)
const cate2GridRef = ref<HTMLElement | null>(null)
const isAnimating = ref(false)
const isExpandedInternal = ref(props.isExpanded) // Internal state mirroring parent's isExpanded
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
}, { deep: true });

watch(() => props.isExpanded, (newVal) => {
  if (isExpandedInternal.value !== newVal) {
    isExpandedInternal.value = newVal;
    animateHeightChange(newVal);
  }
});

onMounted(() => {
  isExpandedInternal.value = props.isExpanded; // Initialize internal state
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

  // 新增逻辑: 处理从 'auto' 高度收起的情况
  if (!targetExpandedState && content.style.height === 'auto') {
    // 1. 先将 'auto' 替换为当前的实际像素高度
    content.style.height = `${content.scrollHeight}px`;
    
    // 2. 强制浏览器重绘/回流 或 延迟到下一帧
    requestAnimationFrame(() => {
      // 3. 现在将高度设置为最终的收起目标值，这将触发动画
      content.style.height = `${targetHeightValue}px`;
    });
  } else {
    // 对于其他情况（展开，或从固定高度收起），直接设置目标高度
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
  // Emit event for parent to toggle its isExpanded state
  // The actual change and animation will be driven by the watch on props.isExpanded
  emit('toggle-expand'); 
};

const selectCate2 = (cate2: DouyinCategory2) => {
  emit('select', cate2)
  // No auto-collapse here, parent DouyinCategory/index.vue handles that if needed
}

</script>

<style scoped>
/* Styles for DouyinCate2Grid.vue, adapted from Douyu Cate2Grid.vue */
.cate2-container-dy { /* Douyin: .cate2-container-dy */
  padding: 14px 12px; /* MODIFIED: top/bottom 14px, left/right 12px */
  display: flex;
  flex-direction: column;
  flex: 1;
  position: relative;
  overflow: visible; /* Douyu: overflow: visible; */
  background: var(--cate2-grid-area-bg-dark, #18181b); /* Night mode Douyu Cate2 area BG */
}

:root[data-theme="light"] .cate2-container-dy {
  background-color: var(--primary-bg, #FFFFFF); /* Light mode for container */
}

.cate2-content-dy { /* Douyin: .cate2-content-dy */
  position: relative;
  height: 0;
  padding-bottom: 8px; /* MODIFIED from 10px back to 8px */
  overflow: hidden;
  transition: height 0.4s cubic-bezier(0.33, 0.66, 0.66, 1);
  will-change: height;
  box-sizing: border-box;
}

.cate2-content-dy.animating { /* Douyin: .cate2-content-dy (if class is used) */
  overflow: hidden !important;
}

/* Scroll wrapper styles - ensure class names match Douyin's template */
.cate2-scroll-wrapper-dy { /* Douyin: .cate2-scroll-wrapper-dy */
  max-height: 100%;
  height: 100%;
  overflow: hidden;
}

.cate2-content-dy.is-expanded .cate2-scroll-wrapper-dy.allow-scroll { /* Adjust if Douyin uses different classes for expanded/scrollable */
  overflow-y: auto !important;
  -ms-overflow-style: none !important;
  scrollbar-width: none !important;
}

.cate2-content-dy.is-expanded .cate2-scroll-wrapper-dy.allow-scroll::-webkit-scrollbar {
  display: none !important;
  width: 0 !important;
  height: 0 !important;
}

.cate2-content-dy:not(.is-expanded) .cate2-scroll-wrapper-dy {
   overflow: hidden !important;
}


.cate2-grid-dy { /* Douyin: .cate2-grid-dy */
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 120px));
  gap: 12px;
  justify-content: flex-start;
  padding-top: 2px; /* ADDED */
  padding-left: 2px; /* ADDED (or adjust if padding was from container) */
  padding-right: 2px; /* ADDED (or adjust if padding was from container) */
  padding-bottom: 18px; /* MODIFIED from 16px */
  /* Ensure grid itself also has horizontal padding if .cate2-container-dy has 0 horizontal padding */
  /* padding-left: 16px; */ /* Removed */
  /* padding-right: 16px; */ /* Removed */
}


.cate2-card-dy { /* Douyin: .cate2-card-dy */
  width: 120px; 
  height: var(--cate2-card-height, 36px); /* Douyu var */
  padding: 6px 8px; 
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
  box-sizing: border-box;
  overflow: hidden;
  text-align: center; 
  display: flex; 
  align-items: center; 
  gap: 8px; 
  background: var(--cate2-card-bg-dark, #252731); /* MODIFIED */
  border: 1px solid var(--cate2-card-border-dark, transparent); 
  color: var(--cate2-card-text-dark, #ccc); 
}

:root[data-theme="light"] .cate2-card-dy { /* Douyin: .cate2-card-dy */
  background-color: var(--content-card-bg-light, #FFFFFF); 
  border: 1px solid var(--content-card-border-light, #e0e0e0); 
  box-shadow: var(--content-card-shadow-light, 0 1px 3px rgba(0,0,0,0.08)); 
  color: var(--main-text-primary-light, #212529);
}

.cate2-card-dy:hover { /* Douyin: .cate2-card-dy */
  background-color: var(--cate2-card-hover-bg-dark, rgba(45, 48, 56, 1)); 
  border: 1px solid transparent;
  color: var(--cate2-card-text-dark-hover, #ddd); 
  box-shadow: 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3); /* --- MODIFIED for stronger glow --- */
}

:root[data-theme="light"] .cate2-card-dy:hover { /* Douyin: .cate2-card-dy */
  background-color: var(--content-card-hover-bg-light, #f8f9fa); 
  border: 1px solid transparent;
  box-shadow: var(--douyu-cate2-hover-shadow-light-glow, 0 0 0 2px rgba(50, 150, 255, 0.35), 0 0 9px rgba(50, 150, 255, 0.2)); 
}

.cate2-card-dy.active {  /* Douyin: .cate2-card-dy.active */
  background-color: var(--cate2-card-bg-dark, #252731); 
  border: 1px solid transparent; /* Removed border for night mode selected */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3)); 
}

/* This rule might be redundant if the one above has :root[data-theme="dark"] or is specific enough */
:root[data-theme="dark"] .cate2-card-dy.active { /* Douyin: .cate2-card-dy.active */
  background-color: var(--cate2-card-bg-dark, #252731); 
  border-color: transparent; /* Removed border for night mode selected */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3)); 
}

:root[data-theme="dark"] .cate2-card-dy.active .cate2-name-dy { /* Douyin: .cate2-name-dy */
  color: var(--douyu-cate2-active-text-dark-glow-revived, rgb(79, 209, 197)); 
}

/* Icon style for active dark - check if .cate2-icon-dy exists or if it's direct img */
:root[data-theme="dark"] .cate2-card-dy.active img { /* Assuming img is direct child or within a generic container */
  filter: none; 
}

:root[data-theme="dark"] .cate2-card-dy.active:hover { /* Douyin: .cate2-card-dy.active */
  background-color: var(--cate2-card-bg-dark, #252731); /* MODIFIED */
  border-color: transparent;
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived-hover, 0 0 0 2.5px rgba(79, 209, 197, 0.55), 0 0 12px rgba(79, 209, 197, 0.4)); 
}

:root[data-theme="light"] .cate2-card-dy.active { /* Douyin: .cate2-card-dy.active */
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); 
  border-color: transparent; 
  box-shadow: var(--douyu-cate2-active-shadow-light-scheme-a, 0 2px 5px rgba(0,0,0,0.12)); 
}

:root[data-theme="light"] .cate2-card-dy.active .cate2-name-dy { /* Douyin: .cate2-name-dy */
  color: var(--douyu-cate2-active-text-light-scheme-a, #FFFFFF);
}

/* Icon style for active light - check if .cate2-icon-dy exists */
:root[data-theme="light"] .cate2-card-dy.active img {  /* Assuming img is direct child or within a generic container */
  filter: none; 
}

:root[data-theme="light"] .cate2-card-dy.active:hover { /* Douyin: .cate2-card-dy.active */
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); 
  border: 1px solid transparent;
  box-shadow: var(--content-card-hover-shadow-light-larger, 0 5px 12px rgba(0,0,0,0.15)); 
}

.cate2-name-dy { /* Douyin: .cate2-name-dy */
  font-size: 14px; /* Douyu: .cate2-name */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2; /* Douyu: .cate2-name */
  flex: 1; /* Added to allow text to take available space and ellipsize */
  text-align: left; /* Usually better for category names */
}

.expand-button-dy { /* Douyin: .expand-button-dy */
  position: absolute;
  bottom: 0;
  left: 16px; 
  right: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 0;
  font-size: 12px;
  height: 28px;
  box-sizing: border-box;
  cursor: pointer;
  transition: color 0.2s ease, background-color 0.3s ease, border-color 0.3s ease;
  background: var(--douyu-cate2-expand-btn-bg-dark, #18181b);
  color: var(--douyu-cate2-expand-btn-text-dark, rgba(255, 255, 255, 0.6));
  border-top: 1px solid var(--douyu-cate2-expand-btn-border-dark, rgba(255, 255, 255, 0.1));
  z-index: 10;
}

.expand-button-dy:hover { /* Douyin: .expand-button-dy */
  color: var(--douyu-cate2-expand-btn-hover-text-dark, #ffffff);
}

:root[data-theme="light"] .expand-button-dy { /* Douyin: .expand-button-dy */
  background: var(--douyu-cate2-expand-btn-bg-light, var(--primary-bg, #ffffff));
  color: var(--douyu-cate2-expand-btn-text-light, var(--secondary-text, #4b5563));
  border-top: 1px solid var(--douyu-cate2-expand-btn-border-light, var(--border-color-light, #e5e7eb));
}

:root[data-theme="light"] .expand-button-dy:hover { /* Douyin: .expand-button-dy */
  color: var(--douyu-cate2-expand-btn-hover-text-light, var(--primary-text, #1f2937));
}

.expand-icon-dy { /* Douyin: .expand-icon-dy */
  margin-left: 4px;
  transition: transform 0.4s cubic-bezier(0.33, 0.66, 0.66, 1);
  width: 12px; /* Original Douyin was 12px, Douyu was 16px, keeping 12px */
  height: 12px;
}

/* Stroke colors for expand icon - Douyin uses .expand-icon-dy */
:root[data-theme="dark"] .expand-button-dy .expand-icon-dy {
  stroke: var(--douyu-cate2-expand-btn-text-dark, rgba(255, 255, 255, 0.6));
}
:root[data-theme="dark"] .expand-button-dy:hover .expand-icon-dy {
  stroke: var(--douyu-cate2-expand-btn-hover-text-dark, #ffffff);
}

:root[data-theme="light"] .expand-button-dy .expand-icon-dy {
  stroke: var(--douyu-cate2-expand-btn-text-light, var(--secondary-text, #4b5563));
}
:root[data-theme="light"] .expand-button-dy:hover .expand-icon-dy {
  stroke: var(--douyu-cate2-expand-btn-hover-text-light, var(--primary-text, #1f2937));
}

/* Additional styles from Douyu that might be needed if Douyin lacks them */
/* If Douyin cards DO have images like <img src="..."/> directly inside .cate2-card-dy: */
.cate2-card-dy img {
  width: 24px; /* Match Douyu's .cate2-icon img */
  height: 24px; /* Match Douyu's .cate2-icon img */
  object-fit: cover; 
  border-radius: 4px; 
  transition: filter 0.2s ease;
}

/* Ensure expand icon rotates */
.expand-icon-dy.is-expanded {
  transform: rotate(180deg);
}

</style> 