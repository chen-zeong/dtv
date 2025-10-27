<template>
    <div class="danmu-list-wrapper">
      <div class="danmu-messages-area" ref="danmakuListEl" @scroll="handleScroll" @pointerdown="onPointerDown">
        <!-- Empty/Loading Placeholder -->
        <div v-if="(!renderMessages || renderMessages.length === 0)" class="empty-danmu-placeholder">
          <p v-if="!props.roomId">请先选择一个直播间</p>
          <p v-else>暂无弹幕或连接中...</p> <!-- Simplified placeholder -->
        </div>

        <div v-for="(danmaku, idx) in renderMessages" :key="danmaku.id || `${danmaku.room_id || ''}-${danmaku.nickname}-${danmaku.content}-${idx}`" 
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
  import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue';

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
    room_id?: string; // 补充房间ID以便 key 生成更稳定
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
      scrollToBottomForce();
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
  
  const renderMessages = ref<DanmakuUIMessage[]>([]);
  const MAX_MSG = 200;
  const PRUNE_BATCH = 100;
  const pointerActive = ref(false);
  
  const onPointerDown = () => {
    pointerActive.value = true;
    autoScroll.value = false; // 用户主动拖动时暂停自动滚动
  };
  
  const onGlobalPointerUp = () => {
    if (pointerActive.value) {
      pointerActive.value = false;
      autoScroll.value = true; // 松开后恢复自动滚动
      userScrolled.value = false;
      scrollToBottomForce();
    }
  };
  
  const scrollToBottomForce = () => {
    nextTick(() => {
      const el = danmakuListEl.value;
      if (!el) return;
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight;
        requestAnimationFrame(() => {
          el.scrollTop = el.scrollHeight; // 双RAF确保强制到底部，减少偶发失效
        });
      });
    });
  };

  watch(() => props.messages, (newMessages, _oldMessages) => {
    const msgs = Array.isArray(newMessages) ? newMessages : [];
    if (msgs.length > MAX_MSG) {
      // 批量裁剪，避免频繁处理导致性能问题
      if (msgs.length % PRUNE_BATCH === 0 || msgs.length > MAX_MSG + PRUNE_BATCH) {
        renderMessages.value = msgs.slice(-MAX_MSG);
      } else {
        renderMessages.value = msgs.slice(-MAX_MSG);
      }
    } else {
      renderMessages.value = msgs;
    }
    if (autoScroll.value && !pointerActive.value) {
      scrollToBottomForce();
    }
  }, { deep: true });

  watch(() => props.roomId, (_newRoomId, _oldRoomId) => {
    userScrolled.value = false;
    autoScroll.value = true;
    scrollToBottomForce();
  });
  
  onMounted(() => {
    window.addEventListener('pointerup', onGlobalPointerUp);
  });
  
  onUnmounted(() => {
    window.removeEventListener('pointerup', onGlobalPointerUp);
  });
  
  </script>
  
  <style scoped>
  .danmu-list-wrapper {
    display: flex;
    flex-direction: column;
    position: relative;
    height: 100%;
    min-height: 0;
    max-height: 100%;
    width: 100%;
    background: linear-gradient(165deg, rgba(28, 30, 42, 0.96), rgba(15, 17, 26, 0.9));
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    color: var(--primary-text, #e5e9f5);
    font-family: "HarmonyOS Sans Bold", "HarmonyOS Sans", "PingFang SC", "Helvetica Neue", Arial, sans-serif;
    border-radius: 0 16px 16px 0;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-left: none;
    box-shadow: 0 16px 36px rgba(7, 10, 20, 0.48);
    overflow: hidden;
    isolation: isolate;
  }

  .danmu-list-wrapper::before {
    content: '';
    position: absolute;
    inset: -30% -15% 45% -15%;
    background: radial-gradient(110% 70% at 18% -4%, rgba(92, 140, 226, 0.22), transparent 72%);
    opacity: 0.45;
    pointer-events: none;
  }

  .danmu-list-wrapper::after {
    content: '';
    position: absolute;
    inset: 55% -25% -40% -25%;
    background: radial-gradient(120% 65% at 78% 118%, rgba(198, 118, 214, 0.18), transparent 75%);
    opacity: 0.4;
    pointer-events: none;
  }

  .danmu-list-wrapper > * {
    position: relative;
    z-index: 1;
  }
  
  .danmu-messages-area {
    position: relative;
    flex: 1;
    min-height: 0;
    max-height: 100%;
    overflow-y: auto; 
    padding: 10px 12px;
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
    padding: 6px 10px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.04);
    word-wrap: break-word;
    overflow-wrap: break-word;
    margin-bottom: 8px;
    box-shadow: 0 10px 22px rgba(8, 10, 22, 0.28);
    transition: transform 0.2s ease, background 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
    display: flex;
    flex-direction: column;
    max-width: 100%; 
  }
  
  .danmu-item:hover {
    background: rgba(255, 255, 255, 0.1);
    transform: translateY(-2px);
    border-color: rgba(255, 255, 255, 0.08);
    box-shadow: 0 16px 32px rgba(8, 10, 22, 0.42);
  }
  
  .danmu-meta-line {
    font-size: 0.72rem;
    color: rgba(204, 212, 236, 0.72);
    margin-bottom: 2px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    letter-spacing: 0.01em;
  }
  
  .danmu-badge {
    background: linear-gradient(135deg, rgba(92, 153, 255, 0.75), rgba(236, 112, 214, 0.68)); 
    color: #ffffff; 
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 0.64rem; 
    margin-right: 8px;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    height: auto;
    line-height: normal;
    flex-shrink: 0;
    box-shadow: 0 6px 14px rgba(100, 140, 255, 0.24);
  }
  
  .badge-level {
    margin-left: 4px;
    font-weight: 600;
    font-size: 0.62rem; 
  }
  
  .danmu-user {
    font-weight: 600;
    margin-right: 6px;
    color: inherit;
  }
  
  .user-level {
    font-size: 0.7rem;
    color: rgba(166, 183, 219, 0.85); 
    margin-right: 5px;
  }
  
  .danmu-content-line {
    font-size: 0.8rem;
    line-height: 1.4;
  }

  .danmu-content {
    color: rgba(244, 246, 255, 0.94); 
    white-space: pre-wrap; 
    word-wrap: break-word;
    overflow-wrap: break-word;
    font-size: 0.84rem; 
    line-height: 1.45;
    text-shadow: 0 1px 2px rgba(6, 9, 18, 0.6);
  }
  
  .danmu-messages-area::-webkit-scrollbar {
    width: 6px;
  }
  
  .danmu-messages-area::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.04);
    border-radius: 3px;
  }
  
  .danmu-messages-area::-webkit-scrollbar-thumb {
    background-color: rgba(102, 164, 255, 0.45);
    border-radius: 3px;
  }
  
  .danmu-messages-area::-webkit-scrollbar-thumb:hover {
    background-color: rgba(236, 112, 214, 0.65);
  }
  
  .danmu-messages-area {
    scrollbar-width: thin;
    scrollbar-color: rgba(102, 164, 255, 0.45) rgba(255, 255, 255, 0.04);
  }

  @media (max-width: 1024px) {
    .danmu-list-wrapper {
      width: 100%;
      border-radius: 12px;
      border-left: 1px solid rgba(255, 255, 255, 0.08);
    }
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
    color: #2f8f46;
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
    background: rgba(57, 185, 108, 0.16);
    border-left: 3px solid rgba(57, 185, 108, 0.75);
    margin-top: 4px;
    margin-bottom: 6px;
    box-shadow: 0 10px 20px rgba(26, 54, 39, 0.32);
  }

  .danmu-item.system-message .danmu-content {
    font-weight: 500;
    color: rgba(210, 240, 220, 0.95);
  }

  .danmu-item.system-message.success .danmu-content {
    color: #49df85;
    font-weight: 600;
  }

  .inline-icon {
    width: 1.1em;
    height: 1.1em;
    margin-right: 8px;
    vertical-align: -0.15em;
  }
  
