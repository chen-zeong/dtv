<template>
  <div class="win-controls" data-tauri-drag-region="none">
    <button
      type="button"
      class="win-control win-control--minimize"
      @click="handleMinimize"
      aria-label="最小化窗口"
      data-tauri-drag-region="none"
    >
      <svg class="win-icon" viewBox="0 0 10 10" aria-hidden="true">
        <path d="M1 5h8" />
      </svg>
    </button>
    <button
      type="button"
      class="win-control win-control--maximize"
      @click="handleMaximize"
      :aria-label="isMaximized ? '还原窗口' : '最大化窗口'"
      data-tauri-drag-region="none"
    >
      <svg v-if="!isMaximized" class="win-icon" viewBox="0 0 10 10" aria-hidden="true">
        <rect x="1.5" y="1.5" width="7" height="7" />
      </svg>
      <svg v-else class="win-icon is-restore" viewBox="0 0 10 10" aria-hidden="true">
        <path d="M3 2h5v5h-1.5" />
        <rect x="2" y="3" width="5" height="5" />
      </svg>
    </button>
    <button
      type="button"
      class="win-control win-control--close"
      @click="handleClose"
      aria-label="关闭窗口"
      data-tauri-drag-region="none"
    >
      <svg class="win-icon" viewBox="0 0 10 10" aria-hidden="true">
        <path d="M2 2l6 6M8 2L2 8" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { UnlistenFn } from '@tauri-apps/api/event';

const currentWindow = getCurrentWindow();
const isMaximized = ref(false);
let unlistenResize: UnlistenFn | null = null;

const syncMaximizedState = async () => {
  try {
    isMaximized.value = await currentWindow.isMaximized();
  } catch (error) {
    console.error('[WindowsWindowControls] Failed to query maximized state', error);
  }
};

const handleMinimize = async () => {
  try {
    await currentWindow.minimize();
  } catch (error) {
    console.error('[WindowsWindowControls] Failed to minimize window', error);
  }
};

const handleMaximize = async () => {
  try {
    if (isMaximized.value) {
      await currentWindow.unmaximize();
    } else {
      await currentWindow.maximize();
    }
    await syncMaximizedState();
  } catch (error) {
    console.error('[WindowsWindowControls] Failed to toggle maximize', error);
  }
};

const handleClose = async () => {
  try {
    await currentWindow.close();
  } catch (error) {
    console.error('[WindowsWindowControls] Failed to close window', error);
  }
};

onMounted(async () => {
  await syncMaximizedState();
  try {
    unlistenResize = await currentWindow.onResized(() => {
      syncMaximizedState();
    });
  } catch (error) {
    console.error('[WindowsWindowControls] Failed to listen for resize events', error);
  }
});

onBeforeUnmount(async () => {
  if (unlistenResize) {
    await unlistenResize();
    unlistenResize = null;
  }
});
</script>

<style scoped>
.win-controls {
  display: flex;
  align-items: stretch;
  background-color: transparent;
  border-radius: 0;
  overflow: hidden;
  box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.18);
  -webkit-app-region: no-drag;
}

.win-control {
  width: 46px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.82);
  cursor: pointer;
  transition: background-color 0.16s ease, color 0.16s ease;
  -webkit-app-region: no-drag;
}

.win-control:focus-visible {
  outline: 2px solid rgba(88, 142, 255, 0.8);
  outline-offset: -2px;
}

.win-control:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.win-control:active {
  background-color: rgba(255, 255, 255, 0.16);
}

.win-control--close {
  color: rgba(255, 255, 255, 0.92);
}

.win-control--close:hover {
  background-color: #e81123;
  color: #ffffff;
}

.win-control--close:active {
  background-color: #c50f1f;
  color: #ffffff;
}

.win-icon {
  width: 10px;
  height: 10px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.2;
  stroke-linecap: square;
  stroke-linejoin: miter;
}

.win-icon.is-restore path:first-of-type {
  fill: none;
}

:global(:root[data-theme="light"] .win-controls) {
  box-shadow: none;
}

:global(:root[data-theme="light"] .win-controls .win-control) {
  color: #111318;
}

:global(:root[data-theme="light"] .win-controls .win-control:hover) {
  background-color: rgba(17, 19, 24, 0.08);
}

:global(:root[data-theme="light"] .win-controls .win-control:active) {
  background-color: rgba(17, 19, 24, 0.16);
}

:global(:root[data-theme="light"] .win-controls .win-control--close) {
  color: #111318;
}

:global(:root[data-theme="light"] .win-controls .win-control--close:hover) {
  background-color: #e81123;
  color: #ffffff;
}

:global(:root[data-theme="light"] .win-controls .win-control--close:active) {
  background-color: #c50f1f;
  color: #ffffff;
}
</style>
