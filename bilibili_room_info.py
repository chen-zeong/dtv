import requests
import hashlib
import time
from typing import Dict, Any
from urllib.parse import urlparse, parse_qs, quote


class BilibiliRoomInfo:
    """
    通过直播间号获取房间与主播信息（独立文件）
    - get_room_info(room_id): 调用 B 站接口获取完整房间数据（含主播基本信息）
    - get_anchor_info(room_id): 提取主播关键信息（uid、uname、face等）
    说明：实现了与 Dart 版一致的 WBI 签名流程（img_key、sub_key、mixinKey、wts、w_rid）。
    """

    k_default_user_agent = (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
    )
    k_default_referer = "https://live.bilibili.com/"

    # WBI 签名相关常量
    mixin_key_enc_tab = [
        46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35,
        27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13,
        37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4,
        22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
    ]

    def __init__(self, cookie: str = ""):
        self.cookie = cookie or ""
        self.k_img_key = ""
        self.k_sub_key = ""

    def get_header(self) -> Dict[str, str]:
        headers = {
            "user-agent": self.k_default_user_agent,
            "referer": self.k_default_referer,
        }
        if self.cookie:
            headers["cookie"] = self.cookie
        return headers

    def get_wbi_keys(self) -> tuple[str, str]:
        """获取最新的 img_key 与 sub_key，并做缓存。"""
        if self.k_img_key and self.k_sub_key:
            return self.k_img_key, self.k_sub_key
        resp = requests.get(
            "https://api.bilibili.com/x/web-interface/nav",
            headers=self.get_header(),
            timeout=10,
        )
        resp.raise_for_status()
        j = resp.json()
        wbi_img = j.get("data", {}).get("wbi_img", {})
        img_url = str(wbi_img.get("img_url", ""))
        sub_url = str(wbi_img.get("sub_url", ""))
        img_key = img_url[img_url.rfind("/") + 1:].split(".")[0] if img_url else ""
        sub_key = sub_url[sub_url.rfind("/") + 1:].split(".")[0] if sub_url else ""
        self.k_img_key = img_key
        self.k_sub_key = sub_key
        return img_key, sub_key

    def get_mixin_key(self, origin: str) -> str:
        """根据 mixin_key_enc_tab 生成混淆后的 key，取前 32 位。"""
        return "".join(origin[i] for i in self.mixin_key_enc_tab)[:32]

    def get_wbi_sign(self, url: str) -> dict:
        """为请求参数生成 WBI 签名，返回包含 wts 与 w_rid 的完整参数字典。"""
        img_key, sub_key = self.get_wbi_keys()
        mixin_key = self.get_mixin_key(img_key + sub_key)
        now = int(time.time())
        parsed = urlparse(url)
        qp_raw = parse_qs(parsed.query)  # values are lists
        qp = {k: (v[0] if isinstance(v, list) and v else "") for k, v in qp_raw.items()}
        qp["wts"] = str(now)
        # 过滤 value 中的 "!'()*" 字符并按 key 排序
        banned = set("!'()*")
        sorted_keys = sorted(qp.keys())
        filtered = {}
        for k in sorted_keys:
            v = qp[k]
            filtered[k] = "".join(ch for ch in v if ch not in banned)
        query = "&".join(f"{k}={quote(filtered[k], safe='')}" for k in sorted_keys)
        w_rid = hashlib.md5((query + mixin_key).encode("utf-8")).hexdigest()
        qp["w_rid"] = w_rid
        return qp

    def get_room_info(self, room_id: str) -> Dict[str, Any]:
        """通过直播间号获取房间与主播信息，等价于 Dart 的 getRoomInfo。"""
        base = "https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByRoom"
        url = f"{base}?room_id={room_id}"
        params = self.get_wbi_sign(url)
        resp = requests.get(base, params=params, headers=self.get_header(), timeout=10)
        resp.raise_for_status()
        return resp.json().get("data", {})

    def get_anchor_info(self, room_id: str) -> Dict[str, Any]:
        """便捷方法：提取主播关键信息。"""
        data = self.get_room_info(room_id)
        base_info = (data.get("anchor_info", {}) or {}).get("base_info", {})
        room_info = data.get("room_info", {}) or {}
        return {
            "uid": base_info.get("uid"),
            "uname": base_info.get("uname"),
            "face": base_info.get("face"),
            "room_id": room_info.get("room_id"),
            "title": room_info.get("title"),
            "live_status": room_info.get("live_status"),
            "area_name": room_info.get("area_name"),
        }


if __name__ == "__main__":
    room = input("输入哔哩哔哩直播房间号：\n").strip()
    cookie = input("如需带 cookie（可留空）：\n").strip()
    api = BilibiliRoomInfo(cookie=cookie)
    info = api.get_anchor_info(room)
    if not info:
        print("未获取到主播信息")
    else:
        print("主播信息：")
        for k, v in info.items():
            print(f"{k}: {v}")