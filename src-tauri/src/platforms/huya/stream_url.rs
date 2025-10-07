use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ORIGIN, REFERER, ACCEPT};
use serde_json::Value;
use md5::{Digest, Md5};
use chrono::{Utc, FixedOffset};
use serde::Serialize;


#[derive(Clone, Debug, Serialize)]
#[allow(non_snake_case)]
pub struct HuyaUnifiedStreamEntry {
    pub quality: String,
    pub bitRate: i32,
    pub url: String,
}

#[derive(Clone, Debug, Serialize)]
#[allow(non_snake_case)]
pub struct HuyaUnifiedResponse {
    pub title: Option<String>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub introduction: Option<String>,
    pub profileRoom: Option<String>,
    pub is_live: bool,
    pub flv_tx_urls: Vec<HuyaUnifiedStreamEntry>,
    pub selected_url: Option<String>,
}

fn md5_hex(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

fn current_millis() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    now.as_millis() as i64
}

fn asia_shanghai_sv() -> String {
    // Asia/Shanghai yyyyMMddHH using chrono
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let dt = Utc::now().with_timezone(&tz);
    dt.format("%Y%m%d%H").to_string()
}

fn parse_query(qs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in url::form_urlencoded::parse(qs.as_bytes()) {
        map.insert(k.into_owned(), v.into_owned());
    }
    map
}

fn url_decode(s: &str) -> String {
    url::form_urlencoded::parse(format!("a={}", s).as_bytes())
        .find(|(k, _)| k == "a")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| s.to_string())
}

fn parse_uid_from_cookie_or_stream(cookie: Option<&str>, stream_name: &str) -> i64 {
    if let Some(ck) = cookie {
        if ck.contains("yyuid=") {
            let re = Regex::new(r"yyuid=(\\d+)").unwrap();
            if let Some(caps) = re.captures(ck) {
                if let Some(m) = caps.get(1) {
                    if let Ok(uid) = m.as_str().parse::<i64>() {
                        if uid > 0 { return uid; }
                    }
                }
            }
        }
    }
    let parts: Vec<&str> = stream_name.split('-').collect();
    if let Some(first) = parts.first() {
        if let Ok(uid) = first.parse::<i64>() { if uid > 0 { return uid; } }
    }
    // Fallback large uid
    1400000000000i64 + (current_millis() % 100000000000i64)
}

fn process_anticode(anticode: &str, stream_name: &str, cookie: Option<&str>) -> String {
    let q = parse_query(anticode);
    let uid = parse_uid_from_cookie_or_stream(cookie, stream_name);

    let ctype = q.get("ctype").cloned().unwrap_or_else(|| "huya_live".to_string());
    let t = q.get("t").cloned().unwrap_or_else(|| "100".to_string());
    let ws_time = q.get("wsTime").cloned().unwrap_or_default();

    let convert_uid: i64 = (((uid as u32) << 8) as u64 | ((uid as u32) >> 24) as u64) as i64 & 0xFFFFFFFF;
    let seqid = (current_millis() + uid).to_string();

    // fm param base64 decode after url decode
    let fm_raw = q.get("fm").map(|s| url_decode(s)).unwrap_or_default();
    let fm_decoded = if fm_raw.is_empty() { String::new() } else {
        match general_purpose::STANDARD.decode(fm_raw.as_bytes()) {
            Ok(bytes) => String::from_utf8(bytes).unwrap_or_default(),
            Err(_) => String::new(),
        }
    };
    let ws_prefix = fm_decoded.split('_').next().unwrap_or("").to_string();

    let ws_hash = md5_hex(&format!("{}|{}|{}", seqid, ctype, t));
    let ws_secret = md5_hex(&format!("{}_{}_{}_{}_{}", ws_prefix, convert_uid, stream_name, ws_hash, ws_time));

    // Asia/Shanghai yyyyMMddHH
    let sv = asia_shanghai_sv();

    let fs = q.get("fs").cloned().unwrap_or_default();

    // uuid
    let ct_base = if !ws_time.is_empty() {
        i64::from_str_radix(&ws_time, 16).unwrap_or(current_millis() / 1000)
    } else {
        current_millis() / 1000
    };
    let ct = ((ct_base as f64 + 0.12345) * 1000.0) as i64;
    let uuid = (((ct % 10_000_000_000) as f64 + 0.6789) * 1_000.0) as i64 & 0xFFFF_FFFF;

    let mut params: Vec<(String, String)> = vec![
        ("wsSecret".into(), ws_secret),
        ("wsTime".into(), ws_time),
        ("seqid".into(), seqid),
        ("ctype".into(), ctype.clone()),
        ("ver".into(), "1".into()),
    ];
    if !fs.is_empty() { params.push(("fs".into(), fs)); }
    params.extend([
        ("t".into(), t),
        ("u".into(), convert_uid.to_string()),
        ("uuid".into(), uuid.to_string()),
        ("sdk_sid".into(), current_millis().to_string()),
        ("codec".into(), "264".into()),
        ("sv".into(), sv),
        ("dMod".into(), "mseh-0".into()),
        ("sdkPcdn".into(), "1_1".into()),
        ("a_block".into(), "0".into()),
    ]);

    params
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&")
}

