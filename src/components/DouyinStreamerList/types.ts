export interface DouyinStreamer {
  web_rid: string;        // Room ID, from Douyin demo
  title: string;          // Room title, from Douyin demo
  nickname: string;        // Streamer nickname (hoping it comes from the list API)
  avatar?: string;         // Streamer avatar (hoping it comes from the list API)
  room_cover: string;      // Room thumbnail/preview (hoping it comes from the list API)
  viewer_count_str?: string; // Viewer count as string (e.g., "1.2W在线")
  platform: 'douyin';
}


export interface DouyinPartitionRoomsResponse {
  rooms: any[];
  has_more?: boolean;
  next_offset?: number;
} 