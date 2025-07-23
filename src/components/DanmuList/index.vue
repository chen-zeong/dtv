<template>
    <div class="danmu-list-wrapper">
      <div class="danmu-header">
        <h4>弹幕列表</h4>
        <div class="danmu-controls">
          <!-- 保留功能但隐藏控件 -->
          <input type="checkbox" v-model="autoScroll" id="auto-scroll-toggle" class="hidden-toggle">
        </div>
      </div>
      <div class="danmu-messages-area" ref="danmakuListEl" @scroll="handleScroll">
        <!-- Empty/Loading Placeholder -->
        <div v-if="(!messages || messages.length === 0)" class="empty-danmu-placeholder">
          <p v-if="!props.roomId">请先选择一个直播间</p>
          <p v-else>暂无弹幕或连接中...</p> <!-- Simplified placeholder -->
        </div>

        <div v-for="(danmaku) in messages" :key="danmaku.id || danmaku.content + danmaku.nickname" 
             :class="['danmu-item', { 'system-message': danmaku.isSystem, 'success': danmaku.isSystem && danmaku.type === 'success' }]"
        >
          <div class="danmu-meta-line" v-if="!danmaku.isSystem">
            <span v-if="danmaku.badgeName" class="danmu-badge">
              <span class="badge-name">{{ danmaku.badgeName }}</span>
              <span v-if="danmaku.badgeLevel" class="badge-level">{{ danmaku.badgeLevel }}</span>
            </span>
            <span class="danmu-user" :style="{ color: danmaku.color || userColor(danmaku.nickname) }">
              <span v-if="danmaku.level" class="user-level">[Lv.{{ danmaku.level }}]</span>
              {{ danmaku.nickname }}:
            </span>
          </div>
          <div class="danmu-content-line">
            <span class="danmu-content">
              <svg v-if="danmaku.isSystem && danmaku.type === 'success'" class="inline-icon success-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/></svg>
              {{ danmaku.isSystem ? danmaku.nickname + ': ' : '' }}{{ danmaku.content }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, watch, nextTick } from 'vue';

  interface DanmakuUIMessage {
    id?: string;
    nickname: string;
    content: string;
    level?: string;
    badgeName?: string;
    badgeLevel?: string;
    color?: string;
    isSystem?: boolean; // 系统消息
    type?: string;
  }
  
  const props = defineProps<{
    roomId: string | null;
    messages: DanmakuUIMessage[];
  }>();
  
  const danmakuListEl = ref<HTMLElement | null>(null);
  const autoScroll = ref(true); 
  const userScrolled = ref(false);
  
  const userColor = (nickname: string | undefined) => {
    if (!nickname || nickname.length === 0) {
      const defaultHue = 0;
      const defaultSaturation = 0;
      const defaultLightness = 75;
      return `hsl(${defaultHue}, ${defaultSaturation}%, ${defaultLightness}%)`;
    }
    let hash = 0;
    for (let i = 0; i < nickname.length; i++) {
      hash = nickname.charCodeAt(i) + ((hash << 5) - hash);
      hash = hash & hash; 
    }
    const hue = hash % 360;
    return `hsl(${hue}, 70%, 75%)`;
  };
  
  const handleScroll = () => {
    if (!danmakuListEl.value) return;
    const el = danmakuListEl.value;

    const isScrolledUp = el.scrollHeight - el.scrollTop - el.clientHeight > 20; 

    if (isScrolledUp) {
      if (!userScrolled.value) {
        userScrolled.value = true;
      }
    } else {
      if (userScrolled.value) {
        userScrolled.value = false;
      }
    }
  };
  
  const scrollToBottom = () => {
    nextTick(() => {
      if (danmakuListEl.value && autoScroll.value && !userScrolled.value) {
        const el = danmakuListEl.value;
        el.scrollTop = el.scrollHeight;
      } else {
      }
    });
  };

  watch(autoScroll, (newValue) => {
    if (newValue) {
      userScrolled.value = false; 
      scrollToBottom();
    }
  });
  
  watch(() => props.messages, (newMessages, _oldMessages) => {
    if (newMessages && autoScroll.value && !userScrolled.value) {
      scrollToBottom();
    }
  }, { deep: true });

  watch(() => props.roomId, (_newRoomId, _oldRoomId) => {
    userScrolled.value = false;
    autoScroll.value = true;
    // scrollToBottom(); // Optionally scroll to bottom if there are initial messages for the new room
  });
  
  </script>
  
  <style scoped>
  .danmu-list-wrapper {
    display: flex;
    flex-direction: column;
    position: relative;
    height: 100%;
    width: 220px;
    background-color: var(--secondary-bg, #2c2c2e);
    color: var(--primary-text, #e0e0e0);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .danmu-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--border-color-dark, #1e1e1e);
    flex-shrink: 0;
    background-color: var(--tertiary-bg, #3a3a3c);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .danmu-header h4 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--primary-text-light, #f5f5f5);
  }
  
  .danmu-controls {
    display: flex;
    align-items: center;
  }
  
  .hidden-toggle {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
    pointer-events: none;
  }
  
  .danmu-messages-area {
    position: absolute;
    top: 40px;
    bottom: 0;
    left: 0;
    right: 0;
    overflow-y: auto; 
    padding: 8px 12px;
    scroll-behavior: smooth;
  }
  
  .empty-danmu-placeholder {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    width: 100%;
  }
  .empty-danmu-placeholder p {
    margin: 4px 0;
  }
  
  .danmu-item {
    text-align: left;
    padding: 4px 6px;
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.1);
    word-wrap: break-word;
    overflow-wrap: break-word;
    margin-bottom: 8px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
    transition: transform 0.15s ease-out, background-color 0.15s ease;
    display: flex;
    flex-direction: column;
    max-width: 100%; 
  }
  
  .danmu-item:hover {
    background: linear-gradient(to right, rgba(0,0,0,0.3), rgba(0,0,0,0.15));
    transform: translateY(-1px);
  }
  
  .danmu-meta-line {
    font-size: 0.8rem;
    color: var(--secondary-text, #aaa);
    margin-bottom: 2px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
  }
  
  .danmu-badge {
    background-color: var(--tag-bg, #FB7299); 
    color: #ffffff; 
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.7rem; 
    margin-right: 8px;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    height: auto;
    line-height: normal;
    flex-shrink: 0;
  }
  
  .badge-level {
    margin-left: 4px;
    font-weight: bold;
    font-size: 0.65rem; 
  }
  
  .danmu-user {
    font-weight: 500;
    margin-right: 5px;
  }
  
  .user-level {
    font-size: 0.7rem;
    color: var(--meta-text, #ababab); 
    margin-right: 5px;
  }
  
  .danmu-content-line {
    font-size: 0.85rem;
    line-height: 1.4;
  }
  
  .danmu-content {
    color: var(--primary-text-light, #f0f0f0); 
    white-space: pre-wrap; 
    word-wrap: break-word;
    overflow-wrap: break-word;
    font-size: 0.875rem; 
    line-height: 1.4;
  }
  
  .danmu-messages-area::-webkit-scrollbar {
    width: 6px;
  }
  
  .danmu-messages-area::-webkit-scrollbar-track {
    background: var(--tertiary-bg, #3a3a3c);
    border-radius: 3px;
  }
  
  .danmu-messages-area::-webkit-scrollbar-thumb {
    background-color: var(--border-color-light, #5a5a5e);
    border-radius: 3px;
  }
  
  .danmu-messages-area::-webkit-scrollbar-thumb:hover {
    background-color: var(--primary-accent, #007aff);
  }
  
  .danmu-messages-area {
    scrollbar-width: thin;
    scrollbar-color: var(--border-color-light, #5a5a5e) var(--tertiary-bg, #3a3a3c);
  }
  
  .connection-status-placeholder {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 10px;
  }
  
  .connection-status-placeholder.success {
    color: #28a745;
  }
  
  .connection-status-placeholder .status-icon {
    width: 32px;
    height: 32px;
    margin-bottom: 8px;
  }
  
  .connection-status-placeholder p {
    margin: 4px 0;
    font-size: 0.9rem; 
    font-weight: 500;
  }
  
  .danmu-item.system-message {
    background-color: rgba(40, 167, 69, 0.1);
    border-left: 3px solid #28a745;
    margin-top: 4px;
    margin-bottom: 8px;
  }

  .danmu-item.system-message .danmu-content {
    font-weight: normal;
  }

  .danmu-item.system-message.success .danmu-content {
    color: #218838;
    font-weight: 500;
  }

  .inline-icon {
    width: 1.1em;
    height: 1.1em;
    margin-right: 8px;
    vertical-align: -0.15em;
  }
  
  .success-icon {
    fill: #28a745;
  }
  

:root[data-theme="light"] .danmu-list-wrapper {
  background-color: var(--primary-bg-light, #ffffff);
  color: var(--primary-text-light, #333333);
  border: 1px solid var(--border-color-light, #e0e0e0);
}

:root[data-theme="light"] .danmu-header {
  background-color: var(--header-bg-light, #e9ecef);
  border-bottom: 1px solid var(--border-color-light, #e0e0e0);
}

:root[data-theme="light"] .danmu-header h4 {
  color: var(--primary-text-light, #333333);
}

:root[data-theme="light"] .danmu-messages-area {
  /* Scrollbar for light theme */
  scrollbar-color: var(--scrollbar-thumb-light, #cccccc) var(--scrollbar-track-light, #f0f0f0);
}

:root[data-theme="light"] .danmu-messages-area::-webkit-scrollbar-track {
  background: var(--scrollbar-track-light, #f0f0f0);
}

:root[data-theme="light"] .danmu-messages-area::-webkit-scrollbar-thumb {
  background-color: var(--scrollbar-thumb-light, #cccccc);
}

:root[data-theme="light"] .danmu-messages-area::-webkit-scrollbar-thumb:hover {
  background-color: var(--scrollbar-thumb-hover-light, #aaaaaa);
}

:root[data-theme="light"] .empty-danmu-placeholder p {
  color: var(--secondary-text-light, #777777);
}

:root[data-theme="light"] .danmu-item {
  background-color: var(--danmu-item-bg-light, #f9f9f9);
  box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  border: 1px solid var(--danmu-item-border-light, #eeeeee);
}

:root[data-theme="light"] .danmu-item:hover {
  background-color: var(--danmu-item-hover-bg-light, #f0f0f0);
  transform: none; /* Optional: remove transform for light mode if it feels too busy */
}

:root[data-theme="light"] .danmu-meta-line {
  color: var(--meta-text-light, #666666);
}

:root[data-theme="light"] .danmu-badge {
   color: #ffffff; 
}

:root[data-theme="light"] .user-level {
  color: var(--meta-text-light, #888888);
}

:root[data-theme="light"] .danmu-content {
  color: var(--primary-text-light, #333333);
}

:root[data-theme="light"] .danmu-item.system-message {
  background-color: var(--system-message-bg-light, #e6f7ff);
  border-left-color: var(--system-message-border-light, #91d5ff);
}

:root[data-theme="light"] .danmu-item.system-message .danmu-content {
  color: var(--system-message-text-light, #0050b3);
}

:root[data-theme="light"] .danmu-item.system-message.success {
  background-color: var(--system-success-bg-light, #f6ffed); /* Light green for success */
  border-left-color: var(--system-success-border-light, #b7eb8f);
}

:root[data-theme="light"] .danmu-item.system-message.success .danmu-content {
  color: var(--system-success-text-light, #389e0d);
}

:root[data-theme="light"] .success-icon {
  fill: var(--system-success-text-light, #389e0d); /* Match success text color */
}

:root[data-theme="light"] .connection-status-placeholder.success {
    color: var(--system-success-text-light, #28a745); /* Consistent green for success */
}
  
  </style>