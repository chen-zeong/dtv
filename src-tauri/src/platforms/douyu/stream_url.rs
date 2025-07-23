use deno_core::{JsRuntime, RuntimeOptions}; // 替换 rquickjs 导入
use isahc::config::{Configurable, RedirectPolicy};
use isahc::{http, prelude::*, HttpClient, Request};
use md5::Digest;
use regex::Regex;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH}; // Added for JSON deserialization

#[derive(Deserialize, Debug)]
struct RoomInfoData {
    // Assuming '1' is live, '2' is offline/not broadcasting, '3' might be replay/video.
    // This field name might need adjustment based on actual API response.
    room_status: Option<String>,
    // Add other fields if needed, e.g., room_name, owner_name
}

#[derive(Deserialize, Debug)]
struct RoomInfoResponse {
    error: i32,
    data: Option<RoomInfoData>,
}

struct DouYu {
    did: String,
    rid: String,
    client: HttpClient,
}

impl DouYu {
    async fn new(rid: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = HttpClient::builder()
            .redirect_policy(RedirectPolicy::Follow)
            .default_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        Ok(Self {
            did: "10000000000000000000000000001501".to_string(),
            rid: rid.to_string(),
            client,
        })
    }

    fn md5(data: &str) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    async fn get_pc_js(&self, cdn: &str, rate: i32) -> Result<String, Box<dyn std::error::Error>> {
        match self.check_room_status().await {
            Ok(true) => {
                println!(
                    "[Douyu Stream URL] Room {} is live. Proceeding to fetch stream URL.",
                    self.rid
                );
            }
            Ok(false) => {
                println!(
                    "[Douyu Stream URL] Room {} is not live. Aborting stream fetch.",
                    self.rid
                );
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "主播未开播",
                )));
            }
            Err(e) => {
                println!("[Douyu Stream URL] Error checking room status for room {}: {}. Proceeding with caution or returning error.", self.rid, e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("检查房间状态失败: {}", e),
                )));
            }
        }
        // 获取PC网页内容
        let request = Request::builder()
            .method(http::Method::GET)
            .uri(&format!("https://www.douyu.com/{}", self.rid))
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header("Connection", "keep-alive")
            .body(())?;

        let text = self.client.send(request)?.text()?;
        // 提取JS函数
        let re = Regex::new(r"(vdwdae325w_64we[\s\S]*function ub98484234[\s\S]*?)function")?;
        let result = re
            .captures(&text)
            .ok_or("Cannot find js function")?
            .get(1)
            .ok_or("No capture group")?
            .as_str();

        let re_eval = Regex::new(r"eval.*?;\}")?;
        let func_ub9 = re_eval.replace_all(result, "strc;}");

        let mut runtime = JsRuntime::new(RuntimeOptions::default());

        // 将字符串转换为静态字符串
        let func_ub9_static = String::from(func_ub9);
        runtime.execute_script("[douyu]", deno_core::FastString::from(func_ub9_static))?;
        let js_result = runtime.execute_script(
            "[douyu]",
            deno_core::FastString::from(String::from("ub98484234()")),
        )?;

        // 获取 JavaScript 执行结果
        let res = {
            let scope = &mut runtime.handle_scope();
            let result = js_result.open(scope);
            result.to_rust_string_lossy(scope)
        };

        // 提取v参数
        let re = Regex::new(r"v=(\d+)")?;
        let v = re
            .captures(&res)
            .ok_or("v parameter not found")?
            .get(1)
            .ok_or("No capture group")?
            .as_str();

        let t10 = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();

        let rb = Self::md5(&format!("{}{}{}{}", self.rid, self.did, &t10, v));

        // 构造签名函数
        let func_sign = res.replace("return rt;})", "return rt;}");
        let func_sign = func_sign.replace("(function (", "function sign(");
        let func_sign = func_sign.replace("CryptoJS.MD5(cb).toString()", &format!("\"{}\"", rb));

        let func_sign_static = String::from(func_sign);
        runtime.execute_script("[douyu]", deno_core::FastString::from(func_sign_static))?;

        let sign_call = format!("sign(\"{}\", \"{}\", \"{}\");", self.rid, self.did, t10);

        let sign_call_static = String::from(sign_call);
        let js_params =
            runtime.execute_script("[douyu]", deno_core::FastString::from(sign_call_static))?;

        // 获取签名结果
        let mut params = {
            let scope = &mut runtime.handle_scope();
            let result = js_params.open(scope);
            result.to_rust_string_lossy(scope)
        };

        params.push_str(&format!("&cdn={}&rate={}", cdn, rate));

        // 获取真实URL
        let url = format!("https://www.douyu.com/lapi/live/getH5Play/{}", self.rid);
        let request = Request::builder()
            .method(http::Method::POST)
            .uri(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "https://www.douyu.com")
            .header("Referer", format!("https://www.douyu.com/{}", self.rid))
            .body(params)?;

        let mut response = self.client.send(request)?;

        let json: serde_json::Value = response.json()?;

        let data = json["data"]
            .as_object()
            .ok_or("No data field in response")?;
        let rtmp_url = data["rtmp_url"].as_str().ok_or("No rtmp_url field")?;
        let rtmp_live = data["rtmp_live"].as_str().ok_or("No rtmp_live field")?;

        let final_url = format!("{}/{}", rtmp_url, rtmp_live);

        Ok(final_url)
    }

    pub async fn get_real_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.get_pc_js("ws-h5", 0).await
    }

    async fn check_room_status(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let room_api_url = format!("http://open.douyucdn.cn/api/RoomApi/room/{}", self.rid);

        let request = Request::builder()
            .method(http::Method::GET)
            .uri(&room_api_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .body(())?;

        let mut response = self.client.send_async(request).await?;

        if !response.status().is_success() {
            return Err(
                format!("Room API request failed with status: {}", response.status()).into(),
            );
        }

        let room_info_response: RoomInfoResponse = response.json().await?;
        println!(
            "[Douyu Stream URL] Room API response for {}: {:?}",
            self.rid, room_info_response
        );

        if room_info_response.error != 0 {
            return Err(
                format!("Room API returned error code: {}", room_info_response.error).into(),
            );
        }

        match room_info_response.data {
            Some(data) => {
                match data.room_status.as_deref() {
                    Some("1") => Ok(true),  // Live
                    Some("2") => Ok(false), // Not live
                    Some(_status) => {
                        Ok(false) // Or handle as an error / unknown state
                    }
                    None => {
                        Ok(false) // Or handle as an error
                    }
                }
            }
            None => Err("No 'data' field in Room API response".into()),
        }
    }
}

pub async fn get_stream_url(room_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let douyu = DouYu::new(room_id).await?;
    let url = douyu.get_real_url().await?;
    Ok(url) // 直接返回实际的流地址
}
