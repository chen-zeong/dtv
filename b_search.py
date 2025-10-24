"""
命令行版哔哩哔哩直播搜索，独立文件可直接运行。

示例：
    python3 bilibili_search.py 天使彦 --type room
"""

from __future__ import annotations

import argparse
import re
from typing import Any, Dict, List, Optional

import requests

DEFAULT_UA = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
)
LIVE_REFERER = "https://live.bilibili.com/"


def _strip_em_tags(text: str) -> str:
    """移除搜索结果里高亮用的 <em> 标签。"""
    return re.sub(r"<.*?em.*?>", "", text)


class BilibiliClient:
    """提供哔哩哔哩直播搜索能力的轻量客户端。"""

    def __init__(self, user_agent: str = DEFAULT_UA, cookie: Optional[str] = None):
        self._session = requests.Session()
        self._session.headers.update(
            {
                "User-Agent": user_agent,
                "Referer": LIVE_REFERER,
            }
        )
        if cookie:
            self._session.headers["Cookie"] = cookie
        self._buvid3 = ""
        self._buvid4 = ""

    def search_rooms(self, keyword: str, page: int = 1) -> List[Dict[str, Any]]:
        """搜索直播间，返回精简列表。"""
        headers = self._ensure_headers()
        params = {
            "context": "",
            "search_type": "live",
            "cover_type": "user_cover",
            "order": "",
            "keyword": keyword,
            "category_id": "",
            "__refresh__": "",
            "_extra": "",
            "highlight": 0,
            "single_column": 0,
            "page": page,
        }
        resp = self._session.get(
            "https://api.bilibili.com/x/web-interface/search/type",
            params=params,
            headers=headers,
            timeout=10,
        )
        resp.raise_for_status()
        payload = resp.json()
        if payload.get("code") != 0:
            raise RuntimeError(f"Bilibili search failed: {payload.get('message', 'unknown error')}")

        data = payload.get("data", {})
        live_rooms = data.get("result", {}).get("live_room") or []
        rooms: List[Dict[str, Any]] = []
        for item in live_rooms:
            title = _strip_em_tags(item.get("title", ""))
            cover = item.get("cover", "")
            avatar = item.get("uface", "")
            rooms.append(
                {
                    "room_id": str(item.get("roomid", "")),
                    "title": title,
                    "cover": f"https:{cover}@400w.jpg" if cover else "",
                    "anchor": item.get("uname", ""),
                    "avatar": f"https:{avatar}@400w.jpg" if avatar else "",
                    "watching": str(item.get("online", "")),
                    "area": item.get("cate_name", ""),
                    "is_live": bool(item.get("live_status") == 1),
                    "raw": item,
                }
            )

        return rooms

    # 内部工具 --------------------------------------------------------------
    def _ensure_headers(self) -> Dict[str, str]:
        if not self._buvid3 or not self._buvid4:
            self._ensure_buvid()
        return self._session.headers

    def _ensure_buvid(self) -> None:
        cookie_header = self._session.headers.get("Cookie", "")
        if cookie_header:
            match3 = re.search(r"buvid3=([^;]+)", cookie_header)
            match4 = re.search(r"buvid4=([^;]+)", cookie_header)
            if match3:
                self._buvid3 = match3.group(1)
            if match4:
                self._buvid4 = match4.group(1)

        if self._buvid3 and self._buvid4:
            return

        resp = self._session.get(
            "https://api.bilibili.com/x/frontend/finger/spi",
            headers=self._session.headers,
            timeout=10,
        )
        resp.raise_for_status()
        data = resp.json().get("data", {})
        self._buvid3 = data.get("b_3", "") or self._buvid3
        self._buvid4 = data.get("b_4", "") or self._buvid4
        self._refresh_cookie()

    def _refresh_cookie(self) -> None:
        cookie_header = self._session.headers.get("Cookie", "")
        existing: Dict[str, str] = {}
        order: List[str] = []

        if cookie_header:
            for segment in cookie_header.split(";"):
                segment = segment.strip()
                if not segment:
                    continue
                key = segment.split("=", 1)[0].lower()
                if key not in existing:
                    order.append(key)
                existing[key] = segment

        if self._buvid3:
            if "buvid3" not in existing:
                order.append("buvid3")
            existing["buvid3"] = f"buvid3={self._buvid3}"
        if self._buvid4:
            if "buvid4" not in existing:
                order.append("buvid4")
            existing["buvid4"] = f"buvid4={self._buvid4}"

        if existing:
            cookie_value = "; ".join(existing[key] for key in order if existing[key])
            self._session.headers["Cookie"] = cookie_value


def main() -> None:
    parser = argparse.ArgumentParser(description="按关键词搜索哔哩哔哩直播间，输出第一个搜索结果。")
    parser.add_argument("keyword", help="搜索关键词。")
    parser.add_argument("--page", type=int, default=1, help="页码，从 1 开始。")
    args = parser.parse_args()

    client = BilibiliClient()
    try:
        rooms = client.search_rooms(args.keyword, page=args.page)
    except Exception as exc:  # pragma: no cover - CLI 外层兜底
        parser.exit(1, f"搜索失败: {exc}\n")

    if not rooms:
        parser.exit(2, "未找到匹配的直播间。\n")

    for room in rooms:
        print(f"房间ID: {room['room_id']}")
        print(f"标题: {room['title']}")
        print(f"主播: {room['anchor']}")
        print(f"人气: {room['watching']}")
        print(f"分区: {room['area']}")
        print(f"封面: {room['cover']}")
        print(f"头像: {room['avatar']}")
        print(f"直播中: {'是' if room['is_live'] else '否'}")


if __name__ == "__main__":
    main()
