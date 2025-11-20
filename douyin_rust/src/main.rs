use reqwest::{Client, Proxy};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ---- a_bogus generation (ported from Python) ----
fn rc4_encrypt(plaintext: &str, key: &str) -> String {
    // Match Python: iterate over Unicode scalars, each expected within 0-255.
    let pbytes: Vec<u8> = plaintext.chars().map(|c| c as u32 as u8).collect();
    let kbytes: Vec<u8> = key.chars().map(|c| c as u32 as u8).collect();

    let mut s: Vec<usize> = (0..256).collect();
    let mut j = 0usize;
    for i in 0..256 {
        j = (j + s[i] + kbytes[i % kbytes.len()] as usize) % 256;
        s.swap(i, j);
    }

    let mut i = 0usize;
    j = 0usize;
    let mut result = String::with_capacity(pbytes.len());
    for ch in pbytes {
        i = (i + 1) % 256;
        j = (j + s[i]) % 256;
        s.swap(i, j);
        let t = (s[i] + s[j]) % 256;
        let out = (ch ^ s[t] as u8) as char;
        result.push(out);
    }
    result
}

fn left_rotate(x: u32, n: u32) -> u32 {
    let n = n % 32;
    (x << n) | (x >> (32 - n))
}

fn get_t_j(j: usize) -> u32 {
    if j < 16 {
        0x79CC4519
    } else {
        0x7A879D8A
    }
}

fn ff_j(j: usize, x: u32, y: u32, z: u32) -> u32 {
    if j < 16 { x ^ y ^ z } else { (x & y) | (x & z) | (y & z) }
}

fn gg_j(j: usize, x: u32, y: u32, z: u32) -> u32 {
    if j < 16 { x ^ y ^ z } else { (x & y) | (!x & z) }
}

struct SM3 {
    reg: [u32; 8],
    chunk: Vec<u8>,
    size: usize,
}

impl SM3 {
    fn new() -> Self {
        let mut sm3 = SM3 { reg: [0; 8], chunk: Vec::new(), size: 0 };
        sm3.reset();
        sm3
    }

    fn reset(&mut self) {
        self.reg = [
            1937774191, 1226093241, 388252375, 3666478592,
            2842636476, 372324522, 3817729613, 2969243214,
        ];
        self.chunk.clear();
        self.size = 0;
    }

    fn write(&mut self, data: &[u8]) {
        self.size += data.len();
        let mut remaining = data;
        while !remaining.is_empty() {
            let needed = 64 - self.chunk.len();
            let take = remaining.len().min(needed);
            self.chunk.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];
            if self.chunk.len() == 64 {
                self.compress_block();
            }
        }
    }

    fn fill(&mut self) {
        let bit_length = (self.size as u64) * 8;
        self.chunk.push(0x80);
        while (self.chunk.len() % 64) != 56 {
            self.chunk.push(0);
        }
        self.chunk.extend_from_slice(&bit_length.to_be_bytes());
    }

    fn compress_block(&mut self) {
        if self.chunk.len() < 64 { return; }
        let mut w = [0u32; 132];
        for t in 0..16 {
            let i = 4 * t;
            w[t] = ((self.chunk[i] as u32) << 24)
                | ((self.chunk[i + 1] as u32) << 16)
                | ((self.chunk[i + 2] as u32) << 8)
                | (self.chunk[i + 3] as u32);
        }
        for j in 16..68 {
            let a = w[j - 16] ^ w[j - 9] ^ left_rotate(w[j - 3], 15);
            let a = a ^ left_rotate(a, 15) ^ left_rotate(a, 23);
            w[j] = (a ^ left_rotate(w[j - 13], 7) ^ w[j - 6]) & 0xFFFFFFFF;
        }
        for j in 0..64 {
            w[j + 68] = (w[j] ^ w[j + 4]) & 0xFFFFFFFF;
        }

        let mut a = self.reg[0];
        let mut b = self.reg[1];
        let mut c = self.reg[2];
        let mut d = self.reg[3];
        let mut e = self.reg[4];
        let mut f = self.reg[5];
        let mut g = self.reg[6];
        let mut h = self.reg[7];

        for j in 0..64 {
            let ss1 = left_rotate((left_rotate(a, 12).wrapping_add(e).wrapping_add(left_rotate(get_t_j(j), j as u32))) & 0xFFFFFFFF, 7);
            let ss2 = ss1 ^ left_rotate(a, 12);
            let tt1 = ff_j(j, a, b, c).wrapping_add(d).wrapping_add(ss2).wrapping_add(w[j + 68]) & 0xFFFFFFFF;
            let tt2 = gg_j(j, e, f, g).wrapping_add(h).wrapping_add(ss1).wrapping_add(w[j]) & 0xFFFFFFFF;

            d = c;
            c = left_rotate(b, 9);
            b = a;
            a = tt1;
            h = g;
            g = left_rotate(f, 19);
            f = e;
            e = tt2 ^ left_rotate(tt2, 9) ^ left_rotate(tt2, 17);
        }

        self.reg[0] ^= a;
        self.reg[1] ^= b;
        self.reg[2] ^= c;
        self.reg[3] ^= d;
        self.reg[4] ^= e;
        self.reg[5] ^= f;
        self.reg[6] ^= g;
        self.reg[7] ^= h;
        self.chunk.clear();
    }

    fn sum_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        self.reset();
        self.write(data);
        self.fill();
        for block in self.chunk.clone().chunks(64) {
            self.chunk = block.to_vec();
            self.compress_block();
        }
        let mut out = Vec::with_capacity(32);
        for &c in &self.reg {
            out.extend_from_slice(&c.to_be_bytes());
        }
        self.reset();
        out
    }
}

