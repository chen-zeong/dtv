import requests
from typing import List, Dict, Any


class BilibiliStream:
    """
    仅包含从 bilibili_site.dart 中提取并改写的获取直播清晰度与播放链接的方法：
    - get_play_qualities(room_id): 获取可用清晰度列表（qn 与描述）
    - get_play_urls(room_id, qn): 按清晰度获取播放链接（非 flv 格式，保持与原 Dart 逻辑一致）
    其他未使用的功能（例如分类、搜索、WBI 签名、弹幕等）均未包含。
    """

    k_default_user_agent = (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
    )
    k_default_referer = "https://live.bilibili.com/"

    def __init__(self, cookie: str = ""):
        self.cookie = cookie or ""

    def get_header(self) -> Dict[str, str]:
        # 精简版本，仅提供 UA 与 Referer；如需传入 cookie，直接使用用户提供的 cookie。
        headers = {
            "user-agent": self.k_default_user_agent,
            "referer": self.k_default_referer,
        }
        if self.cookie:
            headers["cookie"] = self.cookie
        return headers

    def get_play_qualities(self, room_id: str) -> List[Dict[str, Any]]:
        """获取直播可用清晰度列表，返回 [{"quality": 描述, "qn": 数值}]"""
        url = "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo"
        params = {
            "room_id": room_id,
            "protocol": "0,1",
            "format": "0,1,2",
            "codec": "0",
            "platform": "html5",
            "dolby": "5",
        }
        resp = requests.get(url, params=params, headers=self.get_header(), timeout=10)
        resp.raise_for_status()
        data = resp.json()

        playurl = data.get("data", {}).get("playurl_info", {}).get("playurl", {})
        qn_desc_list = playurl.get("g_qn_desc", [])
        qn_desc_map = {}
        for item in qn_desc_list:
            try:
                qn_desc_map[int(str(item.get("qn", "0")))] = str(item.get("desc", ""))
            except Exception:
                pass

        qualities: List[Dict[str, Any]] = []
        streams = playurl.get("stream", [])
        if streams:
            # 仅取第一个流的第一个格式的第一个编码的清晰度列表（与 Dart 逻辑一致）
            try:
                accept_qn = streams[0]["format"][0]["codec"][0]["accept_qn"]
                for qn in accept_qn:
                    qualities.append({
                        "quality": qn_desc_map.get(qn, "未知清晰度"),
                        "qn": qn,
                    })
            except Exception:
                pass
        return qualities

    def get_play_urls(self, room_id: str, qn: int) -> List[str]:
        """按清晰度 qn 获取播放链接（保持 Dart 逻辑：仅非 flv 的格式）。"""
        url = "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo"
        params = {
            "room_id": room_id,
            "protocol": "0,1",
            "format": "0,1,2",
            "codec": "0",
            "platform": "html5",
            "dolby": "5",
            "qn": qn,
        }
        resp = requests.get(url, params=params, headers=self.get_header(), timeout=10)
        resp.raise_for_status()
        data = resp.json()
        print(data)

        urls: List[str] = []
        playurl = data.get("data", {}).get("playurl_info", {}).get("playurl", {})
        streams = playurl.get("stream", [])
        for stream_item in streams:
            for format_item in stream_item.get("format", []):
                format_name = format_item.get("format_name", "")
                codec_list = format_item.get("codec", [])
                # 仅处理非 flv（与原 Dart 实现一致），通常生成 m3u8/hls 链接
                if format_name != "flv":
                    for codec_item in codec_list:
                        base_url = str(codec_item.get("base_url", ""))
                        for url_info in codec_item.get("url_info", []):
                            host = str(url_info.get("host", ""))
                            extra = str(url_info.get("extra", ""))
                            video_url = f"{host}{base_url}{extra}"

                            if ".mcdn.bilivideo" in video_url:
                                # 走代理以提升可用性（与 Dart 逻辑一致）
                                from urllib.parse import quote
                                video_url = f"https://proxy-tf-all-ws.bilivideo.com/?url={quote(video_url)}"
                            elif "/upgcxcode/" in video_url:
                                # 选择一个固定 CDN（ali），并替换 host（与 Dart 逻辑一致）
                                import re as _re
                                cdn = "upos-sz-mirrorali.bilivideo.com"
                                video_url = _re.sub(r"(http|https)://(.*?)/upgcxcode/", f"https://{cdn}/upgcxcode/", video_url)
                            urls.append(video_url)
        # 将包含 mcdn 的链接排序到最后
        urls.sort(key=lambda u: ("mcdn" in u, u))
        return urls


if __name__ == "__main__":
    room = input("输入哔哩哔哩直播房间号：\n").strip()
    cookie = input("如需带 cookie（可留空）：\n").strip()
    bili = BilibiliStream(cookie=cookie)
    qs = bili.get_play_qualities(room)
    if not qs:
        print("未获取到清晰度信息")
    else:
        print("可用清晰度：")
        for i, q in enumerate(qs, 1):
            print(f"{i}. {q['quality']} (qn={q['qn']})")
        try:
            idx = int(input("选择要获取链接的序号：\n").strip()) - 1
        except Exception:
            idx = 0
        idx = max(0, min(idx, len(qs) - 1))
        urls = bili.get_play_urls(room, qs[idx]["qn"])
        print(f"清晰度 {qs[idx]['quality']} 的播放链接（非 flv）：")
        for u in urls:
            print(u)