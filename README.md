<div align="center">
  <img src="images/icon.png" alt="DTV Logo" width="128" height="128">
  <h1>DTV</h1>
  <p>基于 Tauri 2.0 的跨平台斗鱼、虎牙、抖音、bilibili直播桌面客户端</p>
</div>





## 说明

1. 本项目基于 Tauri 2.0 开发，体积小，占用率低。最初基于 Electron 开发的版本已存放于 [electron-legacy](https://github.com/c-zeong/DTV/tree/electron-legacy) 分支，可以做对比
2. 平台接口有访问频率限制，过于频繁的请求会触发验证码校验，建议合理使用
3. 本项目仅供学习编程目的使用，未进行任何逆向工程

### 支持平台

| 平台 | 直播流 | 弹幕 | 搜索 |
| --- | --- | --- | --- |
| 斗鱼 | ✅ | ✅ | ✅ |
| 虎牙 | ✅ | ✅ | ✅ |
| bilibili | ✅ | ✅ | 仅房间号 |
| 抖音 | 最高仅蓝光 | ✅ | 仅房间号 |

## 功能

- 📺 平台支持：支持斗鱼、虎牙、bilibili、抖音直播
- 💬 弹幕显示：实时显示直播间弹幕，只显示聊天弹幕，不显示礼物等其他类型弹幕
- ⭐ 主播收藏：支持收藏喜欢的主播，支持收藏列表手动拖拽排序
- 📋 支持平台：Mac(Intel+Arm)，Windows(Win7需要自行安装Webview2)，Linux(包括Ubuntu和各类发行版)
- 🌓 主题切换：支持明暗主题切换

## 软件截图

<div align="center">
  <p>日间模式</p>
  <img src="images/iShot_light.webp" alt="日间模式" style="width: 100%; max-width: 800px; display: block; margin-left: auto; margin-right: auto;">
</div>


<br>

<div align="center">
  <p>夜间模式</p>
  <img src="images/iShot_dark.webp" alt="夜间模式" style="width: 100%; max-width: 800px; display: block; margin-left: auto; margin-right: auto;">
</div>


<br>

<div align="center">
  <p>收藏列表</p>
  <img src="images/iShot_list.webp" alt="收藏列表" style="width: 100%; max-width: 800px; display: block; margin-left: auto; margin-right: auto;">
</div>

## 安装方式

可以在 [release](https://github.com/c-zeong/dtv/releases) 目录下载对应系统的安装包, 也可以通过源码编译安装

## 编译

```bash
安装protobuf

# 克隆项目
git clone https://github.com/c-zeong/dtv.git
cd dtv

# 安装依赖
pnpm install

# 开发调试
pnpm tauri dev

# 打包构建
pnpm tauri build      # 构建当前系统的安装包

# 打包 ARM64 版本（Intel Mac 上交叉编译）
pnpm tauri build --target aarch64-apple-darwin
```

##  参考

- 斗鱼直播流获取参考了 [@wbt5/real-url](https://github.com/wbt5/real-url)  
- 抖音弹幕参考了[@saermart/DouyinLiveWebFetcher](https://github.com/saermart/DouyinLiveWebFetcher)
- 虎牙参考了https://github.com/liuchuancong/pure_live
- b站弹幕参考了https://github.com/xfgryujk/blivedm

