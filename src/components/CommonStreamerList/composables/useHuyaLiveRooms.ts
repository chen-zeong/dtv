import { ref } from 'vue';
import type { Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { CommonStreamer } from '../../../platforms/common/streamerTypes';
import { useImageProxy } from '../../FollowsList/useProxy';

export interface UseHuyaLiveRoomsOptions {
  defaultPageSize?: number; // platform-specific default, Huya=120
}

export function useHuyaLiveRooms(
  gidRef: Ref<string | null>,
  options: UseHuyaLiveRoomsOptions = { defaultPageSize: 120 }
) {
  const rooms = ref<CommonStreamer[]>([]) as Ref<CommonStreamer[]>;
  const isLoading = ref(false);
  const isLoadingMore = ref(false);
  const error = ref<string | null>(null);
  const currentPage = ref(1);
  const hasMore = ref(true);
  const pageSize = options.defaultPageSize ?? 120;

  const { proxify, ensureProxyStarted } = useImageProxy();

  const mapHuyaItemToCommonStreamer = (item: any): CommonStreamer => {
    const viewers = typeof item.lUserCount === 'number' ? item.lUserCount : 0;
    return {
      room_id: item.room_id?.toString() || item.lProfileRoom?.toString() || '',
      title: item.title || item.sIntroduction || '',
      nickname: item.nickname || item.sNick || '',
      avatar: proxify(item.avatar || item.sAvatar180 || ''),
      room_cover: proxify(item.room_cover || item.sScreenshot || ''),
      viewer_count_str: item.viewer_count_str || (viewers ? `${viewers}` : '0'),
      platform: 'huya',
    };
  };

  const fetchRooms = async (pageNo: number, isLoadMore: boolean = false) => {
    const gid = gidRef.value;
    if (!gid) {
      rooms.value = [];
      hasMore.value = false;
      currentPage.value = 1;
      return;
    }

    if (isLoadMore) isLoadingMore.value = true; else isLoading.value = true;
    error.value = null;

    // Ensure proxy server is started before mapping covers
    await ensureProxyStarted();

    try {
      const resp = await invoke<{ error: number; msg?: string; data?: any[] }>('fetch_huya_live_list', {
        iGid: gid,
        iPageNo: pageNo,
        iPageSize: pageSize,
      });

      if (resp.error !== 0 || !Array.isArray(resp.data)) {
        throw new Error(resp.msg || '虎牙接口返回错误');
      }
      const newRooms = resp.data.map(mapHuyaItemToCommonStreamer);

      if (isLoadMore) {
        rooms.value.push(...newRooms);
      } else {
        rooms.value = newRooms;
      }

      hasMore.value = newRooms.length === pageSize;
      currentPage.value = pageNo + 1;

    } catch (e: any) {
      console.error('[useHuyaLiveRooms] invoke error', e);
      error.value = e?.message || '加载失败';
      if (!isLoadMore) {
        rooms.value = [];
        hasMore.value = false;
      }
    } finally {
      if (isLoadMore) isLoadingMore.value = false; else isLoading.value = false;
    }
  };

  const loadInitialRooms = async () => {
    rooms.value = [];
    hasMore.value = true;
    currentPage.value = 1;
    error.value = null;
    isLoading.value = true;
    await fetchRooms(1, false);
  };

  const loadMoreRooms = async () => {
    if (hasMore.value && !isLoading.value && !isLoadingMore.value) {
      await fetchRooms(currentPage.value, true);
    }
  };

  return {
    rooms,
    isLoading,
    isLoadingMore,
    error,
    hasMore,
    loadInitialRooms,
    loadMoreRooms,
  };
}