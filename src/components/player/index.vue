<template>
  <div class="player-page" :class="{ 'web-fs': isInWebFullscreen || isInNativePlayerFullscreen }">
    <button v-if="!isInWebFullscreen" @click="$emit('close-player')" class="player-close-btn" title="关闭播放器">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>

    <div class="player-layout">
      <div class="main-content">
        <div v-if="!roomId" class="empty-player">
          <div class="empty-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
               <circle cx="12" cy="12" r="10"></circle>
               <line x1="12" y1="16" x2="12" y2="12"></line>
               <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
          </div>
          <h3>未选择直播间</h3>
          <p>请从首页选择一个直播间开始观看。</p>
        </div>
        <div v-else-if="isLoadingStream" class="loading-player">
          <div class="spinner">
            <span class="spinner-track"></span>
            <span class="spinner-head"></span>
            <span class="spinner-dot"></span>
          </div>
        </div>
        <div v-else-if="isOfflineError" class="offline-player">
          <!-- Display StreamerInfo if room details are available -->
          <StreamerInfo 
            v-if="props.roomId && props.platform"
            :room-id="props.roomId"
            :platform="props.platform"
            :title="playerTitle"
            :anchor-name="playerAnchorName"
            :avatar="playerAvatar"
            :is-live="false"
            :is-followed="props.isFollowed"
            @follow="$emit('follow', $event)"
            @unfollow="$emit('unfollow', $event)"
            class="streamer-info-offline"
          />
          <div class="offline-message">
            <div class="offline-icon">
              <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M16 16.427A4.002 4.002 0 0 0 12.005 20a4 4 0 0 0-3.995-3.573M12 12V2M8.5 7L7 5.5M15.5 7l1.5-1.5M5.562 10.223l-1.842.511M18.438 10.223l1.842.511M12 2a3.5 3.5 0 0 1 3.5 3.5V12H8.5V5.5A3.5 3.5 0 0 1 12 2z"/>
                <line x1="1" y1="1" x2="23" y2="23" stroke-width="2"></line> 
              </svg>
            </div>
            <h3>😴 获取直播流失败了</h3>
            <p>主播当前未开播，请稍后再来。</p>
            <button @click="retryInitialization" class="retry-btn">再试一次</button>
          </div>
        </div>
        <div v-else-if="streamError && !isOfflineError" class="error-player">
          <div class="error-icon">
             <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
          <h3>加载失败</h3>
          <p>{{ streamError }}</p>
          <button @click="retryInitialization" class="retry-btn">再试一次</button>
        </div>
        <div v-else class="player-container">
          <StreamerInfo
            v-if="props.roomId"
            :room-id="props.roomId"
            :platform="props.platform"
            :title="playerTitle"
            :anchor-name="playerAnchorName"
            :avatar="playerAvatar"
            :is-followed="props.isFollowed"
            :is-live="playerIsLive"
            @follow="$emit('follow', $event)"
            @unfollow="$emit('unfollow', $event)"
            class="streamer-info"
            v-show="!isInWebFullscreen"
            :class="{'hidden-panel': isInWebFullscreen}"
          />
          <div class="video-container">
            <div ref="playerContainerRef" class="video-player"></div>
          </div>
        </div>
      </div>

      <DanmuList 
        v-if="roomId && !isLoadingStream && !streamError" 
        :room-id="props.roomId"
        :messages="danmakuMessages"
        v-show="!isFullScreen" 
        class="danmu-panel" 
        :class="{'hidden-panel': isFullScreen}"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch, onUnmounted, shallowRef, nextTick } from 'vue';
import Player from 'xgplayer';
import FlvPlugin from 'xgplayer-flv';
import DanmuJs from 'danmu.js';
import 'xgplayer/dist/index.min.css';
import Plugin, { POSITIONS } from 'xgplayer/es/plugin/plugin.js';

import './player.css';

import { Platform as StreamingPlatform } from '../../platforms/common/types';
import type { DanmakuMessage, DanmuOverlayInstance } from './types';

// Platform-specific player helpers
import { getDouyuStreamConfig, startDouyuDanmakuListener, stopDouyuDanmaku, stopDouyuProxy } from '../../platforms/douyu/playerHelper';
import { fetchAndPrepareDouyinStreamConfig, startDouyinDanmakuListener, stopDouyinDanmaku } from '../../platforms/douyin/playerHelper';
import { getHuyaStreamConfig, startHuyaDanmakuListener, stopHuyaDanmaku, stopHuyaProxy } from '../../platforms/huya/playerHelper';
import { getBilibiliStreamConfig, startBilibiliDanmakuListener, stopBilibiliDanmaku } from '../../platforms/bilibili/playerHelper';

import StreamerInfo from '../StreamerInfo/index.vue';
import DanmuList from '../DanmuList/index.vue';
import { platform } from '@tauri-apps/plugin-os';

import { invoke } from '@tauri-apps/api/core';
import { useImageProxy } from '../FollowsList/useProxy';

// Ensure image proxy helpers are available in this component
const { ensureProxyStarted, proxify } = useImageProxy();

type DanmuUserSettings = {
  color: string;
  strokeColor: string;
  fontSize: string;
  duration: number;
  area: number;
  mode: 'scroll' | 'top' | 'bottom';
};

const DANMU_PREFERENCES_STORAGE_KEY = 'dtv_danmu_preferences_v1';
const DANMU_AREA_OPTIONS = [0.25, 0.5, 0.75] as const;
const PLAYER_VOLUME_STORAGE_KEY = 'dtv_player_volume_v1';

const sanitizeDanmuArea = (value: number): number => {
  return DANMU_AREA_OPTIONS.reduce((prev, curr) => Math.abs(curr - value) < Math.abs(prev - value) ? curr : prev, DANMU_AREA_OPTIONS[0]);
};

const loadStoredVolume = (): number | null => {
  if (typeof window === 'undefined' || !window.localStorage) {
    return null;
  }
  try {
    const raw = window.localStorage.getItem(PLAYER_VOLUME_STORAGE_KEY);
    if (raw === null) {
      return null;
    }
    const parsed = Number(raw);
    if (Number.isFinite(parsed)) {
      return Math.min(1, Math.max(0, parsed));
    }
    return null;
  } catch (error) {
    console.warn('[Player] Failed to load stored volume:', error);
    return null;
  }
};

const persistStoredVolume = (volume: number) => {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }
  try {
    const clamped = Math.min(1, Math.max(0, volume));
    window.localStorage.setItem(PLAYER_VOLUME_STORAGE_KEY, String(clamped));
  } catch (error) {
    console.warn('[Player] Failed to persist volume:', error);
  }
};

const createLucideIconSvg = (name: string, inner: string) => {
  return `<svg xmlns="http://www.w3.org/2000/svg" class="lucide lucide-${name}" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${inner}</svg>`;
};

const ICONS = {
  play: createLucideIconSvg('play', '<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z"></path>'),
  pause: createLucideIconSvg('pause', '<rect x="14" y="3" width="5" height="18" rx="1"></rect><rect x="5" y="3" width="5" height="18" rx="1"></rect>'),
  maximize2: createLucideIconSvg('maximize-2', '<path d="M15 3h6v6"></path><path d="m21 3-7 7"></path><path d="m3 21 7-7"></path><path d="M9 21H3v-6"></path>'),
  minimize2: createLucideIconSvg('minimize-2', '<path d="m14 10 7-7"></path><path d="M20 10h-6V4"></path><path d="m3 21 7-7"></path><path d="M4 14h6v6"></path>'),
  fullscreen: createLucideIconSvg('fullscreen', '<path d="M3 7V5a2 2 0 0 1 2-2h2"></path><path d="M17 3h2a2 2 0 0 1 2 2v2"></path><path d="M21 17v2a2 2 0 0 1-2 2h-2"></path><path d="M7 21H5a2 2 0 0 1-2-2v-2"></path><rect width="10" height="8" x="7" y="8" rx="1"></rect>'),
  pictureInPicture2: createLucideIconSvg('picture-in-picture-2', '<path d="M21 9V6a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v10c0 1.1.9 2 2 2h4"></path><rect width="10" height="7" x="12" y="13" rx="2"></rect>'),
  cog: createLucideIconSvg('cog', '<path d="M11 10.27 7 3.34"></path><path d="m11 13.73-4 6.93"></path><path d="M12 22v-2"></path><path d="M12 2v2"></path><path d="M14 12h8"></path><path d="m17 20.66-1-1.73"></path><path d="m17 3.34-1 1.73"></path><path d="M2 12h2"></path><path d="m20.66 17-1.73-1"></path><path d="m20.66 7-1.73 1"></path><path d="m3.34 17 1.73-1"></path><path d="m3.34 7 1.73 1"></path><circle cx="12" cy="12" r="2"></circle><circle cx="12" cy="12" r="8"></circle>'),
  rotateCcw: createLucideIconSvg('rotate-ccw', '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path><path d="M3 3v5h5"></path>'),
  volume2: createLucideIconSvg('volume-2', '<path d="M11 4.702a.705.705 0 0 0-1.203-.498L6.413 7.587A1.4 1.4 0 0 1 5.416 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.416a1.4 1.4 0 0 1 .997.413l3.383 3.384A.705.705 0 0 0 11 19.298z"></path><path d="M16 9a5 5 0 0 1 0 6"></path><path d="M19.364 18.364a9 9 0 0 0 0-12.728"></path>')
};

