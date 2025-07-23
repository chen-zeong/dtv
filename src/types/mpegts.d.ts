declare module 'mpegts.js' {
  interface MediaDataSourceConfig {
    type: string;
    url: string;
    isLive?: boolean;
    hasAudio?: boolean;
    hasVideo?: boolean;
    cors?: boolean;
    withCredentials?: boolean;
    enableStashBuffer?: boolean;
    stashInitialSize?: number;
    autoCleanupSourceBuffer?: boolean;
    liveBufferLatencyChasing?: boolean;
    fixAudioTimestampGap?: boolean;
  }

  interface Config {
    enableWorker?: boolean;
    lazyLoad?: boolean;
    stashInitialSize?: number;
    liveBufferLatencyChasing?: boolean;
    liveSync?: boolean;
  }

  interface Events {
    ERROR: string;
    LOADING_COMPLETE: string;
    RECOVERED_EARLY_EOF: string;
    MEDIA_INFO: string;
    METADATA_ARRIVED: string;
    SCRIPTDATA_ARRIVED: string;
    STATISTICS_INFO: string;
  }

  type ErrorType = number;
  type ErrorDetails = {
    code: number;
    msg: string;
  };

  class Player {
    constructor(mediaDataSource: MediaDataSourceConfig, config?: Config);
    attachMediaElement(element: HTMLMediaElement): void;
    load(): void;
    unload(): void;
    play(): Promise<void>;
    pause(): void;
    destroy(): void;
    on(event: string, listener: (type: ErrorType, details: ErrorDetails) => void): void;
    detachMediaElement(): void;
  }

  const Events: Events;
  function createPlayer(mediaDataSource: MediaDataSourceConfig, config?: Config): Player;
  function isSupported(): boolean;
} 