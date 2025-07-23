// 主播信息接口
export interface Streamer {
  rid: string        // 房间ID
  roomName: string   // 房间名称
  nickname: string   // 主播昵称
  roomSrc: string    // 房间缩略图
  avatar: string     // 主播头像
  hn: string         // 观看人数
}

// 直播列表响应接口
export interface LiveListResponse {
  error: number
  msg?: string
  data: {
    list: Streamer[]
    total: number
  }
} 