#[allow(dead_code)]
async fn check_live_status(client: &reqwest::Client, room_id: &str) -> Result<bool, Box<dyn Error>> {
    let url = format!("https://m.huya.com/{}", room_id);
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.0.0 Safari/537.36"));

    let resp = client.get(&url).headers(headers).send().await?;
    let text = resp.text().await?;

    let re = Regex::new(r"window\\.HNF_GLOBAL_INIT.=.\{(.*?)\}\s*</script>").unwrap();
    if let Some(caps) = re.captures(&text) {
        let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let json_str = format!("{{{}}}", inner);
        let v: Value = serde_json::from_str(&json_str)?;
        let status = v
            .get("roomInfo")
            .and_then(|x| x.get("eLiveStatus"))
            .and_then(|x| x.as_i64())
            .unwrap_or(0);
        return Ok(status == 2);
    }
    Ok(false)
}

#[derive(Clone, Debug)]
struct LineInfo {
    line: String,
    line_type: String, // "flv" or "hls"
    flv_anticode: String,
    hls_anticode: String,
    stream_name: String,
    cdn_type: String, // e.g., "TX"
}

#[derive(Clone, Debug)]
struct RoomDetail {
    status: bool,
    lines: Vec<LineInfo>,
    bit_rates: Vec<(String, i32)>, // (name, bitRate)
    title: Option<String>,
    nick: Option<String>,
    #[allow(dead_code)]
    cover: Option<String>,
    #[allow(dead_code)]
    area: Option<String>,
    avatar180: Option<String>,
}