.success-icon {
  fill: #49df85;
}
  

:root[data-theme="light"] .danmu-list-wrapper {
  background: linear-gradient(150deg, rgba(249, 251, 255, 0.97), rgba(233, 238, 252, 0.94));
  color: var(--primary-text-light, #1f2937);
  border: 1px solid rgba(189, 200, 224, 0.45);
  border-left: none;
  box-shadow: 0 16px 28px rgba(15, 23, 42, 0.1);
}

:root[data-theme="light"] .danmu-list-wrapper::before {
  content: '';
  position: absolute;
  inset: -25% -20% 55% -20%;
  background: radial-gradient(115% 70% at 12% -6%, rgba(164, 186, 255, 0.22), transparent 76%);
  opacity: 0.35;
}

:root[data-theme="light"] .danmu-list-wrapper::after {
  content: '';
  position: absolute;
  inset: 60% -20% -35% -20%;
  background: radial-gradient(120% 65% at 82% 110%, rgba(255, 186, 210, 0.18), transparent 78%);
  opacity: 0.3;
}

:root[data-theme="light"] .danmu-messages-area {
  scrollbar-color: rgba(125, 155, 238, 0.55) rgba(226, 232, 250, 0.7);
}

:root[data-theme="light"] .danmu-messages-area::-webkit-scrollbar-track {
  background: rgba(226, 232, 250, 0.75);
}

:root[data-theme="light"] .danmu-messages-area::-webkit-scrollbar-thumb {
  background-color: rgba(125, 155, 238, 0.6);
}

:root[data-theme="light"] .danmu-messages-area::-webkit-scrollbar-thumb:hover {
  background-color: rgba(190, 142, 255, 0.7);
}

:root[data-theme="light"] .empty-danmu-placeholder p {
  color: rgba(100, 116, 139, 0.85);
}

:root[data-theme="light"] .danmu-item {
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 10px 20px rgba(15, 23, 42, 0.08);
  border: 1px solid rgba(189, 200, 224, 0.5);
}

:root[data-theme="light"] .danmu-item:hover {
  background: rgba(235, 240, 255, 0.95);
  transform: translateY(-2px);
  border-color: rgba(125, 155, 238, 0.45);
}

:root[data-theme="light"] .danmu-meta-line {
  color: rgba(71, 85, 105, 0.85);
}

:root[data-theme="light"] .danmu-badge {
  color: #ffffff; 
  box-shadow: 0 6px 14px rgba(100, 140, 255, 0.28);
}

:root[data-theme="light"] .user-level {
  color: rgba(100, 116, 139, 0.78);
}

:root[data-theme="light"] .danmu-content {
  color: var(--primary-text-light, #1f2937);
  text-shadow: none;
}

:root[data-theme="light"] .danmu-item.system-message {
  background: rgba(226, 246, 233, 0.95);
  border-left-color: rgba(78, 196, 120, 0.75);
}

:root[data-theme="light"] .danmu-item.system-message .danmu-content {
  color: rgba(31, 106, 58, 0.9);
}

:root[data-theme="light"] .danmu-item.system-message.success {
  background: rgba(238, 252, 238, 0.96);
  border-left-color: rgba(126, 217, 137, 0.85);
}

:root[data-theme="light"] .danmu-item.system-message.success .danmu-content {
  color: rgba(46, 114, 66, 0.95);
}

:root[data-theme="light"] .success-icon {
  fill: rgba(46, 114, 66, 0.95);
}

:root[data-theme="light"] .connection-status-placeholder.success {
  color: rgba(46, 114, 66, 0.95);
}

@media (max-width: 1024px) {
  :root[data-theme="light"] .danmu-list-wrapper {
    border-left: 1px solid rgba(189, 200, 224, 0.55);
  }
}
  
  </style>
