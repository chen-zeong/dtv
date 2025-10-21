"""
Minimal Douyin live stream fetcher demo.

Requires:
    pip install httpx
"""

from __future__ import annotations

import asyncio
import json
import re
import urllib.parse
from typing import Optional

import httpx

DEFAULT_COOKIE = (
    "ttwid=1%7CB1qls3GdnZhUov9o2NxOMxxYS2ff6OSvEWbv0ytbES4%7C1680522049"
    "%7C280d802d6d478e3e78d0c807f7c487e7ffec0ae4e5fdd6a0fe74c3c6af149511"
)

HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:115.0) "
        "Gecko/20100101 Firefox/115.0"
    ),
    "Accept-Language": "zh-CN,zh;q=0.8",
    "Referer": "https://live.douyin.com/",
    "Cookie": DEFAULT_COOKIE,
}

QUALITY_ORDER = ("OD", "BD", "UHD", "HD", "SD", "LD")


class UnsupportedUrlError(Exception):
    pass


async def _fetch(
    url: str,
    headers: Optional[dict] = None,
    proxy: Optional[str] = None,
    *,
    follow_redirects: bool = False,
) -> httpx.Response:
    async with httpx.AsyncClient(
        headers=headers or HEADERS,
        proxy=proxy,
        timeout=15,
        follow_redirects=follow_redirects,
    ) as client:
        resp = await client.get(url)
        resp.raise_for_status()
        return resp


async def _parse_web_room(
    url: str,
    proxy: Optional[str],
    cookies: Optional[str],
) -> dict:
    headers = HEADERS.copy()
    if cookies:
        headers["Cookie"] = cookies
    html = (await _fetch(url, headers=headers, proxy=proxy)).text
    match = re.search(r'(\{\\"state\\":.*?)]\\n"]\)', html)
    if not match:
        match = re.search(r'(\{\\"common\\":.*?)]\\n"]\)</script><div hidden', html)
    if not match:
        raise ValueError("Cannot locate roomStore JSON")
    cleaned = match.group(1).replace("\\", "").replace("u0026", "&")
    room_store = re.search(
        r'"roomStore":(.*?),"linkmicStore"', cleaned, re.DOTALL
    ).group(1)
    anchor_name = re.search(
        r'"nickname":"(.*?)","avatar_thumb', room_store, re.DOTALL
    ).group(1)
    room_store = room_store.split(',"has_commerce_goods"')[0] + "}}}"
    room_data = json.loads(room_store)["roomInfo"]["room"]
    room_data["anchor_name"] = anchor_name

    stream_orientation = room_data["stream_url"]["stream_orientation"]
    match_list = re.findall(
        r'"(\{\\"common\\":.*?)"]\)</script><script nonce=', html
    )
    origin = None
    if match_list:
        candidate = match_list[0 if stream_orientation == 1 else 1]
        raw = json.loads(
            candidate.replace("\\", "")
            .replace('"{', "{")
            .replace('}"', "}")
            .replace("u0026", "&")
        )
        origin = raw.get("data", {}).get("origin", {}).get("main")
    if not origin:
        fallback = html.replace("\\", "").replace("u0026", "&")
        match = re.search(r'"origin":\{"main":(.*?),"dash"', fallback, re.DOTALL)
        if match:
            origin = json.loads(match.group(1) + "}")

    if origin:
        codec = origin["sdk_params"].get("VCodec") or ""
        origin_m3u8 = {"ORIGIN": f'{origin["hls"]}&codec={codec}'}
        origin_flv = {"ORIGIN": f'{origin["flv"]}&codec={codec}'}
        room_data["stream_url"]["hls_pull_url_map"] = (
            origin_m3u8 | room_data["stream_url"]["hls_pull_url_map"]
        )
        room_data["stream_url"]["flv_pull_url"] = (
            origin_flv | room_data["stream_url"]["flv_pull_url"]
        )
    return room_data