async fn fetch_room_detail(client: &reqwest::Client, room_id: &str) -> Result<RoomDetail, Box<dyn Error>> {
    let url = format!("https://mp.huya.com/cache.php?m=Live&do=profileRoom&roomid={}&showSecret=1", room_id);
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://www.huya.com"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.huya.com/"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.0.0 Safari/537.36"));

    let resp = client.get(&url).headers(headers).send().await?;
    let text = resp.text().await?;
    let v: Value = serde_json::from_str(&text)?;

    let status_code = v.get("status").and_then(|x| x.as_i64()).unwrap_or(0);
    let stream_ok = v
        .get("data")
        .and_then(|d| d.get("stream"))
        .is_some();
    if status_code != 200 || !stream_ok {
        return Ok(RoomDetail { status: false, lines: vec![], bit_rates: vec![], title: None, nick: None, cover: None, area: None, avatar180: None });
    }

    let data = v.get("data").unwrap();
    let base_list = data
        .get("stream")
        .and_then(|s| s.get("baseSteamInfoList"))
        .and_then(|b| b.as_array())
        .unwrap_or(&vec![])
        .clone();

    let mut lines: Vec<LineInfo> = Vec::new();
    for it in base_list {
        let s_flv = it.get("sFlvUrl").and_then(|x| x.as_str());
        let s_hls = it.get("sHlsUrl").and_then(|x| x.as_str());
        let flv_ac = it.get("sFlvAntiCode").and_then(|x| x.as_str()).unwrap_or("");
        let hls_ac = it.get("sHlsAntiCode").and_then(|x| x.as_str()).unwrap_or("");
        let stream_name = it.get("sStreamName").and_then(|x| x.as_str()).unwrap_or("");
        let cdn_type = it.get("sCdnType").and_then(|x| x.as_str()).unwrap_or("");
        if let Some(u) = s_flv {
            lines.push(LineInfo { line: u.to_string(), line_type: "flv".into(), flv_anticode: flv_ac.to_string(), hls_anticode: hls_ac.to_string(), stream_name: stream_name.to_string(), cdn_type: cdn_type.to_string() });
        }
        if let Some(u) = s_hls {
            lines.push(LineInfo { line: u.to_string(), line_type: "hls".into(), flv_anticode: flv_ac.to_string(), hls_anticode: hls_ac.to_string(), stream_name: stream_name.to_string(), cdn_type: cdn_type.to_string() });
        }
    }

    // bitRates
    let mut bit_rates: Vec<(String, i32)> = Vec::new();
    let br_str = data.get("liveData").and_then(|ld| ld.get("bitRateInfo")).and_then(|x| x.as_str());
    let mut brs_json: Vec<Value> = Vec::new();
    if let Some(bs) = br_str {
        if let Ok(arr) = serde_json::from_str::<Vec<Value>>(bs) { brs_json = arr; }
    }
    if brs_json.is_empty() {
        if let Some(arr) = data.get("stream").and_then(|s| s.get("flv")).and_then(|f| f.get("rateArray")).and_then(|x| x.as_array()) {
            brs_json = arr.clone();
        }
    }
    for b in brs_json {
        let nm = b.get("sDisplayName").and_then(|x| x.as_str()).unwrap_or_else(|| b.get("name").and_then(|x| x.as_str()).unwrap_or("原画"));
        let r = b.get("iBitRate").and_then(|x| x.as_i64()).unwrap_or_else(|| b.get("bitRate").and_then(|x| x.as_i64()).unwrap_or(0)) as i32;
        if !bit_rates.iter().any(|(n, _)| n == nm) {
            bit_rates.push((nm.to_string(), r));
        }
    }

    let title = data.get("liveData").and_then(|ld| ld.get("introduction")).and_then(|x| x.as_str()).map(|s| s.to_string());
    let nick = data.get("liveData").and_then(|ld| ld.get("nick")).and_then(|x| x.as_str()).map(|s| s.to_string());
    let cover = data.get("liveData").and_then(|ld| ld.get("screenshot")).and_then(|x| x.as_str()).map(|s| s.to_string());
    let area = data.get("liveData").and_then(|ld| ld.get("gameFullName")).and_then(|x| x.as_str()).map(|s| s.to_string());
    let avatar180 = data.get("liveData").and_then(|ld| ld.get("avatar180")).and_then(|x| x.as_str()).map(|s| s.to_string());

    Ok(RoomDetail { status: true, lines, bit_rates, title, nick, cover, area, avatar180 })
}