const loadDanmuPreferences = (): { enabled: boolean; settings: DanmuUserSettings } | null => {
  if (typeof window === 'undefined' || !window.localStorage) {
    return null;
  }
  try {
    const raw = window.localStorage.getItem(DANMU_PREFERENCES_STORAGE_KEY);
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') {
      return null;
    }
    const settings = parsed.settings ?? {};
    return {
      enabled: typeof parsed.enabled === 'boolean' ? parsed.enabled : true,
      settings: {
        color: typeof settings.color === 'string' ? settings.color : '#ffffff',
        strokeColor: typeof settings.strokeColor === 'string' ? settings.strokeColor : '#444444',
        fontSize: typeof settings.fontSize === 'string' ? settings.fontSize : '20px',
        duration: Number.isFinite(settings.duration) ? settings.duration : 10000,
        area: Number.isFinite(settings.area) ? sanitizeDanmuArea(settings.area) : 0.5,
        mode: settings.mode === 'top' || settings.mode === 'bottom' ? settings.mode : 'scroll',
      },
    };
  } catch (error) {
    console.warn('[DanmuPreferences] Failed to load preferences:', error);
    return null;
  }
};

const persistDanmuPreferences = (payload: { enabled: boolean; settings: DanmuUserSettings }) => {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }
  try {
    window.localStorage.setItem(DANMU_PREFERENCES_STORAGE_KEY, JSON.stringify(payload));
  } catch (error) {
    console.warn('[DanmuPreferences] Failed to persist preferences:', error);
  }
};

class DanmuToggleControl extends Plugin {
  static override pluginName = 'danmuToggle';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 4,
    disable: false,
    getState: (() => true) as () => boolean,
    onToggle: (async (_value: boolean) => {}) as (value: boolean) => Promise<void> | void,
  };

  private handleClick: ((event: Event) => void) | null = null;
  private isActive = true;

  override afterCreate() {
    if (this.config.disable) {
      return;
    }
    this.isActive = typeof this.config.getState === 'function' ? !!this.config.getState() : true;
    this.updateState();
    this.handleClick = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      this.toggle();
    };
    this.bind(['click', 'touchend'], this.handleClick);
  }

  override destroy() {
    if (this.handleClick) {
      this.unbind(['click', 'touchend'], this.handleClick);
      this.handleClick = null;
    }
  }

  override render() {
    if (this.config.disable) {
      return '';
    }
    return `<xg-icon class="xgplayer-danmu-toggle" title="" role="button" aria-pressed="${this.isActive}">
      <span class="danmu-toggle-label">弹幕</span>
      <span class="danmu-toggle-switch">
        <span class="switch-track"></span>
        <span class="switch-thumb"></span>
      </span>
    </xg-icon>`;
  }

  private toggle() {
    this.isActive = !this.isActive;
    this.updateState();
    const callback = this.config.onToggle;
    if (typeof callback === 'function') {
      callback(this.isActive);
    }
  }

  private updateState() {
    const root = this.root as HTMLElement | null;
    if (!root) {
      return;
    }
    root.classList.toggle('is-off', !this.isActive);
    root.setAttribute('aria-pressed', this.isActive ? 'true' : 'false');
  }

  setState(isActive: boolean) {
    this.isActive = isActive;
    this.updateState();
  }
}

