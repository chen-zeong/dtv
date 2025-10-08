<template>
  <div class="streamer-item-content" :class="{ big: big }">
    <div class="item-content" @click="onClick">
      <div class="avatar-container" :class="{ big: big }">
        <img 
          v-if="streamer.avatarUrl && (streamer.platform !== Platform.BILIBILI || !!proxyBase)"
          :src="getAvatarSrc(streamer)"
          :alt="streamer.nickname"
          @error="handleImgError($event, streamer)"
          class="avatar-image"
        >
        <div v-else class="avatar-fallback">{{ streamer.nickname[0] }}</div>
      </div>
      
      <div class="streamer-details">
        <div class="primary-row">
          <span class="nickname" :title="streamer.nickname">{{ streamer.nickname }}</span>
          <!-- 移除左侧平台名，改为右侧胶囊与状态点集成 -->
        </div>
        <div class="secondary-row" :title="streamer.roomTitle">
          {{ streamer.roomTitle || '暂无直播标题' }}
        </div>
      </div>
    </div>

    <div class="status-container">
      <div v-if="showPlatform" class="platform-badge">
        <span class="live-indicator" :class="getLiveIndicatorClass(streamer)"></span>
        <span class="badge-text">{{ platformLabel(streamer.platform) }}</span>
      </div>
      <div v-else class="live-indicator" :class="getLiveIndicatorClass(streamer)"></div>
    </div>
  </div>
</template>

<style scoped>
.streamer-item-content {
  display: flex;
  align-items: center;
  justify-content: flex-start; /* 保持内容靠左，状态胶囊以 margin-left 自动右对齐 */
  width: 100%; /* 占满父级，使右侧胶囊推到最右侧 */
}

.item-content {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1; /* 占据剩余空间，便于右侧胶囊右对齐 */
  min-width: 0; /* 允许内部收缩并出现省略号 */
  overflow: hidden; /* 防止内部因长文本溢出影响右侧状态点 */
}

.avatar-container {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  overflow: hidden;
  background: var(--card-bg, rgba(255,255,255,0.04));
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto; /* 固定尺寸不参与收缩 */
}
.avatar-container.big {
  width: 48px;
  height: 48px;
}
.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.avatar-fallback {
  font-size: 14px;
  font-weight: 600;
  color: var(--primary-text, #e2e8f0);
}

.streamer-details {
  display: flex;
  flex-direction: column;
  flex: 1; /* 允许详情区域占满并在必要时收缩 */
  min-width: 0; /* 对 flex 子项生效的省略号关键设置 */
}
.primary-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}
.nickname {
  font-weight: 700; /* 保留粗体 */
  color: var(--primary-text, #e2e8f0);
  font-size: 14px; /* 缩小字体 */
  max-width: 140px;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
/* 大尺寸卡片（overlay 内使用）：使用百分比限制，保证右侧状态点空间 */
.streamer-item-content.big .nickname { max-width: 100%; font-size: 14px; }
.streamer-item-content.big .secondary-row { max-width: 100%; }

.status-container {
  display: flex;
  align-items: center;
  margin-left: auto; /* 将右侧胶囊/状态点推到最右侧 */
  flex: 0 0 auto; /* 固定宽度，不参与收缩，避免被长文本挤出 */
}
/* 胶囊样式：集成状态点 + 平台名 */
.platform-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  padding: 4px 10px;
  background: var(--card-bg, rgba(255,255,255,0.06));
  color: var(--secondary-text, #a0aec0);
  font-size: 12px;
}
.platform-badge .live-indicator {
  width: 6px;
  height: 6px;
}
.badge-text { line-height: 1; }

.live-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--border-color);
}
.live-indicator.is-live { background: #22c55e; }
.live-indicator.is-replay { background: #f59e0b; }
.live-indicator.is-offline { background: #6b7280; }
</style>

<style scoped>
/* 直播间标题：恢复更小字号与单行显示 */
.secondary-row {
  font-size: 12px;
  color: var(--secondary-text, #a0aec0);
  max-width: 160px;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap; /* 单行 */
}
.streamer-item-content.big .secondary-row { max-width: 100%; font-size: 12px; }
</style>

<script setup lang="ts">
import { Platform } from '../../platforms/common/types';
import type { FollowedStreamer } from '../../platforms/common/types';
import { computed } from 'vue';

const props = defineProps<{
  streamer: FollowedStreamer,
  getAvatarSrc: (s: FollowedStreamer) => string,
  handleImgError: (ev: Event, s: FollowedStreamer) => void,
  getLiveIndicatorClass: (s: FollowedStreamer) => string,
  proxyBase?: string,
  big?: boolean,
  showPlatform?: boolean
}>();

const emit = defineEmits<{ (e: 'clickItem', s: FollowedStreamer): void }>();

const onClick = () => emit('clickItem', props.streamer);

const platformLabel = (p: Platform): string => {
  switch (p) {
    case Platform.DOUYU: return '斗鱼';
    case Platform.DOUYIN: return '抖音';
    case Platform.HUYA: return '虎牙';
    case Platform.BILIBILI: return 'B站';
    default: return '未知';
  }
};

const proxyBase = computed(() => props.proxyBase || '');
const showPlatform = computed(() => !!props.showPlatform);
</script>