fn pick_stream_url(detail: &RoomDetail, quality: &str) -> Option<String> {
    // Map quality name to bit rate
    // 1) Exact name match
    let mut target_rate: Option<i32> = detail
        .bit_rates
        .iter()
        .find(|(name, _)| name == quality)
        .map(|(_, rate)| *rate);

    // 2) Special mapping by quality keyword
    if target_rate.is_none() {
        let mut positive_rates: Vec<i32> = detail
            .bit_rates
            .iter()
            .map(|(_, r)| *r)
            .filter(|r| *r > 0)
            .collect();
        positive_rates.sort();
        match quality {
            "原画" => {
                target_rate = Some(0);
            }
            "标清" => {
                if let Some(min) = positive_rates.first() { target_rate = Some(*min); }
            }
            "高清" => {
                if let Some(max) = positive_rates.last() { target_rate = Some(*max); }
            }
            _ => {
                // fallback: pick first positive if available
                if let Some(first) = positive_rates.first() { target_rate = Some(*first); } else { target_rate = Some(0); }
            }
        }
    }

    // Prefer FLV, TX CDN
    let mut candidates: Vec<&LineInfo> = detail
        .lines
        .iter()
        .filter(|l| l.line_type == "flv" && !l.flv_anticode.is_empty())
        .collect();
    // Sort with TX first
    candidates.sort_by_key(|l| { let t = l.cdn_type.to_uppercase(); if t == "TX" { 0 } else { 1 } });

    for l in candidates {
        let qs = process_anticode(&l.flv_anticode, &l.stream_name, None);
        let mut url = format!("{}/{}.flv?{}", l.line, l.stream_name, qs);
        if let Some(rate) = target_rate { if rate > 0 { url.push_str(&format!("&ratio={}", rate)); } }
        return Some(url);
    }

    // Fallback HLS
    let mut hls_candidates: Vec<&LineInfo> = detail
        .lines
        .iter()
        .filter(|l| l.line_type == "hls" && !l.hls_anticode.is_empty())
        .collect();
    hls_candidates.sort_by_key(|l| { let t = l.cdn_type.to_uppercase(); if t == "TX" { 0 } else { 1 } });
    for l in hls_candidates {
        let qs = process_anticode(&l.hls_anticode, &l.stream_name, None);
        let mut url = format!("{}/{}.m3u8?{}", l.line, l.stream_name, qs);
        if let Some(rate) = target_rate { if rate > 0 { url.push_str(&format!("&ratio={}", rate)); } }
        return Some(url);
    }

    None
}

#[tauri::command]
pub async fn get_huya_unified_cmd(room_id: String, quality: Option<String>) -> Result<HuyaUnifiedResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let detail = fetch_room_detail(&client, &room_id)
        .await
        .map_err(|e| e.to_string())?;

    if !detail.status {
        return Err("主播未开播或获取虎牙房间详情失败".to_string());
    }

    let selected = pick_stream_url(
        &detail,
        quality.as_deref().unwrap_or("原画"),
    );

    let tx_entries = build_flv_tx_urls(&detail);

    Ok(HuyaUnifiedResponse {
        title: detail.title.clone(),
        nick: detail.nick.clone(),
        avatar: detail.avatar180.clone(),
        introduction: None,
        profileRoom: None,
        is_live: detail.status,
        flv_tx_urls: tx_entries,
        selected_url: selected,
    })
}

fn build_flv_tx_urls(detail: &RoomDetail) -> Vec<HuyaUnifiedStreamEntry> {
    // 选取一个可用的 FLV 基础地址，优先 TX CDN
    let mut flv_candidates: Vec<&LineInfo> = detail
        .lines
        .iter()
        .filter(|l| l.line_type == "flv" && !l.flv_anticode.is_empty())
        .collect();
    flv_candidates.sort_by_key(|l| { let t = l.cdn_type.to_uppercase(); if t == "TX" { 0 } else { 1 } });

    let base: Option<(String /*base_url*/, String /*stream_name*/)> = flv_candidates
        .into_iter()
        .next()
        .map(|l| {
            let qs = process_anticode(&l.flv_anticode, &l.stream_name, None);
            let base_url = format!("{}/{}.flv?{}", l.line, l.stream_name, qs);
            (base_url, l.stream_name.clone())
        });

    let mut entries: Vec<HuyaUnifiedStreamEntry> = Vec::new();
    if let Some((base_url, _stream_name)) = base {
        for (quality, r) in &detail.bit_rates {
            let url_with_ratio = if *r > 0 { format!("{}&ratio={}", base_url, r) } else { base_url.clone() };
            entries.push(HuyaUnifiedStreamEntry { quality: quality.clone(), bitRate: *r, url: url_with_ratio });
        }
    }

    entries
}
#[allow(dead_code)]
const HEARTBEAT_BASE64: &str = "ABQdAAwsNgBM"; // same as Python