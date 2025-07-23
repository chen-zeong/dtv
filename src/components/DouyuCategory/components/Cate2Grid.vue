<template>
  <div class="cate2-container">
    <div
      class="cate2-content"
      :class="{ 'is-expanded': isExpanded, 'scrollable': isExpanded && hasMoreRows }"
      ref="cate2ContentRef"
    >
      <div class="cate2-scroll-wrapper" :class="{ 'allow-scroll': isExpanded && hasMoreRows }">
        <div class="cate2-grid" ref="cate2GridRef">
          <div
            v-for="cate2 in cate2List"
            :key="cate2.cate2Id"
            class="cate2-card"
            :class="{ 'active': selectedCate2Id === cate2.cate2Id }"
            @click="$emit('select', cate2)"
          >
            <div class="cate2-icon">
              <img :src="cate2.icon" :alt="cate2.cate2Name">
            </div>
            <div class="cate2-info">
              <div class="cate2-name" :title="cate2.cate2Name">{{ formatCategoryName(cate2.cate2Name) }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="shouldShowExpandButtonComputed" class="expand-button" @click="handleToggleExpand">
      <span>{{ isExpanded ? '收起' : '展开' }}</span>
      <svg
        class="expand-icon"
        :class="{ 'is-expanded': isExpanded }"
        width="16"
        height="16"
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
import { ref, onMounted, watch, nextTick, computed } from 'vue'
import type { Category2 } from '../types'

const props = defineProps<{
  cate2List: Category2[]
  selectedCate2Id: number | null
  isExpanded: boolean
  hasMoreRows: boolean
}>()

const emit = defineEmits<{
  (e: 'select', cate2: Category2): void
  (e: 'toggle-expand'): void
  (e: 'height-changed'): void
}>()

// Constants for height calculation
const CARD_ACTUAL_HEIGHT = 36; // px, height of a .cate2-card
const GRID_VERTICAL_GAP = 12;  // px, gap between cards in the grid
const CONTENT_PADDING_BOTTOM = 8; // px, padding-bottom of .cate2-content
const GRID_INTERNAL_PADDING_BOTTOM = 16; // px, padding-bottom of .cate2-grid itself

const TARGET_CONTENT_HEIGHT_FOR_ONE_ROW = CARD_ACTUAL_HEIGHT + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS = (2 * CARD_ACTUAL_HEIGHT + GRID_VERTICAL_GAP) + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const EXPANDED_CONTENT_MAX_ROWS = 7;
const TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS = (EXPANDED_CONTENT_MAX_ROWS * CARD_ACTUAL_HEIGHT + (EXPANDED_CONTENT_MAX_ROWS - 1) * GRID_VERTICAL_GAP) + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;

const cate2ContentRef = ref<HTMLElement | null>(null)
const cate2GridRef = ref<HTMLElement | null>(null)
const isAnimating = ref(false)

const actualGridScrollHeight = ref(0)

// New function to apply height without animation, typically after content change
const refreshHeightNonAnimated = () => {
  if (cate2ContentRef.value) {
    cate2ContentRef.value.style.height = `${getCurrentTargetHeight(props.isExpanded)}px`;
    nextTick(() => emit('height-changed'));
  }
};

const updateActualGridScrollHeight = () => {
  nextTick(() => {
    if (cate2GridRef.value) {
      actualGridScrollHeight.value = cate2GridRef.value.scrollHeight;
    } else {
      actualGridScrollHeight.value = GRID_INTERNAL_PADDING_BOTTOM;
    }
    refreshHeightNonAnimated();
  });
};

watch(() => props.cate2List, () => {
  updateActualGridScrollHeight();
}, { deep: true });

onMounted(() => {
  updateActualGridScrollHeight();
});

const requiredHeightForAllGridItemsWithPadding = computed(() => {
  return actualGridScrollHeight.value + CONTENT_PADDING_BOTTOM;
});

const shouldShowExpandButtonComputed = computed(() => {
  return requiredHeightForAllGridItemsWithPadding.value > TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS;
});

const getCurrentTargetHeight = (expandedState: boolean) => {
  const naturalContentHeight = requiredHeightForAllGridItemsWithPadding.value;
  if (expandedState) {
    if (props.hasMoreRows) {
      return TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS;
    }
    // If not hasMoreRows (content is <= EXPANDED_ROWS), show all of it.
    // Ensure naturalContentHeight is at least a minimal value if list becomes empty while expanded.
    return props.cate2List.length > 0 ? naturalContentHeight : GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
  } else {
    // Collapsing
    // If naturalContentHeight (all items + padding) fits in one row's allowance or less.
    if (naturalContentHeight <= TARGET_CONTENT_HEIGHT_FOR_ONE_ROW) {
      return naturalContentHeight;
    }
    // Otherwise, if content needs more than one row, collapse to two rows.
    return TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS;
  }
};

watch(() => props.isExpanded, (newValue, _oldValue) => {
  if (isAnimating.value && newValue === props.isExpanded) {
    return;
  }
  animateHeightChange(newValue);
});

const animateHeightChange = (targetExpandedState: boolean) => {
  if (!cate2ContentRef.value) return;
  isAnimating.value = true;
  const content = cate2ContentRef.value;
  const targetHeightValue = getCurrentTargetHeight(targetExpandedState);

  // Handle collapsing from 'auto' height (if it was set)
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
    if (targetExpandedState && !props.hasMoreRows && props.cate2List.length > 0) {
        const originalTransition = content.style.transition;
        content.style.transition = 'none';
        content.style.height = 'auto';
        requestAnimationFrame(() => {
            content.style.transition = originalTransition;
        });
    } else if (!targetExpandedState && props.cate2List.length === 0) {
        // Use the same minimal height as in Douyin for consistency when empty and collapsed
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

const handleToggleExpand = () => {
  if (isAnimating.value) return;
  // The actual props.isExpanded will be toggled by the parent via useExpand, which then triggers the watch
  // This component just signals the intent to toggle
  emit('toggle-expand');
  // The animation will be triggered by the watch on props.isExpanded when the parent updates it.
};

// formatCategoryName function remains the same
const formatCategoryName = (name: string) => {
  if (!name) return '';
  
  // 统计字符串的实际字符数（考虑中文）
  const getStringLength = (str: string) => {
    let len = 0;
    for (let i = 0; i < str.length; i++) {
      if (str.charCodeAt(i) > 127 || str.charCodeAt(i) === 94) {
        len += 1;
      } else {
        len += 0.5; // 英文字符算半个长度
      }
    }
    return Math.ceil(len);
  };

  const nameLength = getStringLength(name);
  if (nameLength <= 4) {
    return name;
  }
  
  // 截取合适的长度加省略号
  let result = '';
  let currentLength = 0;
  for (let i = 0; i < name.length; i++) {
    const charLength = name.charCodeAt(i) > 127 || name.charCodeAt(i) === 94 ? 1 : 0.5;
    if (currentLength + charLength <= 3.5) {
      result += name[i];
      currentLength += charLength;
    } else {
      break;
    }
  }
  
  return result + '...';
}
</script>

<style scoped>
.cate2-container {
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  flex: 1;
  position: relative;
  overflow: visible;
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

.cate2-content.animating {
  overflow: hidden !important;
}

.cate2-content.scrollable:not(.animating) {
  overflow: hidden !important; /* 改为hidden以确保不出现滚动条 */
}

.cate2-scroll-wrapper {
  max-height: 100%;
  height: 100%;
  overflow: hidden;
}

.cate2-scroll-wrapper.allow-scroll:not(.animating) {
  overflow: hidden !important; /* 修改为hidden以彻底禁用滚动条 */
}

/* Expanded state: allow scrolling but hide scrollbar */
.cate2-content.is-expanded {
  overflow: hidden !important;
}

.cate2-content.is-expanded .cate2-scroll-wrapper {
  overflow-y: auto !important;
  -ms-overflow-style: none !important;  /* IE and Edge */
  scrollbar-width: none !important;  /* Firefox */
}

.cate2-content.is-expanded .cate2-scroll-wrapper::-webkit-scrollbar {
  display: none !important;  /* WebKit browsers */
  width: 0 !important;
  height: 0 !important;
}

/* Collapsed state: ensure no scrolling */
.cate2-content:not(.is-expanded) {
  overflow: hidden !important;
}

.cate2-content:not(.is-expanded) .cate2-scroll-wrapper {
  overflow: hidden !important;
}

.cate2-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 120px)); /* Further reduce card width to 120px */
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
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
  box-sizing: border-box;
  overflow: hidden;
  text-align: center; 
  display: flex; 
  align-items: center; 
  gap: 8px; 

  /* Night mode default */
  background: var(--cate2-card-bg-dark, #252731);
  border: 1px solid var(--cate2-card-border-dark, transparent); 
  color: var(--cate2-card-text-dark, #ccc); 
}

/* Day Mode: Ensure background is white and has appropriate border/shadow */
:root[data-theme="light"] .cate2-card {
  background-color: var(--content-card-bg-light, #FFFFFF); /* This should resolve to white */
  border: 1px solid var(--content-card-border-light, #e0e0e0); /* Adjusted for white bg - slightly darker border */
  box-shadow: var(--content-card-shadow-light, 0 1px 3px rgba(0,0,0,0.08)); /* Adjusted for white bg - slightly more visible shadow */
  color: var(--main-text-primary-light, #212529);
}

/* Night mode default hover for .cate2-card - Added Glow */
.cate2-card:hover {
  background-color: var(--cate2-card-hover-bg-dark, rgba(45, 48, 56, 1)); 
  border: 1px solid var(--cate2-card-hover-border-dark-glow, rgba(79, 209, 197, 0.5)); /* Cyan glow border, more transparent */
  color: var(--cate2-card-text-dark-hover, #ddd); 
  box-shadow: var(--cate2-card-hover-shadow-dark-glow, 0 0 5px rgba(79, 209, 197, 0.2), 0 0 0 1px rgba(79, 209, 197, 0.15)); /* Softer cyan glow */
}

/* Day Mode: Hover style for ANY card - Glow Effect (like Header Search active) */
:root[data-theme="light"] .cate2-card:hover {
  background-color: var(--content-card-hover-bg-light, #f8f9fa); /* Standard light hover bg */
  border: 1px solid transparent; /* --- MODIFIED --- */
  box-shadow: var(--douyu-cate2-hover-shadow-light-glow, 0 0 0 2px rgba(50, 150, 255, 0.35), 0 0 9px rgba(50, 150, 255, 0.2)); /* Blue glow */
  /* Text color inherited from day mode .cate2-card */
}

/* Night Mode Active - Revived Glow Style */
.cate2-card.active { 
  background-color: var(--cate2-card-bg-dark, #252731); 
  border: 1px solid transparent; /* Removed border for active state */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3)); /* Restored cyan glow */
}

:root[data-theme="dark"] .cate2-card.active {
  background-color: var(--cate2-card-bg-dark, #252731); 
  border-color: transparent; /* Removed border for active state */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived, 0 0 0 2px rgba(79, 209, 197, 0.45), 0 0 10px rgba(79, 209, 197, 0.3)); /* Restored cyan glow */
}

/* Text color for name inside active card in dark mode - Revived Glow Style */
:root[data-theme="dark"] .cate2-card.active .cate2-name { 
  color: var(--douyu-cate2-active-text-dark-glow-revived, rgb(79, 209, 197)); /* Restored cyan glow text */
}

/* Icon color for ACTIVE DARK mode - ensure original icon visible */
:root[data-theme="dark"] .cate2-card.active .cate2-icon img {
  filter: none; 
}

/* Night Mode: Hover on ACTIVE card - Intensified Glow */
:root[data-theme="dark"] .cate2-card.active:hover {
  background-color: var(--cate2-card-bg-dark, #252731); /* MODIFIED - Keep unselected bg */
  border-color: var(--douyu-cate2-active-border-dark-glow-revived, rgb(79, 209, 197)); /* Keep cyan border */
  box-shadow: var(--douyu-cate2-active-shadow-dark-glow-revived-hover, 0 0 0 2.5px rgba(79, 209, 197, 0.55), 0 0 12px rgba(79, 209, 197, 0.4)); /* Intensified cyan glow */
  /* Text and icon color inherited from .active night mode style */
}

/* Light Mode Active - Scheme A: Deep Emphasis (Modified) */
:root[data-theme="light"] .cate2-card.active {
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); /* Blue background */
  border-color: transparent; /* No border when active */
  box-shadow: var(--douyu-cate2-active-shadow-light-scheme-a, 0 2px 5px rgba(0,0,0,0.12)); /* Normal active shadow */
}

/* Text color for name inside active card in light mode - Scheme A */
:root[data-theme="light"] .cate2-card.active .cate2-name {
  color: var(--douyu-cate2-active-text-light-scheme-a, #FFFFFF);
}

/* Icon color for ACTIVE LIGHT mode - Scheme A (original icon color) */
:root[data-theme="light"] .cate2-card.active .cate2-icon img {
  filter: none; /* Original icon color */
}

/* Hover on Active card in Light Mode - Scheme A */
:root[data-theme="light"] .cate2-card.active:hover {
  background-color: var(--douyu-cate2-active-bg-light-scheme-a, #429cdd); /* Keep blue background */
  border: 1px solid transparent; /* --- ENSURED --- */
  box-shadow: var(--content-card-hover-shadow-light-larger, 0 5px 12px rgba(0,0,0,0.15)); /* Larger shadow like other hovers */
  /* Text and icon color are inherited from .active styles */
}

/* Icon Styling */
.cate2-icon {
  width: 24px; 
  height: 24px; 
  flex-shrink: 0; 
  display: flex; 
  align-items: center;
  justify-content: center;
}

.cate2-icon img {
  width: 100%; 
  height: 100%; 
  object-fit: cover; 
  border-radius: 4px; 
  transition: filter 0.2s ease;
}

.cate2-info {
  flex: 1;
  overflow: hidden;
}

.cate2-name {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

.expand-button {
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

  /* Default (Dark Mode) Styles */
  background: var(--douyu-cate2-expand-btn-bg-dark, #18181b);
  color: var(--douyu-cate2-expand-btn-text-dark, rgba(255, 255, 255, 0.6));
  border-top: 1px solid var(--douyu-cate2-expand-btn-border-dark, rgba(255, 255, 255, 0.1));
  z-index: 10;
}

.expand-button:hover {
  /* Default (Dark Mode) Hover Styles */
  color: var(--douyu-cate2-expand-btn-hover-text-dark, #ffffff);
}

/* Light Mode Styles */
:root[data-theme="light"] .expand-button {
  background: var(--douyu-cate2-expand-btn-bg-light, var(--primary-bg, #ffffff));
  color: var(--douyu-cate2-expand-btn-text-light, var(--secondary-text, #4b5563));
  border-top: 1px solid var(--douyu-cate2-expand-btn-border-light, var(--border-color-light, #e5e7eb));
}

:root[data-theme="light"] .expand-button:hover {
  color: var(--douyu-cate2-expand-btn-hover-text-light, var(--primary-text, #1f2937));
}

.expand-icon {
  margin-left: 4px;
  transition: transform 0.4s cubic-bezier(0.33, 0.66, 0.66, 1);
  width: 12px;
  height: 12px;
}

:root[data-theme="dark"] .expand-button .expand-icon {
  stroke: var(--douyu-cate2-expand-btn-text-dark, rgba(255, 255, 255, 0.6));
}
:root[data-theme="dark"] .expand-button:hover .expand-icon {
  stroke: var(--douyu-cate2-expand-btn-hover-text-dark, #ffffff);
}

:root[data-theme="light"] .expand-button .expand-icon {
  stroke: var(--douyu-cate2-expand-btn-text-light, var(--secondary-text, #4b5563));
}
:root[data-theme="light"] .expand-button:hover .expand-icon {
  stroke: var(--douyu-cate2-expand-btn-hover-text-light, var(--primary-text, #1f2937));
}

.expand-icon.is-expanded {
  transform: rotate(180deg);
}
</style>