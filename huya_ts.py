import re
import time
import json
import base64
import hashlib
import urllib.parse
import requests
import random

from typing import Dict, Any


class Huya:
    baseURL = "https://m.huya.com/"

    def __init__(self, room_id: int = 0, url: str = ""):
        if not room_id and not url:
            raise ValueError("房间号和房间页面链接必需传入一个")
        self.room_id = room_id
        self.page_url = url

    @property
    def roomURL(self) -> str:
        return self.baseURL + str(self.room_id)

    def _get(self, url: str, headers: Dict[str, str] = None):
        headers = headers or {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        }
        resp = requests.get(url, headers=headers, timeout=10)
        resp.raise_for_status()
        return resp

    def getFinalRoomID(self) -> str:
        url = self.roomURL if self.room_id else self.page_url
        html = self._get(url).text
        m = re.search(r"stream: (\{.*?\"iFrameRate\":\d+\})", html, re.S)
        if not m:
            raise ValueError("未能在页面中找到 stream 数据")
        stream = json.loads(m.group(1))
        room_id = stream["data"][0]["gameLiveInfo"]["profileRoom"]
        return str(room_id)

    def getRoomProfile(self, room_id: str) -> Dict[str, Any]:
        url = f"https://mp.huya.com/cache.php?m=Live&do=profileRoom&roomid={room_id}"
        profile = self._get(url).json()
        if profile.get("status") != 200:
            raise RuntimeError(profile.get("message", "获取房间信息失败"))

        data = profile["data"]
        live_status = data.get("liveStatus")
        live_data = data.get("liveData", {})
        nick = live_data.get("nick", "")
        introduction = live_data.get("introduction", "")
        avatar180 = live_data.get("avatar180", "")
        profile_room = live_data.get("profileRoom", "")

        if live_status in ("REPLAY", "OFF"):
            raise RuntimeError("此房间未开播或正在重播")

        base_steam_info_list = data["stream"]["baseSteamInfoList"]

        # 随机 UID 生成反盗链参数
        uid = str(random.randint(10000, 99999999))

        # 仅取腾讯 TX 的 FLV 基础链接
        tx_flv_base = None
        for it in base_steam_info_list:
            if it.get("sCdnType") != "TX":
                continue
            streamname = it.get("sStreamName")
            ac = it.get("sFlvAntiCode", "")
            if not ac:
                continue
            anticode = self.parseAnticode(ac, uid, streamname)
            tx_flv_base = f"{it['sFlvUrl']}/{streamname}.{it['sFlvUrlSuffix']}?{anticode}"
            break

        # 解析清晰度列表，将 ratio 追加到链接末尾
        brs = []
        br_str = live_data.get("bitRateInfo")
        if br_str:
            try:
                brs = json.loads(br_str)
            except Exception:
                brs = []
        if not brs:
            brs = data.get("stream", {}).get("flv", {}).get("rateArray", []) or []

        flv_tx_urls = []
        if tx_flv_base:
            for b in brs:
                name = b.get("sDisplayName") or b.get("name") or "原画"
                r = int(b.get("iBitRate", b.get("bitRate", 0)) or 0)
                url_with_ratio = tx_flv_base + (f"&ratio={r}" if r > 0 else "")
                flv_tx_urls.append({"quality": name, "bitRate": r, "url": url_with_ratio})

        return {
            "nick": nick,
            "avatar180": avatar180,
            "introduction": introduction,
            "live_status": live_status,
            "profileRoom": profile_room,
            "flv_tx_urls": flv_tx_urls,
        }

    @staticmethod
    def newUuid() -> int:
        now = int(time.time() * 1000)
        rand = int(time.time() * 1000) % 1000
        return ((now % 10000000000) * 1000 + rand) % 4294967295

    def parseAnticode(self, code: str, uid: str, streamname: str) -> str:
        q: Dict[str, str] = {k: v for k, v in urllib.parse.parse_qsl(code, keep_blank_values=True)}
        q["ver"] = "1"
        q["sv"] = "2110211124"
        seqid = str(int(uid) + int(time.time() * 1000))
        q["seqid"] = seqid
        q["uid"] = uid
        q["uuid"] = str(self.newUuid())
        ctype = q.get("ctype", "")
        t_val = q.get("t", "")
        ss = hashlib.md5(f"{seqid}|{ctype}|{t_val}".encode("utf-8")).hexdigest()
        fm = base64.b64decode(q.get("fm", "").encode("utf-8")).decode("utf-8")
        fm = (
            fm.replace("$0", uid)
            .replace("$1", streamname)
            .replace("$2", ss)
            .replace("$3", q.get("wsTime", ""))
        )
        q["wsSecret"] = hashlib.md5(fm.encode("utf-8")).hexdigest()
        q.pop("fm", None)
        q.pop("txyp", None)
        return "&".join([f"{urllib.parse.quote(k)}={urllib.parse.quote(v)}" for k, v in q.items()])

    def printLiveLink(self) -> None:
        room_id = self.getFinalRoomID()
        info = self.getRoomProfile(room_id)
        print(f"主播: {info.get('nick','')}  房间ID: {info.get('profileRoom','')}")
        print(f"头像: {info.get('avatar180','')}")
        print(f"标题: {info.get('introduction','')}")
        print(f"状态: {info.get('live_status','')}")
        print("腾讯 FLV 不同清晰度链接：")
        urls = info.get("flv_tx_urls", [])
        if not urls:
            print("未找到腾讯 FLV 链路")
        else:
            for item in urls:
                print(f"{item['quality']}: {item['url']}")


if __name__ == "__main__":
    inp = input("输入虎牙直播房间号（或留空并输入页面URL）：\n").strip()
    if inp:
        h = Huya(room_id=int(inp))
    else:
        page = input("输入房间页面链接：\n").strip()
        h = Huya(url=page)
    h.printLiveLink()