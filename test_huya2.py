#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import asyncio
import json
import base64
import websockets
import urllib.request
from jce import types, JceStruct, JceField


# ----------------------
# TARS/JCE Structs
# ----------------------
class HYPushMessage(JceStruct):
    pushType: types.INT32 = JceField(jce_id=0)
    uri: types.INT32 = JceField(jce_id=1)
    msg: types.BYTES = JceField(jce_id=2)
    protocolType: types.INT32 = JceField(jce_id=3)


class HYSender(JceStruct):
    uid: types.INT64 = JceField(jce_id=0)
    lMid: types.INT64 = JceField(jce_id=1)
    nickName: types.STRING = JceField(jce_id=2)
    gender: types.INT32 = JceField(jce_id=3)


class HYBulletFormat(JceStruct):
    fontColor: types.INT32 = JceField(jce_id=0)
    fontSize: types.INT32 = JceField(jce_id=1)
    textSpeed: types.INT32 = JceField(jce_id=2)
    transitionType: types.INT32 = JceField(jce_id=3)


class HYMessage(JceStruct):
    userInfo: HYSender = JceField(jce_id=0, jce_type=types.STRUCT_START)
    content: types.STRING = JceField(jce_id=3)
    bulletFormat: HYBulletFormat = JceField(jce_id=6, jce_type=types.STRUCT_START)


# ----------------------
# Join room payload (TARS/JCE)
# ----------------------
class JoinPayload(JceStruct):
    ayyuid: types.INT64 = JceField(jce_id=0)
    flagTrue: types.BOOL = JceField(jce_id=1)
    empty2: types.STRING = JceField(jce_id=2)
    empty3: types.STRING = JceField(jce_id=3)
    tid: types.INT32 = JceField(jce_id=4)
    sid: types.INT32 = JceField(jce_id=5)
    zero6: types.INT32 = JceField(jce_id=6)
    zero7: types.INT32 = JceField(jce_id=7)


class WebSocketCommand(JceStruct):
    cmd: types.INT32 = JceField(jce_id=0)
    content: types.BYTES = JceField(jce_id=1)


# 收到的外层包：先有一个type(int, jce_id=0)，再是内层bytes(jce_id=1)
class WSOuterPacket(JceStruct):
    msgType: types.INT32 = JceField(jce_id=0)
    payload: types.BYTES = JceField(jce_id=1)


def build_join_payload(ayyuid: int, tid: int, sid: int) -> bytes:
    inner = JoinPayload(
        ayyuid=ayyuid,
        flagTrue=True,
        empty2="",
        empty3="",
        tid=tid,
        sid=sid,
        zero6=0,
        zero7=0,
    ).encode()
    outer = WebSocketCommand(cmd=1, content=inner).encode()
    return outer


HEARTBEAT_DATA = base64.b64decode("ABQdAAwsNgBM")
WS_URL = "wss://cdnws.api.huya.com"


async def huya_danmaku(ayyuid: int, topSid: int):
    async with websockets.connect(WS_URL) as ws:
        # join room（按Flutter实现，tid=topSid，sid也用topSid）
        await ws.send(build_join_payload(ayyuid, topSid, topSid))
        print("[Huya] Joined room, start receiving...")

        async def heartbeat_loop():
            while True:
                await asyncio.sleep(60)
                await ws.send(HEARTBEAT_DATA)

        hb_task = asyncio.create_task(heartbeat_loop())
        try:
            async for msg in ws:
                if isinstance(msg, bytes):
                    await handle_huya_message(msg)
        finally:
            hb_task.cancel()


async def handle_huya_message(data: bytes):
    try:
        outer = WSOuterPacket.decode(data)
        if int(outer.msgType) != 7:
            return
        push = HYPushMessage.decode(bytes(outer.payload))
        if int(push.uri) == 1400:
            hymsg = HYMessage.decode(bytes(push.msg))
            uname = str(hymsg.userInfo.nickName)
            content = str(hymsg.content)
            print(f"{uname}: {content}")
    except Exception as e:
        print("[Huya][DecodeError]", e)


def fetch_huya_ids(room_id: int):
    """根据虎牙房间号自动获取 ayyuid、topSid"""
    url = f"https://mp.huya.com/cache.php?m=Live&do=profileRoom&roomid={room_id}&showSecret=1"
    headers = {
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36",
        "Accept": "*/*",
        "Origin": "https://www.huya.com",
        "Referer": "https://www.huya.com/",
    }
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=10) as resp:
        body = resp.read().decode("utf-8", errors="ignore")
    result = json.loads(body)

    if int(result.get("status", 0)) == 200 and result.get("data", {}).get("stream"):
        data = result["data"]
        ayyuid = int(data.get("profileInfo", {}).get("yyid", 0))
        topSid = 0

        baseSteamInfoList = data.get("stream", {}).get("baseSteamInfoList", []) or []

        # 直接从基础流信息取频道ID，避免解析无关的 flv/hls 细节
        if baseSteamInfoList:
            topSid = int(baseSteamInfoList[0].get("lChannelId", 0))
        else:
            topSid = 0

        if not topSid:
            raise ValueError("未找到频道ID，房间可能未开播")

        return ayyuid, topSid
    else:
        raise ValueError("房间未开播或无流信息，无法获取弹幕参数")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Huya Danmaku via WebSocket (TARS/JCE)")
    parser.add_argument("--roomId", type=int, required=True, help="虎牙房间号")
    args = parser.parse_args()

    try:
        ayyuid, topSid = fetch_huya_ids(args.roomId)
        print(f"[Huya] 参数: ayyuid={ayyuid}, topSid={topSid}")
    except Exception as e:
        print(f"[Huya] 获取参数失败: {e}")
        raise SystemExit(1)

    asyncio.run(huya_danmaku(ayyuid, topSid))