class DanmuSettingsControl extends Plugin {
  static override pluginName = 'danmuSettings';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 4,
    disable: false,
    getSettings: (() => ({
      color: '#ffffff',
      strokeColor: '#444444',
      fontSize: '20px',
      duration: 10000,
      area: 0.5,
      mode: 'scroll',
    })) as () => DanmuUserSettings,
    onChange: (async (_partial: Partial<DanmuUserSettings>) => {}) as (partial: Partial<DanmuUserSettings>) => Promise<void> | void,
  };

  private panel: HTMLElement | null = null;
  private handleToggle: ((event: Event) => void) | null = null;
  private handleDocumentClick: ((event: MouseEvent) => void) | null = null;
  private handleHoverEnter: ((event: Event) => void) | null = null;
  private handleHoverLeave: ((event: Event) => void) | null = null;
  private hoverCloseTimer: ReturnType<typeof setTimeout> | null = null;
  private isOpen = false;
  private currentSettings: DanmuUserSettings = {
    color: '#ffffff',
    strokeColor: '#444444',
    fontSize: '20px',
    duration: 10000,
    area: 0.5,
    mode: 'scroll',
  };
  private textColorInput: HTMLInputElement | null = null;
  private strokeColorInput: HTMLInputElement | null = null;
  private fontSizeSlider: HTMLInputElement | null = null;
  private durationSlider: HTMLInputElement | null = null;
  private areaSlider: HTMLInputElement | null = null;

  override afterCreate() {
    if (this.config.disable) {
      return;
    }
    this.currentSettings = typeof this.config.getSettings === 'function'
      ? this.config.getSettings()
      : this.currentSettings;
    this.currentSettings.area = sanitizeDanmuArea(this.currentSettings.area);
    if (typeof this.currentSettings.strokeColor !== 'string') {
      this.currentSettings.strokeColor = '#444444';
    }

    this.createPanel();
    this.updateInputs();

    this.handleToggle = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      this.togglePanel();
    };

    this.bind(['click', 'touchend'], this.handleToggle);

    if (typeof document !== 'undefined') {
      this.handleDocumentClick = (event: MouseEvent) => {
        if (!this.root.contains(event.target as Node)) {
          this.closePanel();
        }
      };
      document.addEventListener('click', this.handleDocumentClick);
    }

    this.handleHoverEnter = () => {
      if (this.hoverCloseTimer) {
        clearTimeout(this.hoverCloseTimer);
        this.hoverCloseTimer = null;
      }
      this.openPanel();
    };
    this.handleHoverLeave = () => {
      if (this.hoverCloseTimer) {
        clearTimeout(this.hoverCloseTimer);
      }
      this.hoverCloseTimer = setTimeout(() => {
        this.hoverCloseTimer = null;
        this.closePanel();
      }, 220);
    };
    this.bind('mouseenter', this.handleHoverEnter);
    this.bind('mouseleave', this.handleHoverLeave);
  }

  override destroy() {
    if (this.handleToggle) {
      this.unbind(['click', 'touchend'], this.handleToggle);
      this.handleToggle = null;
    }
    if (this.handleDocumentClick) {
      document.removeEventListener('click', this.handleDocumentClick);
      this.handleDocumentClick = null;
    }
    if (this.handleHoverEnter) {
      this.unbind('mouseenter', this.handleHoverEnter);
      this.handleHoverEnter = null;
    }
    if (this.handleHoverLeave) {
      this.unbind('mouseleave', this.handleHoverLeave);
      this.handleHoverLeave = null;
    }
    if (this.hoverCloseTimer) {
      clearTimeout(this.hoverCloseTimer);
      this.hoverCloseTimer = null;
    }
    (this.root as HTMLElement | null)?.classList.remove('menu-open');
    this.panel?.remove();
    this.panel = null;
    this.textColorInput = null;
    this.strokeColorInput = null;
    this.fontSizeSlider = null;
    this.durationSlider = null;
    this.areaSlider = null;
  }

  override render() {
    if (this.config.disable) {
      return '';
    }
    return `<xg-icon class="xgplayer-danmu-settings" title="">
      ${ICONS.cog}
    </xg-icon>`;
  }

  private createPanel() {
    this.panel = document.createElement('div');
    this.panel.className = 'xgplayer-danmu-settings-panel';
    this.panel.innerHTML = `
      <div class="settings-shell">
        <div class="settings-body">
          <div class="settings-row settings-row-color">
            <span class="settings-label">颜色</span>
            <input class="danmu-setting-color" type="color" value="${this.currentSettings.color}">
          </div>
          <div class="settings-row settings-row-color">
            <span class="settings-label">描边</span>
            <input class="danmu-setting-stroke-color" type="color" value="${this.currentSettings.strokeColor}">
          </div>
          <div class="settings-row">
            <label>字体 <span class="settings-value font-size-value">${this.currentSettings.fontSize}</span></label>
            <input class="danmu-setting-font-range" type="range" min="14" max="30" step="2" value="${parseInt(this.currentSettings.fontSize, 10)}">
          </div>
          <div class="settings-row">
            <label>速度 <span class="settings-value speed-value">${this.formatDurationLabel(this.currentSettings.duration)}</span></label>
            <input class="danmu-setting-duration-range" type="range" min="3000" max="20000" step="500" value="${this.currentSettings.duration}">
          </div>
          <div class="settings-row">
            <label>显示区域 <span class="settings-value area-value">${this.formatAreaLabel(this.currentSettings.area)}</span></label>
            <input class="danmu-setting-area-range" type="range" min="0.25" max="0.75" step="0.25" value="${this.currentSettings.area}">
          </div>
        </div>
      </div>
    `;
    this.root.appendChild(this.panel);

    this.panel.addEventListener('click', (event) => {
      event.stopPropagation();
    });
    this.panel.addEventListener('pointerdown', (event) => {
      event.stopPropagation();
    });
    this.panel.addEventListener('mousedown', (event) => {
      event.stopPropagation();
    });

    this.textColorInput = this.panel.querySelector<HTMLInputElement>('.danmu-setting-color');
    this.strokeColorInput = this.panel.querySelector<HTMLInputElement>('.danmu-setting-stroke-color');
    this.fontSizeSlider = this.panel.querySelector<HTMLInputElement>('.danmu-setting-font-range');
    this.durationSlider = this.panel.querySelector<HTMLInputElement>('.danmu-setting-duration-range');
    this.areaSlider = this.panel.querySelector<HTMLInputElement>('.danmu-setting-area-range');

    this.textColorInput?.addEventListener('input', (event) => {
      const value = (event.target as HTMLInputElement).value;
      this.currentSettings.color = value;
      this.emitChange({ color: value });
    });
    this.strokeColorInput?.addEventListener('input', (event) => {
      const value = (event.target as HTMLInputElement).value;
      this.currentSettings.strokeColor = value;
      this.emitChange({ strokeColor: value });
    });

    const handleRange = (el: HTMLInputElement | null, key: keyof DanmuUserSettings, transform: (value: string) => unknown, displaySelector: string, formatter: (value: number) => string) => {
      const updateDisplay = (value: number) => {
        const label = this.panel?.querySelector<HTMLSpanElement>(displaySelector);
        if (label) {
          label.textContent = formatter(value);
        }
      };
      el?.addEventListener('input', (event) => {
        const rawValue = (event.target as HTMLInputElement).value;
        const numericValue = Number(rawValue);
        updateDisplay(numericValue);
        const nextValue = transform(rawValue);
        (this.currentSettings as Record<string, unknown>)[key as string] = nextValue;
        this.emitChange({ [key]: nextValue } as Partial<DanmuUserSettings>);
        this.updateSliderVisual(el);
      });
      if (el) {
        updateDisplay(Number(el.value));
        this.updateSliderVisual(el);
      }
    };

    handleRange(
      this.fontSizeSlider,
      'fontSize',
      (value) => `${Math.min(30, Math.max(14, Number(value)))}px`,
      '.font-size-value',
      (value) => `${Math.min(30, Math.max(14, value))}px`,
    );

    handleRange(
      this.durationSlider,
      'duration',
      (value) => {
        const numeric = Number(value);
        const clamped = Number.isFinite(numeric) ? Math.min(20000, Math.max(3000, numeric)) : 10000;
        return clamped;
      },
      '.speed-value',
      (value) => this.formatDurationLabel(value),
    );

    handleRange(
      this.areaSlider,
      'area',
      (value) => {
        const numeric = Number(value);
        return sanitizeDanmuArea(numeric);
      },
      '.area-value',
      (value) => this.formatAreaLabel(value),
    );
  }

  private updateSliderVisual(el: HTMLInputElement | null) {
    if (!el) {
      return;
    }
    const min = Number(el.min) || 0;
    const max = Number(el.max) || 100;
    const value = Number(el.value);
    const clamped = Math.min(max, Math.max(min, value));
    const percent = max === min ? 0 : ((clamped - min) / (max - min)) * 100;
    el.style.background = `linear-gradient(90deg, var(--player-accent) ${percent}%, rgba(255, 255, 255, 0.15) ${percent}%)`;
  }

  private togglePanel() {
    if (this.isOpen) {
      this.closePanel();
    } else {
      this.openPanel();
    }
  }

  private openPanel() {
    if (!this.panel || this.isOpen) {
      return;
    }
    if (this.hoverCloseTimer) {
      clearTimeout(this.hoverCloseTimer);
      this.hoverCloseTimer = null;
    }
    this.isOpen = true;
    this.panel.classList.add('show');
    this.root.classList.add('menu-open');
    this.updateInputs();
  }

  private closePanel() {
    if (!this.panel) {
      return;
    }
    if (this.hoverCloseTimer) {
      clearTimeout(this.hoverCloseTimer);
      this.hoverCloseTimer = null;
    }
    this.isOpen = false;
    this.panel.classList.remove('show');
    this.root.classList.remove('menu-open');
  }

  private updateInputs() {
    if (!this.panel) {
      return;
    }
    if (this.textColorInput) {
      this.textColorInput.value = this.currentSettings.color;
    }
    if (this.strokeColorInput) {
      this.strokeColorInput.value = this.currentSettings.strokeColor;
    }
    if (this.fontSizeSlider) {
      const numericFont = parseInt(this.currentSettings.fontSize, 10);
      this.fontSizeSlider.value = String(Math.min(30, Math.max(14, numericFont)));
      const fontLabel = this.panel.querySelector<HTMLSpanElement>('.font-size-value');
      if (fontLabel) {
        fontLabel.textContent = `${Math.min(30, Math.max(14, numericFont))}px`;
      }
      this.updateSliderVisual(this.fontSizeSlider);
    }
    if (this.durationSlider) {
      const durationValue = Math.min(20000, Math.max(3000, this.currentSettings.duration));
      this.durationSlider.value = String(durationValue);
      const speedLabel = this.panel.querySelector<HTMLSpanElement>('.speed-value');
      if (speedLabel) {
        speedLabel.textContent = this.formatDurationLabel(durationValue);
      }
      this.updateSliderVisual(this.durationSlider);
    }
    if (this.areaSlider) {
      const areaValue = sanitizeDanmuArea(this.currentSettings.area);
      this.areaSlider.value = String(areaValue);
      const areaLabel = this.panel.querySelector<HTMLSpanElement>('.area-value');
      if (areaLabel) {
        areaLabel.textContent = this.formatAreaLabel(areaValue);
      }
      this.updateSliderVisual(this.areaSlider);
    }
  }

  private formatDurationLabel(value: number): string {
    const clamped = Math.min(20000, Math.max(3000, value));
    if (clamped <= 4500) {
      return '极快';
    }
    if (clamped <= 7500) {
      return '很快';
    }
    if (clamped <= 10000) {
      return '标准';
    }
    if (clamped <= 14000) {
      return '稍慢';
    }
    return '慢速';
  }

  private formatAreaLabel(value: number): string {
    const clamped = sanitizeDanmuArea(value);
    if (clamped <= 0.25) {
      return '上 1/4';
    }
    if (clamped <= 0.5) {
      return '上 1/2';
    }
    return '上 3/4';
  }

  private emitChange(partial: Partial<DanmuUserSettings>) {
    const callback = this.config.onChange;
    if (typeof callback === 'function') {
      callback(partial);
    }
  }

  setSettings(settings: Partial<DanmuUserSettings>) {
    const normalized: Partial<DanmuUserSettings> = { ...settings };
    if (typeof normalized.area === 'number') {
      normalized.area = sanitizeDanmuArea(normalized.area);
    }
    if (typeof normalized.strokeColor !== 'undefined' && typeof normalized.strokeColor !== 'string') {
      delete (normalized as any).strokeColor;
    }
    this.currentSettings = {
      ...this.currentSettings,
      ...normalized,
    };
    if (typeof this.currentSettings.strokeColor !== 'string') {
      this.currentSettings.strokeColor = '#444444';
    }
    this.updateInputs();
  }
}