fn sm3_sum(data: &[u8]) -> Vec<u8> {
    let mut sm3 = SM3::new();
    sm3.sum_bytes(data)
}

fn get_long_int(round_num: usize, long_str: &str) -> u32 {
    // Operate on Unicode scalar values (Python `chr` behavior), not UTF-8 bytes.
    let chars: Vec<u32> = long_str.chars().map(|c| c as u32).collect();
    let i = round_num * 3;
    let b1 = *chars.get(i).unwrap_or(&0);
    let b2 = *chars.get(i + 1).unwrap_or(&0);
    let b3 = *chars.get(i + 2).unwrap_or(&0);
    (b1 << 16) | (b2 << 8) | b3
}

fn result_encrypt(long_str: &str, num: &str) -> String {
    let encoding_tables: HashMap<&str, &str> = HashMap::from([
        ("s0", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/="),
        ("s1", "Dkdpgh4ZKsQB80/Mfvw36XI1R25+WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe="),
        ("s2", "Dkdpgh4ZKsQB80/Mfvw36XI1R25-WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe="),
        ("s3", "ckdp1h4ZKsUB80/Mfvw36XIgR25+WQAlEi7NLboqYTOPuzmFjJnryx9HVGDaStCe"),
        ("s4", "Dkdpgh2ZmsQB80/MfvV36XI1R45-WUAlEixNLwoqYTOPuzKFjJnry79HbGcaStCe"),
    ]);
    let masks = [16515072u32, 258048, 4032, 63];
    let shifts = [18u32, 12, 6, 0];
    let table = encoding_tables[num].as_bytes();

    let mut result = String::new();
    let char_len = long_str.chars().count();
    let mut round_num = 0usize;
    let mut long_int = get_long_int(round_num, long_str);
    let total_chars = ((char_len as f64 / 3.0) * 4.0).ceil() as usize;
    for i in 0..total_chars {
        if i / 4 != round_num {
            round_num += 1;
            long_int = get_long_int(round_num, long_str);
        }
        let idx = i % 4;
        let char_index = ((long_int & masks[idx]) >> shifts[idx]) as usize;
        result.push(table[char_index] as char);
    }
    result
}

fn gener_random(random_num: i32, option: [i32; 2]) -> Vec<u8> {
    let byte1 = random_num & 255;
    let byte2 = (random_num >> 8) & 255;
    vec![
        ((byte1 & 170) | (option[0] & 85)) as u8,
        ((byte1 & 85) | (option[0] & 170)) as u8,
        ((byte2 & 170) | (option[1] & 85)) as u8,
        ((byte2 & 85) | (option[1] & 170)) as u8,
    ]
}

fn generate_random_str() -> String {
    let vals = [0.123456789f64, 0.987654321f64, 0.555555555f64];
    let mut bytes = Vec::new();
    bytes.extend(gener_random((vals[0] * 10000.0) as i32, [3, 45]));
    bytes.extend(gener_random((vals[1] * 10000.0) as i32, [1, 0]));
    bytes.extend(gener_random((vals[2] * 10000.0) as i32, [1, 5]));
    bytes.into_iter().map(|b| b as char).collect()
}

fn generate_rc4_bb_str(url_search_params: &str, user_agent: &str, window_env_str: &str, suffix: &str, arguments: [i32; 3]) -> String {
    let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

    // double SM3 hashing like python: sm3.sum(sm3.sum(...))
    let url_list = sm3_sum(&sm3_sum(format!("{}{}", url_search_params, suffix).as_bytes()));
    let cus_once = sm3_sum(suffix.as_bytes());
    let cus = sm3_sum(&cus_once);
    let ua_key = [0u8, 1u8, 14u8];
    let ua = sm3_sum(result_encrypt(&rc4_encrypt(user_agent, &String::from_utf8_lossy(&ua_key)), "s3").as_bytes());

    let end_time = start_time + 100;

    // fixed-size array to mirror python dict indices
    let mut b = vec![0i64; 80];
    b[8] = 3;
    b[10] = end_time;
    b[16] = start_time;
    b[18] = 44;

    let split_to_bytes = |num: i64| -> [i64; 4] {
        [
            (num >> 24) & 255,
            (num >> 16) & 255,
            (num >> 8) & 255,
            num & 255,
        ]
    };

    let st_bytes = split_to_bytes(b[16]);
    b[20] = st_bytes[0];
    b[21] = st_bytes[1];
    b[22] = st_bytes[2];
    b[23] = st_bytes[3];
    b[24] = (b[16] / 256 / 256 / 256 / 256) & 255;
    b[25] = (b[16] / 256 / 256 / 256 / 256 / 256) & 255;

    let arg0_bytes = split_to_bytes(arguments[0] as i64);
    b[26] = arg0_bytes[0];
    b[27] = arg0_bytes[1];
    b[28] = arg0_bytes[2];
    b[29] = arg0_bytes[3];

    b[30] = (arguments[1] / 256) as i64 & 255;
    b[31] = (arguments[1] % 256) as i64 & 255;

    let arg1_bytes = split_to_bytes(arguments[1] as i64);
    b[32] = arg1_bytes[0];
    b[33] = arg1_bytes[1];

    let arg2_bytes = split_to_bytes(arguments[2] as i64);
    b[34] = arg2_bytes[0];
    b[35] = arg2_bytes[1];
    b[36] = arg2_bytes[2];
    b[37] = arg2_bytes[3];

    b[38] = *url_list.get(21).unwrap_or(&0) as i64;
    b[39] = *url_list.get(22).unwrap_or(&0) as i64;
    b[40] = *cus.get(21).unwrap_or(&0) as i64;
    b[41] = *cus.get(22).unwrap_or(&0) as i64;
    b[42] = *ua.get(23).unwrap_or(&0) as i64;
    b[43] = *ua.get(24).unwrap_or(&0) as i64;

    let et_bytes = split_to_bytes(b[10]);
    b[44] = et_bytes[0];
    b[45] = et_bytes[1];
    b[46] = et_bytes[2];
    b[47] = et_bytes[3];
    b[48] = b[8];
    b[49] = (b[10] / 256 / 256 / 256 / 256) & 255;
    b[50] = (b[10] / 256 / 256 / 256 / 256 / 256) & 255;

    let page_id = 110624i64;
    b[51] = page_id;
    let page_bytes = split_to_bytes(page_id);
    b[52] = page_bytes[0];
    b[53] = page_bytes[1];
    b[54] = page_bytes[2];
    b[55] = page_bytes[3];

    let aid = 6383i64;
    b[56] = aid;
    b[57] = aid & 255;
    b[58] = (aid >> 8) & 255;
    b[59] = (aid >> 16) & 255;
    b[60] = (aid >> 24) & 255;

    let window_env_list: Vec<i64> = window_env_str.chars().map(|c| c as i64).collect();
    b[64] = window_env_list.len() as i64;
    b[65] = (window_env_list.len() as i64) & 255;
    b[66] = ((window_env_list.len() as i64) >> 8) & 255;
    b[69] = 0;
    b[70] = 0;
    b[71] = 0;

    let checksum = b[18] ^ b[20] ^ b[26] ^ b[30] ^ b[38] ^ b[40] ^ b[42] ^ b[21] ^ b[27] ^ b[31] ^ b[35] ^ b[39]
        ^ b[41] ^ b[43] ^ b[22] ^ b[28] ^ b[32] ^ b[36] ^ b[23] ^ b[29] ^ b[33] ^ b[37] ^ b[44] ^ b[45]
        ^ b[46] ^ b[47] ^ b[48] ^ b[49] ^ b[50] ^ b[24] ^ b[25] ^ b[52] ^ b[53] ^ b[54] ^ b[55] ^ b[57]
        ^ b[58] ^ b[59] ^ b[60] ^ b[65] ^ b[66] ^ b[70] ^ b[71];
    b[72] = checksum;

    let mut bb: Vec<i64> = vec![
        b[18], b[20], b[52], b[26], b[30], b[34], b[58], b[38], b[40], b[53], b[42], b[21],
        b[27], b[54], b[55], b[31], b[35], b[57], b[39], b[41], b[43], b[22], b[28], b[32],
        b[60], b[36], b[23], b[29], b[33], b[37], b[44], b[45], b[59], b[46], b[47], b[48],
        b[49], b[50], b[24], b[25], b[65], b[66], b[70], b[71],
    ];
    bb.extend(window_env_list);
    bb.push(checksum);

    let plaintext: String = bb.into_iter().map(|b| (b as u8) as char).collect();
    rc4_encrypt(&plaintext, &(121u8 as char).to_string())
}

fn ab_sign(url_search_params: &str, user_agent: &str) -> String {
    let window_env_str = "1920|1080|1920|1040|0|30|0|0|1872|92|1920|1040|1857|92|1|24|Win32";
    format!(
        "{}=",
        result_encrypt(
            &format!(
                "{}{}",
                generate_random_str(),
                generate_rc4_bb_str(url_search_params, user_agent, window_env_str, "cus", [0, 1, 14])
            ),
            "s4"
        )
    )
}

// ---- Douyin stream fetch ----
const MOBILE_UA: &str = "Mozilla/5.0 (Linux; Android 11; SAMSUNG SM-G973U) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/14.2 Chrome/87.0.4280.141 Mobile Safari/537.36";
const WEB_UA: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.5845.97 Safari/537.36 Core/1.116.567.400 QQBrowser/19.7.6764.400";
const APP_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0";
const DEFAULT_COOKIE: &str = "ttwid=1%7C2iDIYVmjzMcpZ20fcaFde0VghXAA3NaNXE_SLR68IyE%7C1761045455%7Cab35197d5cfb21df6cbb2fa7ef1c9262206b062c315b9d04da746d0b37dfbc7d";

async fn http_get(url: &str, headers: &HashMap<&str, &str>, proxy: Option<&str>) -> reqwest::Result<(u16, String)> {
    let mut client_builder = Client::builder().danger_accept_invalid_certs(true);
    if let Some(p) = proxy { client_builder = client_builder.proxy(Proxy::all(p)?); }
    let client = client_builder.build()?;
    let mut req = client.get(url);
    for (k, v) in headers { req = req.header(*k, *v); }
    let resp = req.send().await?;
    let status = resp.status().as_u16();
    let bytes = resp.bytes().await?;
    let len = bytes.len();
    let text = String::from_utf8_lossy(&bytes).to_string();
    if text.trim().is_empty() {
        println!("debug: empty body, status {status}, content-length {}", len);
    }
    Ok((status, text))
}

async fn resolve_room_and_sec(url: &str, proxy: Option<&str>) -> anyhow::Result<(String, String)> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent", MOBILE_UA);
    headers.insert("Accept-Language", "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2");
    headers.insert("Cookie", "s_v_web_id=verify_lk07kv74_QZYCUApD_xhiB_405x_Ax51_GYO9bUIyZQVf");
    let mut client_builder = Client::builder();
    if let Some(p) = proxy { client_builder = client_builder.proxy(Proxy::all(p)?); }
    let client = client_builder.build()?;
    let resp = client.get(url).header("User-Agent", MOBILE_UA).send().await?;
    let redirect_url = resp.url().clone();
    let redirect = redirect_url.as_str().to_string();
    if !redirect.contains("reflow/") {
        anyhow::bail!("Unsupported share link, cannot find reflow segment");
    }
    let sec_user_id = redirect
        .split("sec_user_id=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or_else(|| anyhow::anyhow!("sec_user_id not found"))?;
    let room_id = redirect
        .split('?')
        .next()
        .and_then(|s| s.rsplit('/').next())
        .ok_or_else(|| anyhow::anyhow!("room_id not found"))?;
    Ok((room_id.to_string(), sec_user_id.to_string()))
}

fn merge_origin_stream(room_data: &mut Value) {
    let stream_url = match room_data.get_mut("stream_url") { Some(v) => v, None => return };
    let live_core_sdk_data = stream_url.get("live_core_sdk_data");
    if live_core_sdk_data.is_none() { return; }

    let pull_datas = stream_url.get("pull_datas").and_then(|v| v.as_object());
    let json_str = if let Some(pd) = pull_datas {
        if let Some((_, entry)) = pd.iter().next() {
            entry.get("stream_data").and_then(|s| s.as_str()).map(|s| s.to_string())
        } else { None }
    } else {
        live_core_sdk_data
            .and_then(|d| d.get("pull_data"))
            .and_then(|p| p.get("stream_data"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
    };
    let Some(json_str) = json_str else { return; };
    let parsed: Value = serde_json::from_str(&json_str).unwrap_or(Value::Null);
    let origin_main = parsed
        .get("data").and_then(|d| d.get("origin")).and_then(|o| o.get("main"));
    if origin_main.is_none() { return; }
    let origin_main = origin_main.unwrap();
    let origin_codec = origin_main
        .get("sdk_params")
        .and_then(|s| s.as_str())
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
        .and_then(|v| v.get("VCodec").cloned())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let origin_m3u8 = format!(
        "{}&codec={}",
        origin_main.get("hls").and_then(|v| v.as_str()).unwrap_or(""),
        origin_codec
    );
    let origin_flv = format!(
        "{}&codec={}",
        origin_main.get("flv").and_then(|v| v.as_str()).unwrap_or(""),
        origin_codec
    );

    if let Some(map) = stream_url.get_mut("hls_pull_url_map").and_then(|m| m.as_object_mut()) {
        let mut new_map = serde_json::Map::new();
        new_map.insert("ORIGIN".to_string(), Value::String(origin_m3u8));
        for (k, v) in map.iter() { new_map.insert(k.clone(), v.clone()); }
        *map = new_map;
    }
    if let Some(map) = stream_url.get_mut("flv_pull_url").and_then(|m| m.as_object_mut()) {
        let mut new_map = serde_json::Map::new();
        new_map.insert("ORIGIN".to_string(), Value::String(origin_flv));
        for (k, v) in map.iter() { new_map.insert(k.clone(), v.clone()); }
        *map = new_map;
    }
}

async fn fetch_web_room(url: &str, proxy: Option<&str>, cookies: Option<&str>) -> anyhow::Result<Value> {
    let mut headers = HashMap::new();
    headers.insert("cookie", cookies.unwrap_or(DEFAULT_COOKIE));
    headers.insert("referer", "https://live.douyin.com/335354047186");
    headers.insert("user-agent", WEB_UA);
    headers.insert("accept-encoding", "identity");

    let web_rid = url.split('?').next().unwrap_or(url).split("live.douyin.com/").last().unwrap_or("");
    let params = vec![
        ("aid", "6383"),
        ("app_name", "douyin_web"),
        ("live_id", "1"),
        ("device_platform", "web"),
        ("language", "zh-CN"),
        ("browser_language", "zh-CN"),
        ("browser_platform", "Win32"),
        ("browser_name", "Chrome"),
        ("browser_version", "116.0.0.0"),
        ("web_rid", web_rid),
        ("msToken", ""),
    ];
    let query = serde_urlencoded::to_string(&params)?;
    let mut api = format!("https://live.douyin.com/webcast/room/web/enter/?{}", query);
    let sign = ab_sign(&query, WEB_UA);
    api.push_str("&a_bogus=");
    api.push_str(&sign);

    let (status, json_str) = http_get(&api, &headers, proxy).await?;
    let data: Value = serde_json::from_str(&json_str).map_err(|e| {
        anyhow::anyhow!(
            "web parse json error: {e}. status: {status}. resp snippet: {}",
            &json_str.chars().take(200).collect::<String>()
        )
    })?;
    let mut room_data = data["data"]["data"][0].clone();
    if let Some(user) = data["data"]["user"].get("nickname") {
        room_data["anchor_name"] = user.clone();
    }
    merge_origin_stream(&mut room_data);
    Ok(room_data)
}

async fn fetch_app_room(url: &str, proxy: Option<&str>, cookies: Option<&str>) -> anyhow::Result<Value> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent", APP_UA);
    headers.insert("Accept-Language", "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2");
    headers.insert("Referer", "https://live.douyin.com/");
    headers.insert("accept-encoding", "identity");
    headers.insert("Cookie", cookies.unwrap_or(DEFAULT_COOKIE));

    let (room_id, sec_uid) = resolve_room_and_sec(url, proxy).await?;
    let params = vec![
        ("verifyFp", "verify_hwj52020_7szNlAB7_pxNY_48Vh_ALKF_GA1Uf3yteoOY"),
        ("type_id", "0"),
        ("live_id", "1"),
        ("room_id", room_id.as_str()),
        ("sec_user_id", sec_uid.as_str()),
        ("version_code", "99.99.99"),
        ("app_id", "1128"),
    ];
    let query = serde_urlencoded::to_string(&params)?;
    let mut api = format!("https://webcast.amemv.com/webcast/room/reflow/info/?{}", query);
    let sign = ab_sign(&query, APP_UA);
    api.push_str("&a_bogus=");
    api.push_str(&sign);

    let (status, json_str) = http_get(&api, &headers, proxy).await?;
    let data: Value = serde_json::from_str(&json_str).map_err(|e| {
        anyhow::anyhow!(
            "app parse json error: {e}. status: {status}. resp snippet: {}",
            &json_str.chars().take(200).collect::<String>()
        )
    })?;
    let mut room_data = data["data"]["room"].clone();
    if let Some(name) = room_data["owner"].get("nickname") {
        room_data["anchor_name"] = name.clone();
    }
    merge_origin_stream(&mut room_data);
    Ok(room_data)
}

fn pick_stream(room_data: &Value, quality: &str) -> Value {
    if room_data.get("status").and_then(|v| v.as_i64()).unwrap_or(4) != 2 {
        return serde_json::json!({
            "anchor_name": room_data.get("anchor_name"),
            "is_live": false
        });
    }
    let stream_url = &room_data["stream_url"];
    let mut flv_list: Vec<String> = stream_url["flv_pull_url"].as_object()
        .map(|m| m.values().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut m3u8_list: Vec<String> = stream_url["hls_pull_url_map"].as_object()
        .map(|m| m.values().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    while flv_list.len() < 5 { if let Some(last) = flv_list.last().cloned() { flv_list.push(last.clone()); m3u8_list.push(m3u8_list.last().cloned().unwrap_or_default()); } else { break; } }
    let quality_idx = match quality.to_uppercase().as_str() {
        "OD" | "BD" => 0,
        "UHD" => 1,
        "HD" => 2,
        "SD" => 3,
        "LD" => 4,
        _ => 0,
    };
    let idx = quality_idx.min(flv_list.len().saturating_sub(1));
    serde_json::json!({
        "anchor_name": room_data.get("anchor_name"),
        "title": room_data.get("title"),
        "is_live": true,
        "quality": quality,
        "m3u8_url": m3u8_list.get(idx),
        "flv_url": flv_list.get(idx),
    })
}

async fn fetch_douyin_stream(url: &str, quality: &str, proxy: Option<&str>, cookies: Option<&str>) -> anyhow::Result<Value> {
    let room_data = if url.contains("live.douyin.com") {
        fetch_web_room(url, proxy, cookies).await?
    } else {
        fetch_app_room(url, proxy, cookies).await?
    };
    Ok(pick_stream(&room_data, quality))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let test_url = "https://live.douyin.com/126833924894";
    let result = fetch_douyin_stream(test_url, "OD", None, None).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
