# 抖音直播API接口分析

## 1. 概述

抖音直播API接口主要用于获取直播流URL、房间信息、主播详情等功能。与斗鱼相比，抖音的API相对简单，但在画质选择和流处理方面有其独特的实现方式。

## 2. 核心接口列表

### 2.1 直播流获取接口
- **接口地址**：`https://live.douyin.com/webcast/room/web/enter/`
- **请求方式**：GET
- **功能**：获取抖音直播间的流媒体URL

### 2.2 房间信息接口
- **接口地址**：`https://live.douyin.com/webcast/room/web/enter/`
- **请求方式**：GET
- **功能**：获取直播间基本信息

### 2.3 主播详情接口
- **接口地址**：通过房间信息接口获取
- **功能**：获取主播昵称、头像等信息

### 2.4 弹幕监听接口
- **协议**：WebSocket
- **功能**：实时接收直播间弹幕消息

## 3. 直播流获取接口详解

### 3.1 接口地址
```
https://live.douyin.com/webcast/room/web/enter/
```

### 3.2 请求方式
GET

### 3.3 请求参数
| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| web_rid | string | 是 | 直播间ID |
| room_id_str | string | 是 | 直播间ID字符串 |
| enter_from | string | 否 | 进入来源，默认"web_live" |
| cookie_enabled | string | 否 | Cookie启用状态，默认"true" |
| screen_width | string | 否 | 屏幕宽度 |
| screen_height | string | 否 | 屏幕高度 |
| browser_language | string | 否 | 浏览器语言 |
| browser_platform | string | 否 | 浏览器平台 |
| browser_name | string | 否 | 浏览器名称 |
| browser_version | string | 否 | 浏览器版本 |

### 3.4 请求头
```
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36
Referer: https://live.douyin.com/
Cookie: 相关认证Cookie
```

### 3.5 返回数据结构

```json
{
  "status_code": 0,
  "data": {
    "data": [
      {
        "title": "直播间标题",
        "status": 2,
        "stream_url": {
          "flv_pull_url": {
            "FULL_HD1": "高清FLV流地址",
            "HD1": "标清FLV流地址"
          },
          "hls_pull_url_map": {
            "FULL_HD1": "高清HLS流地址",
            "HD1": "标清HLS流地址"
          },
          "live_core_sdk_data": {
            "pull_data": {
              "stream_data": "{\"data\":{\"origin\":{\"main\":{\"flv\":\"原画FLV地址\"}}}}"
            }
          }
        }
      }
    ],
    "user": {
      "nickname": "主播昵称",
      "avatar_thumb": {
        "url_list": ["头像URL"]
      }
    }
  }
}
```

## 4. 画质切换技术实现

### 4.1 画质等级说明

抖音支持多种画质等级，主要包括：
- **原画(origin)**：最高画质，来源于live_core_sdk_data
- **超清(FULL_HD1)**：1080p画质
- **高清(HD1)**：720p画质
- **标清(SD1)**：480p画质

### 4.2 画质获取优先级

```rust
// 画质获取的优先级顺序
1. live_core_sdk_data.pull_data.stream_data 中的 FLV 流（原画质量最高）
2. hls_pull_url_map 中的 FULL_HD1（超清）
3. hls_pull_url_map 中的 HD1（高清）
4. flv_pull_url 中的其他画质
```

### 4.3 画质切换实现方案

```rust
// 画质选择函数
pub async fn get_douyin_stream_with_quality(
    room_id: &str, 
    preferred_quality: &str
) -> Result<String, String> {
    // 获取完整的流信息
    let stream_info = get_douyin_live_stream_url(room_id).await?;
    
    // 根据用户偏好选择画质
    match preferred_quality {
        "原画" | "origin" => {
            // 优先从 live_core_sdk_data 获取原画
            get_origin_quality_stream(stream_info).await
        },
        "超清" | "full_hd" => {
            // 从 hls_pull_url_map 获取 FULL_HD1
            get_full_hd_stream(stream_info).await
        },
        "高清" | "hd" => {
            // 从 hls_pull_url_map 获取 HD1
            get_hd_stream(stream_info).await
        },
        _ => {
            // 默认获取最佳可用画质
            get_best_available_stream(stream_info).await
        }
    }
}

// 获取原画质量流
fn get_origin_quality_stream(stream_info: &StreamUrlContainer) -> Option<String> {
    if let Some(sdk_data) = &stream_info.live_core_sdk_data {
        if let Some(pull_data) = &sdk_data.pull_data {
            if let Some(stream_data_str) = &pull_data.stream_data {
                // 解析嵌套的JSON字符串
                if let Ok(inner_data) = serde_json::from_str::<InnerStreamDataWrapper>(stream_data_str) {
                    if let Some(data) = inner_data.data {
                        // 按优先级获取画质
                        let stream_options = [
                            data.origin.as_ref(),
                            data.hd.as_ref(),
                            data.sd.as_ref(),
                        ];
                        
                        for quality_detail in stream_options.iter().flatten() {
                            if let Some(links) = &quality_detail.main {
                                if let Some(flv_url) = &links.flv {
                                    if !flv_url.is_empty() {
                                        return Some(flv_url.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// 获取超清画质流
fn get_full_hd_stream(stream_info: &StreamUrlContainer) -> Option<String> {
    if let Some(hls_map) = &stream_info.hls_pull_url_map {
        if let Some(full_hd_url) = hls_map.get("FULL_HD1") {
            if !full_hd_url.is_empty() {
                return Some(full_hd_url.clone());
            }
        }
    }
    None
}

// 获取高清画质流
fn get_hd_stream(stream_info: &StreamUrlContainer) -> Option<String> {
    if let Some(hls_map) = &stream_info.hls_pull_url_map {
        if let Some(hd_url) = hls_map.get("HD1") {
            if !hd_url.is_empty() {
                return Some(hd_url.clone());
            }
        }
    }
    None
}
```

