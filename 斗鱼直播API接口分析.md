# 斗鱼直播API接口分析

## 概述

本文档分析了斗鱼直播平台的各种API接口，包括直播链接获取、房间信息获取、分类获取等核心功能的实现。

## 1. 直播链接获取接口

### 1.1 房间状态检查接口

**接口地址：** `http://open.douyucdn.cn/api/RoomApi/room/{room_id}`

**请求方式：** GET

**请求头：**
```
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36
```

**参数说明：**
- `room_id`: 房间ID（路径参数）

**返回数据结构：**
```json
{
  "error": 0,
  "data": {
    "room_status": "1"  // "1"表示直播中，"2"表示未开播
  }
}
```

### 1.2 获取房间网页内容

**接口地址：** `https://www.douyu.com/{room_id}`

**请求方式：** GET

**请求头：**
```
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8
Accept-Language: zh-CN,zh;q=0.9
Connection: keep-alive
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36
```

**功能：** 获取房间页面HTML内容，从中提取JavaScript函数用于生成签名

### 1.3 获取直播流地址

**接口地址：** `https://www.douyu.com/lapi/live/getH5Play/{room_id}`

**请求方式：** POST

**请求头：**
```
Content-Type: application/x-www-form-urlencoded
Origin: https://www.douyu.com
Referer: https://www.douyu.com/{room_id}
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36
```

**请求参数：**
- 通过JavaScript函数生成的签名参数
- `cdn`: 固定值 "ws-h5"
- `rate`: 固定值 0

**签名生成过程：**
1. 从房间页面提取JavaScript函数 `vdwdae325w_64we` 和 `ub98484234`
2. 执行 `ub98484234()` 获取包含v参数的字符串
3. 提取v参数值
4. 生成时间戳 `t10`
5. 计算MD5: `md5(room_id + did + t10 + v)`
6. 构造签名函数并执行获取最终参数

**固定参数：**
- `did`: "10000000000000000000000000001501"

**返回数据结构：**
```json
{
  "data": {
    "rtmp_url": "rtmp服务器地址",
    "rtmp_live": "流路径"
  }
}
```

**最终流地址：** `{rtmp_url}/{rtmp_live}`

## 2. 房间信息获取接口

### 2.1 房间详细信息接口

**接口地址：** `https://www.douyu.com/betard/{room_id}`

**请求方式：** GET

**请求头：**
```
Accept: application/json, text/plain, */*
Accept-Language: zh-CN,zh;q=0.9
Cache-Control: no-cache
Pragma: no-cache
Referer: https://www.douyu.com/{room_id}
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36
```

**参数说明：**
- `room_id`: 房间ID（路径参数）

**返回数据结构：**
```json
{
  "data": {
    "room": {
      "room_id": "房间ID",
      "room_name": "房间名称",
      "nickname": "主播昵称",
      "avatar_mid": "头像URL",
      "videoLoop": 视频循环标识,
      "show_status": 显示状态
    }
  }
}
```

## 3. 分类获取接口

### 3.1 主分类列表接口

**接口地址：** `https://m.douyu.com/api/cate/list`

**请求方式：** GET

**请求头：**
```
User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1
```

**返回数据结构：**
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "cate1Info": [
      {
        "cate1Id": 一级分类ID,
        "cate1Name": "一级分类名称"
      }
    ],
    "cate2Info": [
      {
        "cate1Id": 父级分类ID,
        "cate2Id": 二级分类ID,
        "cate2Name": "二级分类名称",
        "shortName": "简称",
        "icon": "图标URL"
      }
    ]
  }
}
```

## 4. 直播列表获取接口

### 4.1 分类直播列表接口

**接口地址：** `https://m.douyu.com/hgapi/live/cate/newRecList`

**请求方式：** GET

**请求头：**
```
User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1
```

**请求参数：**
- `offset`: 偏移量（分页用）
- `cate2`: 二级分类ID
- `limit`: 每页数量限制

**完整URL示例：**
```
https://m.douyu.com/hgapi/live/cate/newRecList?offset=0&cate2=1&limit=20
```

