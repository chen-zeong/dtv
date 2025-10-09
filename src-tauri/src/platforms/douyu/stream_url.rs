use deno_core::{JsRuntime, RuntimeOptions};
use reqwest::{Client, header::{HeaderMap, HeaderValue}, redirect::Policy};
use md5::Digest;
use regex::Regex;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Debug)]
struct RoomInfoData {
    room_status: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RoomInfoResponse {
    error: i32,
    data: Option<RoomInfoData>,
}

struct DouYu {
    did: String,
    rid: String,
    client: Client,
}

impl DouYu {
    async fn new(rid: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 迁移到 reqwest：禁用系统代理、限制重定向、设置默认 UA/语言等头部
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "User-Agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"),
        );
        default_headers.insert(
            "Accept-Language",
            HeaderValue::from_static("zh-CN,zh;q=0.9"),
        );
        let client = Client::builder()
            .redirect(Policy::limited(10))
            .no_proxy()
            .default_headers(default_headers)
            .build()?;

        // 生成动态 did（与搜索接口一致），避免某些房间页面返回不同脚本结构
        let mut hasher = md5::Md5::new();
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_nanos()
                .to_string(),
        );
        let did = format!("{:x}", hasher.finalize());

        Ok(Self {
            did,
            rid: rid.to_string(),
            client,
        })
    }

    fn md5(data: &str) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    async fn execute_js_functions(&self, func_ub9: &str, rid: &str, did: &str, t10: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut runtime = JsRuntime::new(RuntimeOptions::default());
        
        // 执行第一个函数
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
        
        let rb = Self::md5(&format!("{}{}{}{}", rid, did, t10, v));
        
        // 构造签名函数
        let func_sign = res.replace("return rt;})", "return rt;}");
        let func_sign = func_sign.replace("(function (", "function sign(");
        let func_sign = func_sign.replace("CryptoJS.MD5(cb).toString()", &format!("\"{}\"", rb));
        
        let func_sign_static = String::from(func_sign);
        runtime.execute_script("[douyu]", deno_core::FastString::from(func_sign_static))?;
        
        let sign_call = format!("sign(\"{}\", \"{}\", \"{}\");", rid, did, t10);
        
        let sign_call_static = String::from(sign_call);
        let js_params = runtime.execute_script("[douyu]", deno_core::FastString::from(sign_call_static))?;
        
        // 获取签名结果
        let params = {
            let scope = &mut runtime.handle_scope();
            let result = js_params.open(scope);
            result.to_rust_string_lossy(scope)
        };
        
        Ok(params)
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

        // 获取PC网页内容（保持与 isahc 等价的头部）
        let page_url = format!("https://www.douyu.com/{}", self.rid);
        let text = self.client
            .get(page_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8")
            // .header("Accept-Encoding", "gzip, deflate, br") // 交给 reqwest 自动处理
            .header("Referer", format!("https://www.douyu.com/{}", self.rid))
            .header("Upgrade-Insecure-Requests", "1")
            .header("Host", "www.douyu.com")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Cookie", format!("dy_did={}; acf_did={}", self.did, self.did))
            .send()
            .await?
            .text()
            .await?;

        // 提取JS函数（主正则）
        let re = Regex::new(r"(vdwdae325w_64we[\s\S]*function ub98484234[\s\S]*?)function")?;
        let result_opt = re
            .captures(&text)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));

        // 若主正则未命中，尝试备用正则（页面结构差异时有用）
        let result = if let Some(res) = result_opt {
            res
        } else {
            let re_alt = Regex::new(r"(function\s+ub98484234[\s\S]*?)function")?;
            match re_alt.captures(&text).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string())) {
                Some(res_alt) => res_alt,
                None => {
                    // 打印部分页面内容，便于定位问题
                    let sample = &text.chars().take(800).collect::<String>();
                    return Err(format!("Cannot find js function; page sample: {}", sample).into());
                }
            }
        };

        let re_eval = Regex::new(r"eval.*?;\}")?;
        let func_ub9 = re_eval.replace_all(&result, "strc;}");

        let t10 = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();

        let mut params = self.execute_js_functions(&func_ub9, &self.rid, &self.did, &t10).await?;
        params.push_str(&format!("&cdn={}&rate={}", cdn, rate));

        // 获取真实URL
        let url = format!("https://www.douyu.com/lapi/live/getH5Play/{}", self.rid);
        let json = self.client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "https://www.douyu.com")
            .header("Referer", format!("https://www.douyu.com/{}", self.rid))
            .header("Cookie", format!("dy_did={}; acf_did={}", self.did, self.did))
            .body(params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let data = json["data"].as_object().ok_or("No data field in response")?;
        let rtmp_url = data["rtmp_url"].as_str().ok_or("No rtmp_url field")?;
        let rtmp_live = data["rtmp_live"].as_str().ok_or("No rtmp_live field")?;

        let final_url = format!("{}/{}", rtmp_url, rtmp_live);

        Ok(final_url)
    }

    pub async fn get_real_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.get_pc_js("ws-h5", 0).await
    }

    pub async fn get_real_url_with_quality(&self, rate: i32) -> Result<String, Box<dyn std::error::Error>> {
        self.get_pc_js("ws-h5", rate).await
    }

    async fn check_room_status(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let room_api_url = format!("http://open.douyucdn.cn/api/RoomApi/room/{}", self.rid);

        let response = self.client
            .get(room_api_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Room API request failed with status: {}", response.status()).into());
        }

        let room_info_response: RoomInfoResponse = response.json().await?;
        println!(
            "[Douyu Stream URL] Room API response for {}: {:?}",
            self.rid, room_info_response
        );

        if room_info_response.error != 0 {
            return Err(format!("Room API returned error code: {}", room_info_response.error).into());
        }

        match room_info_response.data {
            Some(data) => {
                match data.room_status.as_deref() {
                    Some("1") => Ok(true),  // Live
                    Some("2") => Ok(false), // Not live
                    Some(_status) => {
                        Ok(false)
                    }
                    None => {
                        Ok(false)
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
    Ok(url)
}

pub async fn get_stream_url_with_quality(room_id: &str, quality: &str) -> Result<String, Box<dyn std::error::Error>> {
    let rate = match quality {
        "原画" | "origin" => 0,
        "高清" | "high" => 4,
        "标清" | "standard" => 3,
        _ => 0, // 默认原画
    };
    
    let douyu = DouYu::new(room_id).await?;
    let url = douyu.get_real_url_with_quality(rate).await?;
    Ok(url)
}