class VolumeControl extends Plugin {
  static override pluginName = 'volumeControl';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_LEFT,
    index: 3,
    disable: false,
  };

  private volumeIcon: HTMLElement | null = null;
  private slider: HTMLInputElement | null = null;
  private valueLabel: HTMLElement | null = null;
  private onVolumeChange: ((value: number) => void) | null = null;
  private handleIconClick: ((event: Event) => void) | null = null;
  private previousVolume = 1;

  override render() {
    if (this.config.disable) {
      return '';
    }
    return `<xg-icon class="xgplayer-volume-control" title="音量/静音切换">
      <div class="volume-icon">
        ${ICONS.volume2}
      </div>
      <input class="volume-slider" type="range" min="0" max="100" step="1" value="100">
      <span class="volume-value">100%</span>
    </xg-icon>`;
  }

  override afterCreate() {
    if (this.config.disable) {
      return;
    }
    this.volumeIcon = this.find('.volume-icon') as HTMLElement | null;
    this.slider = this.find('.volume-slider') as HTMLInputElement | null;
    this.valueLabel = this.find('.volume-value') as HTMLElement | null;

    const updateUI = (volume: number) => {
      const clamped = Math.max(0, Math.min(1, volume));
      if (clamped > 0) {
        this.previousVolume = clamped;
      }
      if (this.slider) {
        this.slider.value = String(Math.round(clamped * 100));
        this.updateSliderVisual(this.slider);
      }
      if (this.valueLabel) {
        this.valueLabel.textContent = `${Math.round(clamped * 100)}%`;
      }
      if (this.volumeIcon) {
        this.volumeIcon.setAttribute('data-muted', clamped === 0 ? 'true' : 'false');
      }
    };

    const storedVolume = loadStoredVolume();
    if (storedVolume !== null) {
      if (storedVolume > 0) {
        this.previousVolume = storedVolume;
      }
      this.player.volume = storedVolume;
      this.player.muted = storedVolume === 0 ? true : this.player.muted;
    } else {
      const initial = this.player.volume ?? 1;
      if (initial > 0) {
        this.previousVolume = initial;
      }
    }

    updateUI(this.player.volume ?? storedVolume ?? 1);

    this.slider?.addEventListener('input', (event) => {
      const value = Number((event.target as HTMLInputElement).value);
      const clampedPercent = Math.max(0, Math.min(100, value));
      const normalized = clampedPercent / 100;
      if (normalized === 0) {
        this.player.muted = true;
      } else {
        this.player.muted = false;
        this.previousVolume = normalized;
      }
      this.player.volume = normalized;
      updateUI(normalized);
      persistStoredVolume(normalized);
    });

    this.handleIconClick = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      const currentVolume = this.player.volume ?? 0;
      if (currentVolume > 0) {
        this.previousVolume = currentVolume;
        this.player.volume = 0;
        this.player.muted = true;
        updateUI(0);
        persistStoredVolume(0);
      } else {
        const restoreVolume = this.previousVolume > 0 ? this.previousVolume : 1;
        this.player.muted = false;
        this.player.volume = restoreVolume;
        updateUI(restoreVolume);
        persistStoredVolume(restoreVolume);
      }
    };

    this.volumeIcon?.addEventListener('click', this.handleIconClick);
    if (this.volumeIcon) {
      this.volumeIcon.setAttribute('title', '点击静音 / 取消静音');
      this.volumeIcon.style.cursor = 'pointer';
    }

    this.onVolumeChange = () => {
      const current = this.player.volume ?? 1;
      updateUI(current);
      persistStoredVolume(current);
    };
    this.player.on('volumechange', this.onVolumeChange);
  }

  override destroy() {
    if (this.handleIconClick && this.volumeIcon) {
      this.volumeIcon.removeEventListener('click', this.handleIconClick);
      this.handleIconClick = null;
    }
    if (this.onVolumeChange) {
      this.player.off('volumechange', this.onVolumeChange);
      this.onVolumeChange = null;
    }
    this.volumeIcon = null;
    this.slider = null;
    this.valueLabel = null;
  }

  private updateSliderVisual(el: HTMLInputElement | null) {
    if (!el) {
      return;
    }
    const value = Number(el.value);
    const percent = Math.max(0, Math.min(100, value));
    el.style.background = `linear-gradient(90deg, var(--player-accent) ${percent}%, rgba(255, 255, 255, 0.15) ${percent}%)`;
  }
}

class RefreshControl extends Plugin {
  static override pluginName = 'refreshControl';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_LEFT,
    index: 2,
    disable: false,
    onClick: null as (() => void) | null,
  };

  private handleClick: ((event: Event) => void) | null = null;
  private isLoading = false;

  override afterCreate() {
    if (this.config.disable) {
      return;
    }
    this.handleClick = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      if (this.isLoading) {
        return;
      }
      if (typeof this.config.onClick === 'function') {
        this.config.onClick();
      }
    };
    this.bind(['click', 'touchend'], this.handleClick);
  }

  override destroy() {
    if (this.handleClick) {
      this.unbind(['click', 'touchend'], this.handleClick);
      this.handleClick = null;
    }
    this.setLoading(false);
  }

  override render() {
    if (this.config.disable) {
      return '';
    }
    return `<xg-icon class="xgplayer-refresh-control" title="刷新">
      ${ICONS.rotateCcw}
    </xg-icon>`;
  }

  setLoading(isLoading: boolean) {
    this.isLoading = isLoading;
    const root = this.root as HTMLElement | null;
    if (!root) {
      return;
    }
    root.classList.toggle('is-loading', isLoading);
    if (isLoading) {
      root.setAttribute('aria-disabled', 'true');
    } else {
      root.removeAttribute('aria-disabled');
    }
  }
}

class QualityControl extends Plugin {
  static override pluginName = 'qualityControl';
  static override defaultConfig = {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 5,
    disable: false,
    options: [] as string[],
    getCurrent: (() => '') as () => string,
    onSelect: (async (_value: string) => {}) as (value: string) => Promise<void> | void,
  };

  private dropdown: HTMLElement | null = null;
  private handleToggle: ((event: Event) => void) | null = null;
  private handleDocumentClick: ((event: MouseEvent) => void) | null = null;
  private handleHoverEnter: ((event: Event) => void) | null = null;
  private handleHoverLeave: ((event: Event) => void) | null = null;
  private hoverCloseTimer: ReturnType<typeof setTimeout> | null = null;
  private isSwitching = false;

  override afterCreate() {
    if (this.config.disable) {
      return;
    }

    this.createDropdown();
    this.updateLabel(this.getCurrent());

    this.handleToggle = (event: Event) => {
      event.preventDefault();
      event.stopPropagation();
      if (this.isSwitching) {
        return;
      }
      this.toggleDropdown();
    };
    this.bind(['click', 'touchend'], this.handleToggle);

    if (typeof document !== 'undefined') {
      this.handleDocumentClick = (event: MouseEvent) => {
        if (!this.root.contains(event.target as Node)) {
          this.hideDropdown();
        }
      };
      document.addEventListener('click', this.handleDocumentClick);
    }

    this.handleHoverEnter = () => {
      if (this.hoverCloseTimer) {
        clearTimeout(this.hoverCloseTimer);
        this.hoverCloseTimer = null;
      }
      if (!this.isSwitching) {
        this.openDropdown();
      }
    };
    this.handleHoverLeave = () => {
      if (this.hoverCloseTimer) {
        clearTimeout(this.hoverCloseTimer);
      }
      this.hoverCloseTimer = setTimeout(() => {
        this.hoverCloseTimer = null;
        this.hideDropdown();
      }, 220);
    };
    this.bind('mouseenter', this.handleHoverEnter);
    this.bind('mouseleave', this.handleHoverLeave);
  }

  override destroy() {
    if (this.handleToggle) {
      this.unbind(['click', 'touchend'], this.handleToggle);
      this.handleToggle = null;
    }
    if (this.handleDocumentClick) {
      document.removeEventListener('click', this.handleDocumentClick);
      this.handleDocumentClick = null;
    }
    if (this.handleHoverEnter) {
      this.unbind('mouseenter', this.handleHoverEnter);
      this.handleHoverEnter = null;
    }
    if (this.handleHoverLeave) {
      this.unbind('mouseleave', this.handleHoverLeave);
      this.handleHoverLeave = null;
    }
    if (this.hoverCloseTimer) {
      clearTimeout(this.hoverCloseTimer);
      this.hoverCloseTimer = null;
    }
    if (this.dropdown) {
      this.dropdown.remove();
      this.dropdown = null;
    }
    this.setSwitching(false);
  }

  override render() {
    if (this.config.disable) {
      return '';
    }
    const current = this.getCurrent();
    return `<xg-icon class="xgplayer-quality-control" title="">
      <span class="quality-label">${current}</span>
      <svg class="quality-caret" width="10" height="10" viewBox="0 0 10 10" fill="none">
        <path d="M2.5 3.5L5 6l2.5-2.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </xg-icon>`;
  }

  updateLabel(label: string) {
    const textEl = this.find('.quality-label') as HTMLElement | null;
    if (textEl) {
      textEl.textContent = label;
    }
    this.updateActiveState(label);
  }

  setOptions(options: string[]) {
    this.config.options = options;
    this.populateDropdown();
  }

  private getCurrent() {
    return typeof this.config.getCurrent === 'function' ? this.config.getCurrent() : '';
  }

  private createDropdown() {
    this.dropdown = document.createElement('div');
    this.dropdown.className = 'xgplayer-quality-dropdown';
    this.root.appendChild(this.dropdown);
    this.populateDropdown();
  }

  private populateDropdown() {
    if (!this.dropdown) {
      return;
    }
    this.dropdown.innerHTML = '';
    const options: string[] = Array.isArray(this.config.options) ? this.config.options : [];
    options.forEach((option) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'xgplayer-quality-item';
      btn.innerHTML = `
        <span class="quality-name">${option}</span>
        <svg class="quality-check" width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M3 6.5l2 2 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      `;
      btn.addEventListener('click', (event) => {
        event.stopPropagation();
        event.preventDefault();
        if (this.isSwitching) {
          return;
        }
        let actionResult: Promise<void> | void;
        try {
          const callback = this.config.onSelect;
          actionResult = typeof callback === 'function' ? callback(option) : undefined;
        } catch (error) {
          console.error('[QualityControl] onSelect error:', error);
          actionResult = undefined;
        }
        Promise.resolve(actionResult).finally(() => {
          this.hideDropdown();
          this.updateLabel(this.getCurrent());
        });
      });
      this.dropdown!.appendChild(btn);
    });
    this.updateActiveState(this.getCurrent());
    this.applySwitchingState();
  }

  private toggleDropdown() {
    if (this.isSwitching) {
      return;
    }
    if (this.dropdown?.classList.contains('show')) {
      this.hideDropdown();
    } else {
      this.openDropdown();
    }
  }

  private openDropdown() {
    if (this.isSwitching || !this.dropdown) {
      return;
    }
    if (this.hoverCloseTimer) {
      clearTimeout(this.hoverCloseTimer);
      this.hoverCloseTimer = null;
    }
    this.dropdown.classList.add('show');
    this.root.classList.add('menu-open');
    this.updateActiveState(this.getCurrent());
  }

  private hideDropdown() {
    if (this.hoverCloseTimer) {
      clearTimeout(this.hoverCloseTimer);
      this.hoverCloseTimer = null;
    }
    if (this.dropdown) {
      this.dropdown.classList.remove('show');
    }
    this.root.classList.remove('menu-open');
  }

  private updateActiveState(current: string) {
    if (!this.dropdown) {
      return;
    }
    const items = this.dropdown.querySelectorAll<HTMLButtonElement>('.xgplayer-quality-item');
    items.forEach((item) => {
      const label = item.querySelector('.quality-name')?.textContent?.trim();
      item.classList.toggle('active', label === current);
    });
  }

  setSwitching(isSwitching: boolean) {
    this.isSwitching = isSwitching;
    this.applySwitchingState();
    if (isSwitching) {
      this.hideDropdown();
    }
  }

  private applySwitchingState() {
    const root = this.root as HTMLElement | null;
    if (root) {
      root.classList.toggle('is-switching', this.isSwitching);
    }
    if (this.dropdown) {
      this.dropdown.classList.toggle('disabled', this.isSwitching);
      const buttons = this.dropdown.querySelectorAll<HTMLButtonElement>('.xgplayer-quality-item');
      buttons.forEach((button) => {
        button.disabled = this.isSwitching;
      });
    }
  }
}