**返回数据结构：**
```json
{
  "error": 0,
  "msg": "success",
  "data": {
    "list": [
      {
        "rid": 房间ID,
        "roomName": "房间名称",
        "nickname": "主播昵称",
        "roomSrc": "房间封面图",
        "avatar": "头像URL",
        "hn": "观看人数"
      }
    ],
    "total": 总数量
  }
}
```

## 5. 主播搜索接口

### 5.1 主播搜索接口

**功能：** 根据关键词搜索主播

**实现：** 通过 `perform_anchor_search` 函数实现（具体接口地址需要进一步分析）

## 6. 弹幕相关接口

### 6.1 弹幕监听

**功能：** 实时获取房间弹幕消息

**实现方式：** WebSocket连接

**相关命令：**
- `start_danmaku_listener`: 开始监听弹幕
- `stop_danmaku_listener`: 停止监听弹幕

## 7. 画质切换技术实现

### 7.1 画质参数说明
- **rate参数**：在直播链接获取接口中，`rate`参数用于指定画质等级
- **当前实现**：代码中`rate`参数固定为0，表示获取默认画质
- **画质等级**：斗鱼支持多种画质等级，包括：
  - 超清：rate = 0
  - 高清：rate = 1 
  - 标清：rate = 2
  - 流畅：rate = 3

### 7.2 画质切换实现方案

```rust
// 在get_pc_js函数中支持动态画质参数
pub async fn get_pc_js(&self, room_id: &str, rate: i32) -> Result<String, Box<dyn std::error::Error>> {
    // ... 现有逻辑保持不变
    
    // 在构造签名参数时使用传入的rate值
    let params = format!(
        "room_id={}&did={}&time={}&auth={}&rate={}",
        room_id, did, time, auth, rate  // 使用动态rate参数
    );
    
    // ... 其余逻辑保持不变
}

// 提供画质切换的公共接口
pub async fn get_stream_url_with_quality(room_id: &str, quality: &str) -> Result<String, String> {
    let rate = match quality {
        "超清" | "ultra" => 0,
        "高清" | "high" => 1,
        "标清" | "standard" => 2,
        "流畅" | "smooth" => 3,
        _ => 0, // 默认超清
    };
    
    let douyu = DouYu::new();
    douyu.get_real_url(room_id, rate).await
}
```

### 7.3 前端画质切换集成

```javascript
// 在前端调用画质切换
const switchQuality = async (roomId, quality) => {
    try {
        const streamUrl = await invoke('get_stream_url_with_quality', {
            roomId: roomId,
            quality: quality
        });
        // 更新播放器源
        updatePlayerSource(streamUrl);
    } catch (error) {
        console.error('画质切换失败:', error);
    }
};
```

## 8. 技术实现要点

### 8.1 签名算法

斗鱼的直播流获取需要复杂的JavaScript签名算法：

1. **提取JavaScript函数**：从房间页面HTML中提取特定的JavaScript函数
2. **动态执行**：使用Deno JavaScript运行时执行提取的函数
3. **参数计算**：结合房间ID、设备ID、时间戳等生成签名参数
4. **MD5加密**：对特定字符串进行MD5加密

### 8.2 请求头伪装

所有接口都需要设置合适的User-Agent来模拟真实浏览器或移动设备访问。

### 8.3 错误处理

- 网络请求失败处理
- API返回错误码处理
- JSON解析失败处理
- 房间状态检查（是否在直播）

### 8.4 数据转换

后端Rust代码需要将斗鱼API返回的数据结构转换为前端TypeScript期望的数据格式。

### 8.5 画质动态切换

支持运行时动态切换不同画质等级，提升用户体验。

## 9. 安全考虑

1. **反爬虫机制**：斗鱼有复杂的JavaScript混淆和签名验证
2. **请求频率限制**：需要控制API调用频率避免被封禁
3. **User-Agent检测**：必须使用真实的浏览器User-Agent
4. **Referer检查**：某些接口需要正确的Referer头

## 10. 总结

斗鱼的API接口设计相对复杂，特别是直播流获取部分需要动态执行JavaScript代码来生成签名。整个系统采用了多层API调用的方式：

1. 首先检查房间状态
2. 获取房间页面提取签名函数
3. 执行JavaScript生成签名参数
4. 调用流媒体接口获取真实播放地址

这种设计有效防止了简单的API调用，增加了反爬虫的难度。