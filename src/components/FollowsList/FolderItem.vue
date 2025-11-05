<template>
  <div 
    class="folder-item"
    :class="{ 'is-dragging': isDragging, 'is-expanded': folder.expanded }"
    @mousedown="handleMouseDown"
    @contextmenu.prevent="handleContextMenu"
  >
    <div class="folder-header" @click="toggleExpand">
      <svg 
        class="folder-icon" 
        :class="{ 'is-expanded': folder.expanded }"
        xmlns="http://www.w3.org/2000/svg" 
        width="16" 
        height="16" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        stroke-width="2" 
        stroke-linecap="round" 
        stroke-linejoin="round"
      >
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>
      <span class="folder-name">{{ folder.name }}</span>
      <span class="folder-count">({{ folder.streamerIds.length }})</span>
      <svg 
        class="expand-icon" 
        :class="{ 'is-expanded': folder.expanded }"
        xmlns="http://www.w3.org/2000/svg" 
        width="12" 
        height="12" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        stroke-width="2.5" 
        stroke-linecap="round" 
        stroke-linejoin="round"
      >
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </div>
    
    <Transition name="folder-content">
      <div v-if="folder.expanded && folderItems.length > 0" class="folder-content">
        <ul class="folder-streamers-list">
          <li
            v-for="streamer in folderItems"
            :key="`${streamer.platform}:${streamer.id}`"
            class="folder-streamer-item"
            :class="getStreamerItemClass(streamer)"
            @click.stop="handleClick(streamer)"
          >
            <StreamerItem 
              :streamer="streamer"
              :getAvatarSrc="getAvatarSrc"
              :handleImgError="handleImgError"
              :getLiveIndicatorClass="getLiveIndicatorClass"
              :proxyBase="proxyBase"
              @clickItem="(s) => emit('selectAnchor', s)"
            />
          </li>
        </ul>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { FollowedStreamer } from '../../platforms/common/types';
import type { FollowFolder } from '../../store/followStore';
import StreamerItem from './StreamerItem.vue';

const props = defineProps<{
  folder: FollowFolder;
  allStreamers: FollowedStreamer[];
  getAvatarSrc: (s: FollowedStreamer) => string;
  handleImgError: (ev: Event, s: FollowedStreamer) => void;
  getLiveIndicatorClass: (s: FollowedStreamer) => string;
  proxyBase?: string;
  isDragging?: boolean;
}>();

const emit = defineEmits<{
  (e: 'selectAnchor', streamer: FollowedStreamer): void;
  (e: 'toggleExpand', folderId: string): void;
  (e: 'dragStart', folderId: string, event: MouseEvent): void;
  (e: 'contextMenu', folderId: string, event: MouseEvent): void;
}>();

const folderItems = computed(() => {
  return props.folder.streamerIds
    .map((key: string) => {
      const [platform, id] = key.split(':');
      return props.allStreamers.find((s: FollowedStreamer) => s.platform === platform && s.id === id);
    })
    .filter((s): s is FollowedStreamer => s !== undefined);
});

const toggleExpand = () => {
  emit('toggleExpand', props.folder.id);
};

const handleMouseDown = (e: MouseEvent) => {
  if (e.button === 0) {
    emit('dragStart', props.folder.id, e);
  }
};

const handleContextMenu = (e: MouseEvent) => {
  emit('contextMenu', props.folder.id, e);
};

const handleClick = (streamer: FollowedStreamer) => {
  emit('selectAnchor', streamer);
};

const getStreamerItemClass = (streamer: FollowedStreamer) => {
  return {
    'status-live': streamer.liveStatus === 'LIVE',
    'status-replay': streamer.liveStatus === 'REPLAY',
    'status-offline': streamer.liveStatus === 'OFFLINE' || !streamer.liveStatus || streamer.liveStatus === 'UNKNOWN',
  };
};
</script>

<style scoped>
.folder-item {
  position: relative;
  margin-bottom: 8px;
  border-radius: 14px;
  background: rgba(34, 34, 38, 0.96);
  border: 1px solid rgba(96, 98, 112, 0.32);
  overflow: hidden;
  transition: transform 0.25s ease, border-color 0.25s ease, background 0.25s ease;
}

.folder-item.is-dragging {
  opacity: 0.85;
  transform: scale(1.01);
}

.folder-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s ease;
}

.folder-header:hover {
  background: rgba(52, 53, 60, 0.96);
}

.folder-icon {
  width: 16px;
  height: 16px;
  color: rgba(148, 163, 184, 0.8);
  transition: transform 0.2s ease, color 0.2s ease;
  flex-shrink: 0;
}

.folder-icon.is-expanded {
  color: rgba(125, 211, 252, 0.9);
}

.folder-name {
  flex: 1;
  font-weight: 600;
  font-size: 13px;
  color: rgba(226, 232, 240, 0.94);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.folder-count {
  font-size: 12px;
  color: rgba(148, 163, 184, 0.7);
  margin-left: 4px;
}

.expand-icon {
  width: 12px;
  height: 12px;
  color: rgba(148, 163, 184, 0.7);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.expand-icon.is-expanded {
  transform: rotate(180deg);
}

.folder-content {
  padding: 4px 8px 8px;
  border-top: 1px solid rgba(96, 98, 112, 0.2);
}

.folder-streamers-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.folder-streamer-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 4px 12px;
  border-radius: 10px;
  background: rgba(40, 40, 46, 0.8);
  border: 1px solid rgba(96, 98, 112, 0.24);
  cursor: pointer;
  transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease;
}

.folder-streamer-item:hover {
  transform: translateY(-1px);
  border-color: rgba(168, 174, 189, 0.35);
  background: rgba(52, 53, 60, 0.9);
}

.folder-streamer-item.status-live {
  border-color: rgba(16, 185, 129, 0.4);
  background: rgba(16, 185, 129, 0.12);
}

.folder-content-enter-active,
.folder-content-leave-active {
  transition: opacity 0.2s ease, max-height 0.3s ease;
  overflow: hidden;
}

.folder-content-enter-from,
.folder-content-leave-to {
  opacity: 0;
  max-height: 0;
}

:root[data-theme="light"] .folder-item {
  background: #f4f7fd;
  border-color: rgba(209, 217, 234, 0.7);
}

:root[data-theme="light"] .folder-header:hover {
  background: rgba(114, 147, 255, 0.12);
}

:root[data-theme="light"] .folder-name {
  color: #334155;
}

:root[data-theme="light"] .folder-icon {
  color: rgba(71, 85, 105, 0.7);
}

:root[data-theme="light"] .folder-icon.is-expanded {
  color: rgba(114, 147, 255, 0.9);
}

:root[data-theme="light"] .folder-count {
  color: rgba(100, 116, 139, 0.7);
}

:root[data-theme="light"] .folder-streamer-item {
  background: rgba(255, 255, 255, 0.6);
  border-color: rgba(209, 217, 234, 0.6);
}

:root[data-theme="light"] .folder-streamer-item:hover {
  background: rgba(114, 147, 255, 0.15);
  border-color: rgba(114, 147, 255, 0.35);
}

:root[data-theme="light"] .folder-streamer-item.status-live {
  background: rgba(34, 197, 94, 0.18);
  border-color: rgba(34, 197, 94, 0.4);
}
</style>