const props = defineProps<{
  roomId: string | null;
  platform: StreamingPlatform;
  isFollowed?: boolean;
  streamUrl?: string | null; // Primarily for Douyin direct URL
  title?: string | null;
  anchorName?: string | null;
  avatar?: string | null;
  isLive?: boolean | null;
  initialError?: string | null; // Added to accept pre-determined errors like "主播未开播"
  cookie?: string | null; // Optional cookie for platforms like Bilibili
}>();

const emit = defineEmits<{
  (e: 'follow', streamer: any): void;
  (e: 'unfollow', roomId: string): void;
  (e: 'close-player'): void;
  (e: 'fullscreen-change', isFullscreen: boolean): void;
  (e: 'request-refresh-details'): void;
  (e: 'request-player-reload'): void;
}>();

const playerContainerRef = ref<HTMLDivElement | null>(null);
const playerInstance = shallowRef<Player | null>(null);
const refreshControlPlugin = shallowRef<RefreshControl | null>(null);
const qualityControlPlugin = shallowRef<QualityControl | null>(null);
const danmuTogglePlugin = shallowRef<DanmuToggleControl | null>(null);
const danmuSettingsPlugin = shallowRef<DanmuSettingsControl | null>(null);
const volumeControlPlugin = shallowRef<VolumeControl | null>(null);
const danmuInstance = shallowRef<DanmuOverlayInstance | null>(null);
const danmakuMessages = ref<DanmakuMessage[]>([]);
const isDanmakuListenerActive = ref(false); // Tracks if a danmaku listener is supposed to be running
let unlistenDanmakuFn: (() => void) | null = null;

const isLoadingStream = ref(true);
const streamError = ref<string | null>(null);
const isOfflineError = ref(false); // Added to track '主播未开播' state

// Reactive state for streamer info, initialized by props, potentially updated by internal fetches (for Douyin)
const playerTitle = ref(props.title);
const playerAnchorName = ref(props.anchorName);
const playerAvatar = ref(props.avatar);
const playerIsLive = ref(props.isLive);

const isInNativePlayerFullscreen = ref(false); // New: Tracks Artplayer element's native fullscreen
const isInWebFullscreen = ref(false);
const isFullScreen = ref(false); // True if EITHER native player OR web fullscreen is active

const isDanmuEnabled = ref(true);
const danmuSettings = reactive<DanmuUserSettings>({
  color: '#ffffff',
  strokeColor: '#444444',
  fontSize: '20px',
  duration: 10000,
  area: 0.5,
  mode: 'scroll',
});

const storedDanmuPreferences = loadDanmuPreferences();
if (storedDanmuPreferences) {
  isDanmuEnabled.value = storedDanmuPreferences.enabled;
  Object.assign(danmuSettings, storedDanmuPreferences.settings);
}

// OS specific states
const osName = ref<string>('');

// 画质切换相关
const qualityOptions = ['原画', '高清', '标清'] as const;

const resolveStoredQuality = (platform?: StreamingPlatform | null): string => {
  if (!platform) {
    return '原画';
  }
  if (typeof window === 'undefined') {
    return '原画';
  }
  try {
    const saved = window.localStorage.getItem(`${platform}_preferred_quality`);
    if (saved && qualityOptions.includes(saved as (typeof qualityOptions)[number])) {
      return saved;
    }
  } catch (error) {
    console.warn('[Player] Failed to read stored quality preference:', error);
  }
  return '原画';
};

const currentQuality = ref<string>(resolveStoredQuality(props.platform));
const isQualitySwitching = ref(false);
const isRefreshingStream = ref(false);

function resetFullscreenState() {
  isInNativePlayerFullscreen.value = false;
  isInWebFullscreen.value = false;
  isFullScreen.value = false;
  try {
    document.documentElement.classList.remove('web-fs-active');
  } catch (error) {
    console.warn('[Player] Failed to reset web fullscreen flag:', error);
  }
}

function updateFullscreenFlag() {
  isFullScreen.value = isInNativePlayerFullscreen.value || isInWebFullscreen.value;
  emit('fullscreen-change', isFullScreen.value);
}

function destroyPlayerInstance() {
  const player = playerInstance.value;
  if (player) {
    try {
      player.destroy();
    } catch (error) {
      console.error('[Player] Error destroying xgplayer instance:', error);
    }
    const overlayHost = player.root?.querySelector('.player-danmu-overlay') as HTMLElement | null;
    overlayHost?.remove();
  }
  playerInstance.value = null;

  const danmu = danmuInstance.value;
  if (danmu) {
    try {
      danmu.stop?.();
    } catch (error) {
      console.error('[Player] Error stopping danmu overlay:', error);
    }
    danmuInstance.value = null;
  }

  refreshControlPlugin.value = null;
  qualityControlPlugin.value = null;
  danmuTogglePlugin.value = null;
  danmuSettingsPlugin.value = null;
  volumeControlPlugin.value = null;

  resetFullscreenState();
}

function ensureDanmuOverlayHost(player: Player): HTMLElement | null {
  const root = player.root as HTMLElement | undefined;
  if (!root) {
    return null;
  }

  let host = root.querySelector('.player-danmu-overlay') as HTMLElement | null;
  if (!host) {
    host = document.createElement('div');
    host.className = 'player-danmu-overlay';
  }

  const videoContainer = root.querySelector('xg-video-container');
  if (videoContainer && host.parentElement !== videoContainer) {
    videoContainer.appendChild(host);
  } else if (!videoContainer && host.parentElement !== root) {
    root.appendChild(host);
  } else if (!host.parentElement) {
    root.appendChild(host);
  }

  return host;
}

function createDanmuOverlay(player: Player | null) {
  if (!player) {
    return null;
  }

  const overlayHost = ensureDanmuOverlayHost(player);
  if (!overlayHost) {
    return null;
  }

  overlayHost.innerHTML = '';
  overlayHost.style.setProperty('--danmu-stroke-color', danmuSettings.strokeColor);

  try {
    const overlay = new DanmuJs({
      container: overlayHost,
      player: player.video || player.media || undefined,
      comments: [],
      mouseControl: false,
      defaultOff: false,
      channelSize: 36,
      containerStyle: {
        pointerEvents: 'none',
      },
    });

    overlay.start?.();
    danmuInstance.value = overlay;
    applyDanmuOverlayPreferences(overlay);
    syncDanmuEnabledState(overlay);
    return overlay;
  } catch (error) {
    console.error('[Player] Failed to initialize danmu.js overlay:', error);
    danmuInstance.value = null;
    return null;
  }
}

function applyDanmuOverlayPreferences(overlay: DanmuOverlayInstance | null) {
  if (!overlay) {
    return;
  }
  const fontSizeValue = parseInt(danmuSettings.fontSize, 10);
  if (!Number.isNaN(fontSizeValue)) {
    try {
      overlay.setFontSize?.(fontSizeValue);
    } catch (error) {
      console.warn('[Player] Failed to apply danmu font size:', error);
    }
  }
  try {
    const areaValue = sanitizeDanmuArea(danmuSettings.area);
    overlay.setArea?.({ start: 0, end: areaValue });
  } catch (error) {
    console.warn('[Player] Failed to apply danmu area:', error);
  }
  try {
    overlay.setAllDuration?.('scroll', danmuSettings.duration);
    overlay.setAllDuration?.('top', danmuSettings.duration);
    overlay.setAllDuration?.('bottom', danmuSettings.duration);
  } catch (error) {
    // Non-critical for players that do not support bulk duration updates
  }
  try {
    overlay.setOpacity?.(isDanmuEnabled.value ? 1 : 0);
  } catch (error) {
    // Non-critical
  }
  try {
    const host = playerInstance.value?.root?.querySelector('.player-danmu-overlay') as HTMLElement | null;
    host?.style.setProperty('--danmu-stroke-color', danmuSettings.strokeColor);
  } catch (error) {
    console.warn('[Player] Failed to apply danmu stroke color:', error);
  }
}