async def _resolve_ids(
    url: str,
    proxy: Optional[str],
    cookies: Optional[str],
) -> tuple[str, str]:
    headers = HEADERS.copy()
    if cookies:
        headers["Cookie"] = cookies
    resp = await _fetch(
        url, headers=headers, proxy=proxy, follow_redirects=True
    )
    redirected = str(resp.url)
    if "reflow/" not in redirected:
        raise UnsupportedUrlError("URL does not resolve to /reflow/")
    sec_user_id = redirected.split("?")[0].rsplit("/", 1)[-1]
    room_id = redirected.split("?")[0].rsplit("/", 1)[-1]
    return room_id, sec_user_id


async def _fetch_app_room(
    url: str,
    proxy: Optional[str],
    cookies: Optional[str],
) -> dict:
    headers = HEADERS.copy()
    if cookies:
        headers["Cookie"] = cookies
    if "live.douyin.com/" in url:
        web_rid = url.split("?")[0].rsplit("live.douyin.com/", 1)[-1]
        params = {
            "aid": "6383",
            "app_name": "douyin_web",
            "live_id": "1",
            "device_platform": "web",
            "language": "zh-CN",
            "browser_language": "zh-CN",
            "browser_platform": "Win32",
            "browser_name": "Chrome",
            "browser_version": "120.0.0.0",
            "web_rid": web_rid,
            "msToken": "",
            "a_bogus": "",
        }
        api = (
            "https://live.douyin.com/webcast/room/web/enter/?"
            + urllib.parse.urlencode(params)
        )
        json_data = (await _fetch(api, headers=headers, proxy=proxy)).json()["data"]
        room = json_data["data"][0]
        room["anchor_name"] = json_data["user"]["nickname"]
        return room

    room_id, sec_uid = await _resolve_ids(url, proxy, cookies)
    params = {
        "verifyFp": "verify_mocksignature",
        "type_id": "0",
        "live_id": "1",
        "room_id": room_id,
        "sec_user_id": sec_uid,
        "version_code": "99.99.99",
        "app_id": "1128",
    }
    api = (
        "https://webcast.amemv.com/webcast/room/reflow/info/?"
        + urllib.parse.urlencode(params)
    )
    data = (await _fetch(api, headers=headers, proxy=proxy)).json()["data"]["room"]
    data["anchor_name"] = data["owner"]["nickname"]
    return data


async def get_douyin_stream(
    room_url: str,
    *,
    quality: str = "OD",
    proxy: Optional[str] = None,
    cookies: Optional[str] = None,
) -> dict:
    try:
        room = await _parse_web_room(room_url, proxy, cookies)
    except Exception:
        room = await _fetch_app_room(room_url, proxy, cookies)

    status = room.get("status")
    if status != 2:
        return {
            "is_live": False,
            "anchor_name": room.get("anchor_name"),
            "status": status,
        }

    flv_list = list(room["stream_url"]["flv_pull_url"].values())
    hls_list = list(room["stream_url"]["hls_pull_url_map"].values())
    while len(flv_list) < len(QUALITY_ORDER):
        flv_list.append(flv_list[-1])
        hls_list.append(hls_list[-1])

    quality = quality.upper()
    if quality.isdigit():
        quality = QUALITY_ORDER[int(quality[0])]
    idx = QUALITY_ORDER.index(quality) if quality in QUALITY_ORDER else 0
    chosen = {"flv": flv_list[idx], "m3u8": hls_list[idx]}

    if not await _check_url(chosen["m3u8"], proxy):
        fallback = idx + 1 if idx + 1 < len(QUALITY_ORDER) else idx - 1
        chosen = {"flv": flv_list[fallback], "m3u8": hls_list[fallback]}

    return {
        "is_live": True,
        "anchor_name": room["anchor_name"],
        "title": room.get("title"),
        "quality": quality,
        "m3u8_url": chosen["m3u8"],
        "flv_url": chosen["flv"],
    }


async def _check_url(url: str, proxy: Optional[str]) -> bool:
    if not url:
        return False
    try:
        async with httpx.AsyncClient(proxy=proxy, timeout=10) as client:
            resp = await client.head(url)
            if resp.status_code in (403, 405):
                resp = await client.get(url)
            resp.raise_for_status()
            return True
    except httpx.HTTPError:
        return False


if __name__ == "__main__":
    test_url = "https://live.douyin.com/448882955167"
    result = asyncio.run(get_douyin_stream(test_url, quality="OD"))
    print(json.dumps(result, ensure_ascii=False, indent=2))