### 4.4 前端画质切换集成

```javascript
// 抖音画质切换组件
const DouyinQualitySelector = {
    // 可用画质列表
    qualities: [
        { key: 'origin', label: '原画', description: '最高画质' },
        { key: 'full_hd', label: '超清', description: '1080p' },
        { key: 'hd', label: '高清', description: '720p' },
        { key: 'auto', label: '自动', description: '自适应' }
    ],
    
    // 切换画质
    async switchQuality(roomId, quality) {
        try {
            const streamUrl = await invoke('get_douyin_stream_with_quality', {
                roomId: roomId,
                preferredQuality: quality
            });
            
            // 更新播放器源
            this.updatePlayerSource(streamUrl);
            
            // 保存用户偏好
            localStorage.setItem('douyin_preferred_quality', quality);
            
        } catch (error) {
            console.error('抖音画质切换失败:', error);
            // 降级处理
            this.fallbackToDefaultQuality(roomId);
        }
    },
    
    // 降级处理
    async fallbackToDefaultQuality(roomId) {
        try {
            const streamUrl = await invoke('get_douyin_live_stream_url', {
                roomId: roomId
            });
            this.updatePlayerSource(streamUrl);
        } catch (error) {
            console.error('获取默认画质也失败:', error);
        }
    }
};
```

## 5. 流处理和代理机制

### 5.1 代理服务器

抖音的实现中使用了本地代理服务器来处理直播流：

```rust
// 启动代理服务器
match start_proxy(app_handle, proxy_server_handle, stream_url_store).await {
    Ok(proxy_url) => {
        println!("代理启动成功: {}", proxy_url);
        proxied_stream_url = Some(proxy_url);
    }
    Err(e) => {
        eprintln!("代理启动失败: {}", e);
    }
}
```

### 5.2 重定向处理

对于不包含"pull-flv"的URL，需要进行重定向处理：

```rust
// 重定向处理逻辑
if !initial_flv_url.contains("pull-flv") {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
        
    let response = client.get(&initial_flv_url).send().await?;
    
    if response.status().is_redirection() {
        if let Some(location) = response.headers().get(reqwest::header::LOCATION) {
            let redirected_url = location.to_str()?;
            final_stream_url = Some(redirected_url.to_string());
        }
    }
}
```

## 6. 房间状态检查

### 6.1 直播状态码
- **status = 2**：正在直播
- **status = 4**：回放或未直播

### 6.2 状态检查实现

```rust
pub fn check_live_status(status: i32) -> String {
    match status {
        2 => "直播中".to_string(),
        4 => "未直播".to_string(),
        _ => "未知状态".to_string(),
    }
}
```

## 7. 弹幕系统

### 7.1 弹幕协议
抖音使用WebSocket协议进行弹幕通信，支持：
- 实时弹幕消息
- 礼物消息
- 进入房间消息
- 关注消息

### 7.2 弹幕监听实现
```rust
// 启动弹幕监听
pub async fn start_douyin_danmu_listener(
    room_id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // WebSocket连接和消息处理逻辑
    // ...
}
```

## 8. 技术实现要点

### 8.1 多层数据解析
1. **外层API响应**：标准JSON格式
2. **嵌套JSON字符串**：live_core_sdk_data中的stream_data
3. **画质选择逻辑**：多种画质源的优先级处理

### 8.2 错误处理机制
1. **网络请求失败**：重试机制和降级处理
2. **JSON解析失败**：容错处理和默认值
3. **代理启动失败**：直接返回原始URL
4. **画质不可用**：自动降级到可用画质

### 8.3 性能优化
1. **异步处理**：所有网络请求使用async/await
2. **连接复用**：HTTP客户端复用
3. **缓存机制**：流URL缓存避免重复请求

### 8.4 安全考虑
1. **请求头伪装**：模拟真实浏览器访问
2. **Cookie管理**：维护会话状态
3. **代理隔离**：本地代理避免跨域问题

## 9. 与斗鱼的对比

| 特性 | 抖音 | 斗鱼 |
|------|------|------|
| API复杂度 | 相对简单 | 复杂（需要JS签名） |
| 画质选择 | 多源优先级 | rate参数控制 |
| 流格式 | FLV/HLS | 主要FLV |
| 代理需求 | 需要本地代理 | 直接访问 |
| 重定向处理 | 需要处理 | 不需要 |
| 数据结构 | 嵌套JSON字符串 | 标准JSON |

## 10. 总结

抖音直播API的特点：
1. **相对简单**：不需要复杂的JavaScript签名算法
2. **多画质支持**：提供多种画质选择和优先级机制
3. **代理架构**：使用本地代理服务器处理流媒体
4. **容错性强**：多层降级处理确保服务可用性
5. **实时性好**：WebSocket弹幕系统响应迅速

整体而言，抖音的API设计更注重稳定性和用户体验，通过多种技术手段确保直播流的可靠获取和播放。