function syncDanmuEnabledState(overlay: DanmuOverlayInstance | null) {
  if (!overlay) {
    return;
  }
  try {
    if (isDanmuEnabled.value) {
      overlay.play?.();
      overlay.setOpacity?.(1);
      overlay.show?.('scroll');
      overlay.show?.('top');
      overlay.show?.('bottom');
    } else {
      overlay.pause?.();
      overlay.setOpacity?.(0);
    }
  } catch (error) {
    console.warn('[Player] Failed updating danmu enabled state:', error);
  }
}

function arrangeControlClusters(player: Player | null) {
  if (!player || !player.root) {
    return;
  }
  const root = player.root as HTMLElement;
  const run = () => {
    try {
      groupPrimaryControls(root);
      groupDanmuControls(root);
    } catch (error) {
      console.warn('[Player] Failed to arrange player controls:', error);
    }
  };
  if (typeof window !== 'undefined' && typeof window.requestAnimationFrame === 'function') {
    window.requestAnimationFrame(run);
  } else {
    run();
  }
}

function groupPrimaryControls(root: HTMLElement) {
  const leftControls = root.querySelector('.xgplayer-controls-left');
  if (!leftControls) {
    return;
  }
  const playEl = leftControls.querySelector('.xgplayer-play');
  const refreshEl = leftControls.querySelector('.xgplayer-refresh-control');
  const volumeEl = leftControls.querySelector('.xgplayer-volume-control');
  if (!playEl && !refreshEl && !volumeEl) {
    return;
  }
  let cluster = leftControls.querySelector<HTMLElement>('.xgplayer-left-cluster');
  if (!cluster) {
    cluster = document.createElement('div');
    cluster.className = 'xgplayer-left-cluster';
    leftControls.insertBefore(cluster, leftControls.firstChild);
  }
  [playEl, refreshEl, volumeEl].forEach((element) => {
    if (element instanceof HTMLElement && element.parentElement !== cluster) {
      cluster?.appendChild(element);
    }
  });
}

function groupDanmuControls(root: HTMLElement) {
  const rightControls = root.querySelector('.xgplayer-controls-right');
  if (!rightControls) {
    return;
  }
  const toggleEl = rightControls.querySelector('.xgplayer-danmu-toggle');
  const settingsEl = rightControls.querySelector('.xgplayer-danmu-settings');
  if (!(toggleEl instanceof HTMLElement) || !(settingsEl instanceof HTMLElement)) {
    return;
  }
  let cluster = rightControls.querySelector<HTMLElement>('.danmu-control-group');
  if (!cluster) {
    cluster = document.createElement('div');
    cluster.className = 'danmu-control-group';
    rightControls.insertBefore(cluster, toggleEl);
  }
  if (toggleEl.parentElement !== cluster) {
    cluster.appendChild(toggleEl);
  }
  if (settingsEl.parentElement !== cluster) {
    cluster.appendChild(settingsEl);
  }
}

async function mountXgPlayer(
  streamUrl: string,
  platformCode: StreamingPlatform,
  roomId: string,
) {
  await nextTick();

  if (!playerContainerRef.value) {
    streamError.value = '播放器容器初始化失败。';
    return;
  }

  playerContainerRef.value.innerHTML = '';

  const player = new Player({
    el: playerContainerRef.value,
    url: streamUrl,
    isLive: true,
    autoplay: true,
    playsinline: true,
    lang: 'zh-cn',
    width: '100%',
    height: '100%',
    videoFillMode: 'contain',
    closeVideoClick: false,
    keyShortcut: true,
    volume: false as unknown as number,
    pip: {
      position: POSITIONS.CONTROLS_RIGHT,
      index: 3,
      showIcon: true,
    },
    cssFullscreen: {
      index: 2,
    },
    playbackRate: false,
    controls: {
      mode: 'normal',
    },
    icons: {
      play: ICONS.play,
      pause: ICONS.pause,
      fullscreen: ICONS.maximize2,
      exitFullscreen: ICONS.minimize2,
      cssFullscreen: ICONS.fullscreen,
      exitCssFullscreen: ICONS.minimize2,
      pipIcon: ICONS.pictureInPicture2,
      pipIconExit: ICONS.pictureInPicture2,
    },
    plugins: [FlvPlugin],
    flv: {
      isLive: true,
      cors: true,
      autoCleanupSourceBuffer: true,
      enableWorker: true,
      stashInitialSize: 128,
      lazyLoad: true,
      lazyLoadMaxDuration: 30,
      deferLoadAfterSourceOpen: true,
    },
  });

  playerInstance.value = player;
  const storedPlayerVolume = loadStoredVolume();
  if (storedPlayerVolume !== null) {
    player.volume = storedPlayerVolume;
    player.muted = storedPlayerVolume === 0 ? true : player.muted;
  }

  refreshControlPlugin.value = player.registerPlugin(RefreshControl, {
    position: POSITIONS.CONTROLS_LEFT,
    index: 2,
    onClick: () => {
      void reloadCurrentStream('refresh');
    },
  }) as RefreshControl;

  volumeControlPlugin.value = player.registerPlugin(VolumeControl, {
    position: POSITIONS.CONTROLS_LEFT,
    index: 3,
  }) as VolumeControl;

  danmuTogglePlugin.value = player.registerPlugin(DanmuToggleControl, {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 4,
    getState: () => isDanmuEnabled.value,
    onToggle: (enabled: boolean) => {
      isDanmuEnabled.value = enabled;
    },
  }) as DanmuToggleControl;

  danmuSettingsPlugin.value = player.registerPlugin(DanmuSettingsControl, {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 4.2,
    getSettings: () => ({
      color: danmuSettings.color,
      strokeColor: danmuSettings.strokeColor,
      fontSize: danmuSettings.fontSize,
      duration: danmuSettings.duration,
      area: danmuSettings.area,
      mode: danmuSettings.mode,
    }),
    onChange: (partial: Partial<DanmuUserSettings>) => {
      if (partial.color) {
        danmuSettings.color = partial.color;
      }
      if (partial.strokeColor) {
        danmuSettings.strokeColor = partial.strokeColor;
      }
      if (partial.fontSize) {
        danmuSettings.fontSize = partial.fontSize;
      }
      if (typeof partial.duration === 'number') {
        danmuSettings.duration = partial.duration;
      }
      if (typeof partial.area === 'number') {
        danmuSettings.area = sanitizeDanmuArea(partial.area);
      }
      if (partial.mode) {
        danmuSettings.mode = partial.mode;
      }
    },
  }) as DanmuSettingsControl;

  qualityControlPlugin.value = player.registerPlugin(QualityControl, {
    position: POSITIONS.CONTROLS_RIGHT,
    index: 5,
    options: [...qualityOptions],
    getCurrent: () => currentQuality.value,
    onSelect: async (option: string) => {
      if (option === currentQuality.value) {
        return;
      }
      await switchQuality(option);
    },
  }) as QualityControl;
  qualityControlPlugin.value?.setOptions([...qualityOptions]);
  qualityControlPlugin.value?.updateLabel(currentQuality.value);

  arrangeControlClusters(player);

  let overlayInstance = createDanmuOverlay(player);

  player.on('ready', async () => {
    arrangeControlClusters(player);
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    try {
      if (roomId) {
        await startCurrentDanmakuListener(platformCode, roomId, overlayInstance);
      }
    } catch (error) {
      console.error('[Player] Failed starting danmaku listener after ready:', error);
    }
    overlayInstance?.play?.();
    updateFullscreenFlag();
  });

  player.on('play', () => {
    overlayInstance?.play?.();
  });

  player.on('pause', () => {
    overlayInstance?.pause?.();
  });

  player.on('destroy', () => {
    overlayInstance?.stop?.();
    overlayInstance = null;
  });

  player.on('error', (error: any) => {
    console.error('[Player] xgplayer error:', error);
    streamError.value = `播放器错误: ${error?.message || error}`;
  });

  player.on('enterFullscreen', () => {
    isInNativePlayerFullscreen.value = true;
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    overlayInstance?.play?.();
    updateFullscreenFlag();
  });

  player.on('exitFullscreen', () => {
    isInNativePlayerFullscreen.value = false;
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    updateFullscreenFlag();
  });

  player.on('enterFullscreenWeb', () => {
    isInWebFullscreen.value = true;
    try {
      document.documentElement.classList.add('web-fs-active');
    } catch (error) {
      console.warn('[Player] Failed to set web fullscreen flag:', error);
    }
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    overlayInstance?.play?.();
    arrangeControlClusters(player);
    updateFullscreenFlag();
  });

  player.on('exitFullscreenWeb', () => {
    isInWebFullscreen.value = false;
    try {
      document.documentElement.classList.remove('web-fs-active');
    } catch (error) {
      console.warn('[Player] Failed to clear web fullscreen flag:', error);
    }
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    arrangeControlClusters(player);
    updateFullscreenFlag();
  });

  player.on('cssFullscreen_change', (isCssFullscreen: boolean) => {
    isInWebFullscreen.value = isCssFullscreen;
    try {
      if (isCssFullscreen) {
        document.documentElement.classList.add('web-fs-active');
      } else {
        document.documentElement.classList.remove('web-fs-active');
      }
    } catch (error) {
      console.warn('[Player] Failed toggling css fullscreen flag:', error);
    }
    ensureDanmuOverlayHost(player);
    overlayInstance = overlayInstance ?? createDanmuOverlay(player);
    if (isCssFullscreen) {
      overlayInstance?.play?.();
    }
    arrangeControlClusters(player);
    updateFullscreenFlag();
  });
}


