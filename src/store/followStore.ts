import { defineStore } from 'pinia';
import type { FollowedStreamer, Platform } from '../platforms/common/types';

interface FollowState {
  followedStreamers: FollowedStreamer[];
}

export const useFollowStore = defineStore('follow', {
  state: (): FollowState => ({
    followedStreamers: [], // Initialize with an empty array or load from localStorage
  }),
  getters: {
    isFollowed: (state: FollowState) => (platform: Platform, id: string): boolean => {
      return state.followedStreamers.some((s: FollowedStreamer) => s.platform === platform && s.id === id);
    },
    getFollowedStreamers: (state: FollowState): FollowedStreamer[] => {
      return state.followedStreamers;
    }
  },
  actions: {
    // Action to load followed streamers, e.g., from localStorage
    loadFollowedStreamers() {
      const storedFollows = localStorage.getItem('followedStreamers');
      if (storedFollows) {
        try {
          this.followedStreamers = JSON.parse(storedFollows) as FollowedStreamer[];
        } catch (e) {
          console.error('Error parsing followedStreamers from localStorage', e);
          this.followedStreamers = [];
        }
      }
    },
    // Action to save followed streamers
    _saveFollows() {
      try {
        localStorage.setItem('followedStreamers', JSON.stringify(this.followedStreamers));
      } catch (e) {
        console.error('Error saving followedStreamers to localStorage', e);
      }
    },
    followStreamer(streamer: FollowedStreamer) {
      if (!this.isFollowed(streamer.platform, streamer.id)) {
        this.followedStreamers.push(streamer);
        this._saveFollows();
      }
    },
    unfollowStreamer(platform: Platform, id: string) {
      const index = this.followedStreamers.findIndex((s: FollowedStreamer) => s.platform === platform && s.id === id);
      if (index !== -1) {
        this.followedStreamers.splice(index, 1);
        this._saveFollows();
      }
    },
    // Action to update the order of followed streamers (e.g., after drag-and-drop)
    updateOrder(newList: FollowedStreamer[]) {
      this.followedStreamers = newList;
      this._saveFollows();
    },
    // You might also need an action to update details of a followed streamer (e.g., live status)
    updateStreamerDetails(updatedStreamer: Partial<FollowedStreamer> & { platform: Platform; id: string }) {
      const index = this.followedStreamers.findIndex((s: FollowedStreamer) => s.platform === updatedStreamer.platform && s.id === updatedStreamer.id);
      if (index !== -1) {
        this.followedStreamers[index] = { ...this.followedStreamers[index], ...updatedStreamer };
        this._saveFollows();
      }
    }
  },
}); 