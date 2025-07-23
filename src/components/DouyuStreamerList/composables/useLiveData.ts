import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Streamer {
  rid: string
  roomName: string
  nickname: string
  roomSrc: string
  avatar: string
  hn: string // Viewers count as string
  isLive?: boolean // This was in FrontendLiveRoomInfo, ensure it's populated
}

// This interface should match FrontendLiveListDataWrapper from Rust
interface LiveListDataWrapper {
  list: Streamer[]
  total?: number // total might be optional or not available for all endpoints
  page_count?: number // page_count might also be available
}

// This interface should match FrontendLiveListResponse from Rust
interface LiveListApiResponse {
  error: number
  msg?: string
  data?: LiveListDataWrapper // data is optional
}

const PAGE_SIZE = 20;

export function useLiveData() { // Removed initialCate2 argument
  const streamers = ref<Streamer[]>([])
  const currentPage = ref(0) // 0-indexed for logic, convert to 1-indexed for API if needed
  const hasMore = ref(true)
  const isLoading = ref(false)

  const fetchStreamers = async (categoryType: 'cate2' | 'cate3', categoryId: string, pageToFetch: number) => {
    if (!categoryId) {
      streamers.value = [];
      currentPage.value = 0;
      hasMore.value = false;
      isLoading.value = false; // Ensure loading is stopped
      return;
    }
    
    isLoading.value = true;

    let command = '';
    let params: any = {};

    if (categoryType === 'cate2') {
      command = 'fetch_live_list';
      params = {
        cate2: categoryId,
        offset: pageToFetch * PAGE_SIZE,
        limit: PAGE_SIZE
      };
    } else if (categoryType === 'cate3') {
      command = 'fetch_live_list_for_cate3';
      params = {
        cate3Id: categoryId,
        page: pageToFetch + 1, // Douyu API for cate3 is 1-indexed page
        limit: PAGE_SIZE 
      };
    } else {
      console.error('[useLiveData] Unknown category type:', categoryType);
      isLoading.value = false;
      return;
    }

    try {
      // Invoke now expects the deserialized object directly based on Rust function return type
      const apiResponse = await invoke<LiveListApiResponse>(command, params);
      
      if (apiResponse.error === 0 && apiResponse.data) {
        const newList = apiResponse.data.list || [];
        if (pageToFetch === 0) {
          streamers.value = newList;
        } else {
          streamers.value = [...streamers.value, ...newList];
        }
        
        // Determine hasMore based on total or if fewer items than PAGE_SIZE were returned
        if (apiResponse.data.total !== undefined) {
          const totalFetched = (pageToFetch + 1) * PAGE_SIZE;
          hasMore.value = apiResponse.data.total > totalFetched && newList.length > 0;
        } else if (apiResponse.data.page_count !== undefined) {
            hasMore.value = pageToFetch +1 < apiResponse.data.page_count && newList.length > 0;
        } else {
          // If no total or page_count, assume more if we got a full page
          hasMore.value = newList.length === PAGE_SIZE;
        }

      } else {
        console.error('[useLiveData] Error from API or no data:', apiResponse.msg || 'Unknown API error');
        if (pageToFetch === 0) streamers.value = []; 
        hasMore.value = false;
      }
    } catch (error) {
      console.error('[useLiveData] Failed to invoke command or process response:', error);
      if (pageToFetch === 0) streamers.value = [];
      hasMore.value = false;
    } finally {
      isLoading.value = false;
    }
  }
  
  const loadNextPage = (categoryType: 'cate2' | 'cate3', categoryId: string) => {
    if (!isLoading.value && hasMore.value && categoryId) {
      currentPage.value++; // Increment current page
      fetchStreamers(categoryType, categoryId, currentPage.value); 
    }
  }

  const resetAndFetch = (categoryType: 'cate2' | 'cate3', categoryId: string) => {
    currentPage.value = 0;
    streamers.value = [];
    hasMore.value = true; // Assume more initially
    fetchStreamers(categoryType, categoryId, 0);
  }
  
  return {
    streamers,
    currentPage, // Primarily for knowing which page was last fetched/attempted
    hasMore,
    isLoading,
    fetchStreamers, // Direct fetch, used by component
    loadNextPage,    // For infinite scroll trigger
    resetAndFetch    // For category changes
  }
} 