async function initializePlayerAndStream(
  pRoomId: string, 
  pPlatform: StreamingPlatform,
  _pStreamUrlProp?: string | null, 
  isRefresh: boolean = false,
  oldRoomIdForCleanup?: string | null,
  oldPlatformForCleanup?: StreamingPlatform | null
) {
  isLoadingStream.value = true;
  streamError.value = null;
  isOfflineError.value = false;

  // Detect OS (synchronous call)
  osName.value = platform();

  if (!isRefresh) {
    danmakuMessages.value = [];
  }

  if (props.initialError && props.initialError.includes('主播未开播')) {
    streamError.value = props.initialError;
    isOfflineError.value = true;
    playerTitle.value = props.title;
    playerAnchorName.value = props.anchorName;
    playerAvatar.value = props.avatar;
    playerIsLive.value = false;
    destroyPlayerInstance();
    isLoadingStream.value = false;
    return;
  }

  if (oldRoomIdForCleanup && oldPlatformForCleanup !== undefined && oldPlatformForCleanup !== null) {
    await stopCurrentDanmakuListener(oldPlatformForCleanup, oldRoomIdForCleanup);
    if (oldPlatformForCleanup === StreamingPlatform.DOUYU) {
      await stopDouyuProxy();
    }
    if (oldPlatformForCleanup === StreamingPlatform.HUYA) {
      await stopHuyaProxy();
    }
  } else {
    await stopCurrentDanmakuListener();
  }

  destroyPlayerInstance();

  try {
    let streamConfig: { streamUrl: string; streamType: string | undefined };

    if (pPlatform === StreamingPlatform.DOUYU) {
      if (playerIsLive.value === false) {
        streamError.value = streamError.value || '主播未开播。';
        isOfflineError.value = true;
        isLoadingStream.value = false;
        return;
      }
      streamConfig = await getDouyuStreamConfig(pRoomId, currentQuality.value);
    } else if (pPlatform === StreamingPlatform.DOUYIN) {
      const douyinConfig = await fetchAndPrepareDouyinStreamConfig(pRoomId, currentQuality.value);
      playerTitle.value = douyinConfig.title;
      playerAnchorName.value = douyinConfig.anchorName;
      playerAvatar.value = douyinConfig.avatar;
      playerIsLive.value = douyinConfig.isLive;

      if (douyinConfig.initialError || !douyinConfig.isLive || !douyinConfig.streamUrl) {
        streamError.value = douyinConfig.initialError || '主播未开播或无法获取直播流。';
        isOfflineError.value = true;
        playerIsLive.value = false;
        isLoadingStream.value = false;
        console.warn(`[Player] Douyin config error or not live: ${streamError.value}`);
        return;
      }

      streamConfig = { streamUrl: douyinConfig.streamUrl, streamType: douyinConfig.streamType };
    } else if (pPlatform === StreamingPlatform.HUYA) {
      streamConfig = await getHuyaStreamConfig(pRoomId, currentQuality.value);
    } else if (pPlatform === StreamingPlatform.BILIBILI) {
      streamConfig = await getBilibiliStreamConfig(pRoomId, currentQuality.value, props.cookie || undefined);
    } else {
      throw new Error(`不支持的平台: ${pPlatform}`);
    }

    isLoadingStream.value = false;
    await mountXgPlayer(streamConfig.streamUrl, pPlatform, pRoomId);
  } catch (error: any) {
    console.error(`[Player] Error initializing stream for ${pPlatform} room ${pRoomId}:`, error);
    destroyPlayerInstance();

    const errorMessage = error?.message || '加载直播流失败，请稍后再试。';

    if (errorMessage.includes('主播未开播')) {
      streamError.value = errorMessage;
      isOfflineError.value = true;

      try {
        if (pPlatform === StreamingPlatform.HUYA) {
          const result: any = await invoke('get_huya_unified_cmd', { roomId: pRoomId, quality: currentQuality.value });
          await ensureProxyStarted();
          playerTitle.value = result?.title ?? props.title;
          playerAnchorName.value = result?.nick ?? props.anchorName;
          playerAvatar.value = proxify((result?.avatar ?? props.avatar ?? '') as string);
        } else if (pPlatform === StreamingPlatform.BILIBILI) {
          const payload = { args: { room_id_str: pRoomId } };
          const savedCookie = (typeof localStorage !== 'undefined') ? (localStorage.getItem('bilibili_cookie') || null) : null;
          const res: any = await invoke('fetch_bilibili_streamer_info', { payload, cookie: savedCookie });
          await ensureProxyStarted();
          playerTitle.value = res?.title ?? props.title;
          playerAnchorName.value = res?.anchor_name ?? props.anchorName;
          playerAvatar.value = proxify((res?.avatar ?? props.avatar ?? '') as string);
        }
      } catch (infoError) {
        console.warn('[Player] Failed to fetch basic streamer info for offline page:', infoError);
      }
    } else {
      streamError.value = errorMessage;
      isOfflineError.value = false;
    }

    isLoadingStream.value = false;
  }
}
async function startCurrentDanmakuListener(platform: StreamingPlatform, roomId: string, danmuOverlay: DanmuOverlayInstance | null) {
  if (!roomId) {
    return;
  }
  if (isDanmakuListenerActive.value) {
    return;
  }

  isDanmakuListenerActive.value = true;
  if (!danmuOverlay) {
    console.warn('[Player] Danmu overlay instance missing, incoming danmaku will not render on video but list will update.');
  }

  try {
    const renderOptions = {
      shouldDisplay: () => isDanmuEnabled.value,
      buildCommentOptions: () => ({
        duration: danmuSettings.duration,
        mode: danmuSettings.mode,
        style: {
          color: danmuSettings.color,
          fontSize: danmuSettings.fontSize,
          '--danmu-stroke-color': danmuSettings.strokeColor,
        },
      }),
    };
    let stopFn: (() => void) | null = null;
    if (platform === StreamingPlatform.DOUYU) {
      stopFn = await startDouyuDanmakuListener(roomId, danmuOverlay, danmakuMessages, renderOptions); 
    } else if (platform === StreamingPlatform.DOUYIN) {
      stopFn = await startDouyinDanmakuListener(roomId, danmuOverlay, danmakuMessages, renderOptions);
    } else if (platform === StreamingPlatform.HUYA) {
      stopFn = await startHuyaDanmakuListener(roomId, danmuOverlay, danmakuMessages, renderOptions);
    } else if (platform === StreamingPlatform.BILIBILI) {
      stopFn = await startBilibiliDanmakuListener(roomId, danmuOverlay, danmakuMessages, props.cookie || undefined, renderOptions);
    }

    if (stopFn) {
      unlistenDanmakuFn = stopFn;
      const successMessage: DanmakuMessage = {
        id: `system-conn-${Date.now()}`,
        nickname: '系统消息',
        content: '弹幕连接成功！',
        isSystem: true,
        type: 'success',
        color: '#28a745'
      };
      danmakuMessages.value.push(successMessage);

    } else {
      console.warn(`[Player] Danmaku listener for ${platform}/${roomId} did not return a stop function.`);
      isDanmakuListenerActive.value = false; 
    }
  } catch (error) {
    console.error(`[Player] Failed to start danmaku listener for ${platform}/${roomId}:`, error);
    isDanmakuListenerActive.value = false; 

    const errorMessage: DanmakuMessage = {
      id: `system-err-${Date.now()}`,
      nickname: '系统消息',
      content: '弹幕连接失败，请尝试刷新播放器。',
      isSystem: true,
      type: 'error',
      color: '#dc3545'
    };
    danmakuMessages.value.push(errorMessage);
  }
}

async function stopCurrentDanmakuListener(platform?: StreamingPlatform, roomId?: string | null | undefined) {
  if (platform) {
    if (platform === StreamingPlatform.DOUYU) {
      await stopDouyuDanmaku(roomId!, unlistenDanmakuFn); 
    } else if (platform === StreamingPlatform.DOUYIN) {
      await stopDouyinDanmaku(unlistenDanmakuFn);
    } else if (platform === StreamingPlatform.HUYA) {
      await stopHuyaDanmaku(unlistenDanmakuFn);
    } else if (platform === StreamingPlatform.BILIBILI) {
      await stopBilibiliDanmaku(unlistenDanmakuFn);
    }
    if (unlistenDanmakuFn) { 
        unlistenDanmakuFn = null;
    }

  } else if (unlistenDanmakuFn) {
    console.warn('[Player] stopCurrentDanmakuListener called without platform, but a global unlistenDanmakuFn exists. Calling it now.');
    try {
      unlistenDanmakuFn();
      unlistenDanmakuFn = null; // Nullify after successful call
    } catch (e) {
      console.error('[Player] Error executing fallback unlistenDanmakuFn:', e);
      // Still nullify to prevent repeated errors with a bad function ref
      unlistenDanmakuFn = null; 
    }
  }

  isDanmakuListenerActive.value = false;
}

const retryInitialization = async () => {
  await reloadCurrentStream('refresh');
};

// 画质切换函数
const switchQuality = async (quality: string) => {
  if (isQualitySwitching.value) {
    return;
  }
  if (!qualityOptions.includes(quality as (typeof qualityOptions)[number])) {
    return;
  }
  if (!props.roomId || props.platform == null) {
    emit('request-player-reload');
    return;
  }
  if (quality === currentQuality.value) {
    return;
  }

  isQualitySwitching.value = true;
  const previousQuality = currentQuality.value;

  try {
    currentQuality.value = quality;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(`${props.platform}_preferred_quality`, quality);
    }
    await reloadCurrentStream('quality');
    console.log(`[Player] 画质切换完成: ${quality}`);
  } catch (error) {
    console.error('[Player] 画质切换失败:', error);
    currentQuality.value = previousQuality;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(`${props.platform}_preferred_quality`, previousQuality);
    }
  } finally {
    isQualitySwitching.value = false;
  }
};

// 初始化画质偏好
const initializeQualityPreference = () => {
  currentQuality.value = resolveStoredQuality(props.platform);
};

async function reloadCurrentStream(trigger: 'refresh' | 'quality' = 'refresh') {
  if (isLoadingStream.value) {
    return;
  }
  if (!props.roomId || props.platform == null) {
    emit('request-player-reload');
    return;
  }
  const isRefreshAction = trigger === 'refresh';
  if (isRefreshAction) {
    isRefreshingStream.value = true;
  }
  try {
    await initializePlayerAndStream(
      props.roomId,
      props.platform,
      props.streamUrl ?? null,
      true,
      props.roomId,
      props.platform,
    );
  } finally {
    if (isRefreshAction) {
      isRefreshingStream.value = false;
    }
  }
  if (trigger === 'quality') {
    qualityControlPlugin.value?.updateLabel(currentQuality.value);
  }
}

watch(isRefreshingStream, (isLoading) => {
  refreshControlPlugin.value?.setLoading(isLoading);
});

watch(refreshControlPlugin, (plugin) => {
  plugin?.setLoading(isRefreshingStream.value);
});

const getDanmuSettingsSnapshot = (): DanmuUserSettings => ({
  color: danmuSettings.color,
  strokeColor: danmuSettings.strokeColor,
  fontSize: danmuSettings.fontSize,
  duration: danmuSettings.duration,
  area: sanitizeDanmuArea(danmuSettings.area),
  mode: danmuSettings.mode,
});

const persistCurrentDanmuPreferences = () => {
  persistDanmuPreferences({
    enabled: isDanmuEnabled.value,
    settings: getDanmuSettingsSnapshot(),
  });
};

watch(isQualitySwitching, (isSwitching) => {
  qualityControlPlugin.value?.setSwitching(isSwitching);
});

watch(qualityControlPlugin, (plugin) => {
  plugin?.setSwitching(isQualitySwitching.value);
});

watch(isDanmuEnabled, (enabled) => {
  danmuTogglePlugin.value?.setState(enabled);
  syncDanmuEnabledState(danmuInstance.value);
  persistCurrentDanmuPreferences();
});

watch(danmuTogglePlugin, (plugin) => {
  plugin?.setState(isDanmuEnabled.value);
});

watch(danmuSettingsPlugin, (plugin) => {
  if (!plugin) {
    return;
  }
  plugin.setSettings({
    color: danmuSettings.color,
    strokeColor: danmuSettings.strokeColor,
    fontSize: danmuSettings.fontSize,
    duration: danmuSettings.duration,
    area: sanitizeDanmuArea(danmuSettings.area),
    mode: danmuSettings.mode,
  });
});

watch(() => danmuSettings.color, (color) => {
  danmuSettingsPlugin.value?.setSettings({ color });
  persistCurrentDanmuPreferences();
});

watch(() => danmuSettings.strokeColor, (strokeColor) => {
  danmuSettingsPlugin.value?.setSettings({ strokeColor });
  applyDanmuOverlayPreferences(danmuInstance.value);
  persistCurrentDanmuPreferences();
});

watch(() => danmuSettings.fontSize, (fontSize) => {
  danmuSettingsPlugin.value?.setSettings({ fontSize });
  applyDanmuOverlayPreferences(danmuInstance.value);
  persistCurrentDanmuPreferences();
});

watch(() => danmuSettings.duration, (duration) => {
  danmuSettingsPlugin.value?.setSettings({ duration });
  applyDanmuOverlayPreferences(danmuInstance.value);
  persistCurrentDanmuPreferences();
});

watch(() => danmuSettings.area, (area) => {
  const normalizedArea = sanitizeDanmuArea(area);
  if (normalizedArea !== area) {
    danmuSettings.area = normalizedArea;
    return;
  }
  danmuSettingsPlugin.value?.setSettings({ area: normalizedArea });
  applyDanmuOverlayPreferences(danmuInstance.value);
  persistCurrentDanmuPreferences();
});

watch(danmuInstance, (instance) => {
  applyDanmuOverlayPreferences(instance);
  syncDanmuEnabledState(instance);
});

watch(currentQuality, (quality) => {
  qualityControlPlugin.value?.updateLabel(quality);
});

watch([() => props.roomId, () => props.platform, () => props.streamUrl, () => props.avatar, () => props.title, () => props.anchorName, () => props.isLive], 
  async ([newRoomId, newPlatform, newStreamUrl, _newAvatar, _newTitle, _newAnchorName, _newIsLive], [oldRoomId, oldPlatform, _oldStreamUrl, _oldAvatar, _oldTitle, _oldAnchorName, _oldIsLive]) => {
    // Update internal reactive streamer info when props change
    if (newPlatform === StreamingPlatform.DOUYU) { // Only update from props if Douyu
      playerTitle.value = _newTitle;
      playerAnchorName.value = _newAnchorName;
      playerAvatar.value = _newAvatar;
      if (_newIsLive !== undefined) {
          playerIsLive.value = _newIsLive;
      }
    }

    // Initial error from props (e.g., DouyuPlayerView determined offline before MainPlayer rendered)
    if (newRoomId && newPlatform) {
      // Always reset isOfflineError when props change significantly unless initialError prop dictates it
      if (!(props.initialError && props.initialError.includes('主播未开播'))) {
        isOfflineError.value = false; 
      }

      // Determine if re-initialization is needed
      const isInitialCall = oldRoomId === undefined && oldPlatform === undefined;
      const hasSwitchedStream = newRoomId !== oldRoomId || newPlatform !== oldPlatform;
      // Douyin might also re-init if its specific stream URL prop changes (though less likely with current proxy setup)
      const douyinStreamUrlChanged = newPlatform === StreamingPlatform.DOUYIN && newStreamUrl !== _oldStreamUrl;

      const needsReInit = hasSwitchedStream || isInitialCall || douyinStreamUrlChanged;

      if (needsReInit) {
        // 在重新初始化时更新画质偏好
        initializeQualityPreference();
        // Pass oldRoomId and oldPlatform (which might be undefined on initial call)
        // initializePlayerAndStream will handle undefined cleanup IDs gracefully.
        initializePlayerAndStream(newRoomId, newPlatform, newStreamUrl, false, oldRoomId, oldPlatform);
      }
    } else if (!newRoomId) { 
      if (oldRoomId && oldPlatform !== null && oldPlatform !== undefined) { 
        await stopCurrentDanmakuListener(oldPlatform, oldRoomId);
        if (oldPlatform === StreamingPlatform.DOUYU) {
          await stopDouyuProxy();
        }
        if (oldPlatform === StreamingPlatform.HUYA) {
          await stopHuyaProxy();
        }
      } else {
        await stopCurrentDanmakuListener();
      }

      destroyPlayerInstance();

      isLoadingStream.value = false;
      danmakuMessages.value = [];
      streamError.value = null;
      isOfflineError.value = false; 
    }
    if (!props.roomId || props.platform == null) {
      if (props.initialError) {
        if (props.initialError.includes('主播未开播')) {
            streamError.value = props.initialError;
            isOfflineError.value = true;
        } else {
            streamError.value = props.initialError;
            isOfflineError.value = false; // Ensure it's not marked as offline for other errors
        }
      }
      isLoadingStream.value = false;
    }
}, 
{ immediate: true }
);

onMounted(async () => {
  // 初始化画质偏好
  initializeQualityPreference();
  
  if (!props.roomId || props.platform == null) {
    if (props.initialError) {
      if (props.initialError.includes('主播未开播')) {
          streamError.value = props.initialError;
          isOfflineError.value = true;
      } else {
          streamError.value = props.initialError;
          isOfflineError.value = false; // Ensure it's not marked as offline for other errors
      }
    }
    isLoadingStream.value = false;
  }

  persistCurrentDanmuPreferences();
});

onUnmounted(async () => {
  const platformToStop: StreamingPlatform = props.platform;
  const roomIdToStop: string | null = props.roomId;
  await stopCurrentDanmakuListener(platformToStop, roomIdToStop);

  if (props.platform === StreamingPlatform.DOUYU) {
    await stopDouyuProxy();
  }
  if (props.platform === StreamingPlatform.HUYA) {
    await stopHuyaProxy();
  }

  destroyPlayerInstance();
  danmakuMessages.value = []; 
});

</script>
