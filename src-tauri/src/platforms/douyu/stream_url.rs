use deno_core::{JsRuntime, RuntimeOptions};
use md5::Digest;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Client,
};
use serde::Deserialize;
use serde_json::Value;
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

#[derive(Clone, Debug)]
struct DouyuRateVariant {
    name: String,
    rate: i32,
    bit: Option<i32>,
}

#[derive(Clone, Debug)]
struct DouyuStreamResult {
    url: String,
    variants: Vec<DouyuRateVariant>,
    requested_rate: i32,
}

fn value_to_i32(value: &Value) -> Option<i32> {
    match value {
        Value::Number(num) => num.as_i64().map(|n| n as i32),
        Value::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
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

    async fn execute_js_functions(
        &self,
        func_ub9: &str,
        rid: &str,
        did: &str,
        t10: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
        let js_params =
            runtime.execute_script("[douyu]", deno_core::FastString::from(sign_call_static))?;

        // 获取签名结果
        let params = {
            let scope = &mut runtime.handle_scope();
            let result = js_params.open(scope);
            result.to_rust_string_lossy(scope)
        };

        Ok(params)
    }

    async fn get_pc_js(
        &self,
        cdn: &str,
        rate: i32,
    ) -> Result<DouyuStreamResult, Box<dyn std::error::Error>> {
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
            match re_alt
                .captures(&text)
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            {
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

        let mut params = self
            .execute_js_functions(&func_ub9, &self.rid, &self.did, &t10)
            .await?;
        params.push_str(&format!("&cdn={}&rate={}", cdn, rate));

        // 获取真实URL
        let url = format!("https://www.douyu.com/lapi/live/getH5Play/{}", self.rid);
        let json = self
            .client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "https://www.douyu.com")
            .header("Referer", format!("https://www.douyu.com/{}", self.rid))
            .header(
                "Cookie",
                format!("dy_did={}; acf_did={}", self.did, self.did),
            )
            .body(params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let data = json["data"]
            .as_object()
            .ok_or("No data field in response")?;
        let rtmp_url = data["rtmp_url"].as_str().ok_or("No rtmp_url field")?;
        let rtmp_live = data["rtmp_live"].as_str().ok_or("No rtmp_live field")?;

        let final_url = format!("{}/{}", rtmp_url, rtmp_live);

        let variants = data
            .get("multirates")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())?;
                        let rate_value = item.get("rate").and_then(value_to_i32)?;
                        let bit_value = item.get("bit").and_then(value_to_i32);
                        Some(DouyuRateVariant {
                            name,
                            rate: rate_value,
                            bit: bit_value,
                        })
                    })
                    .collect::<Vec<DouyuRateVariant>>()
            })
            .unwrap_or_default();

        if !variants.is_empty() {
            println!(
                "[Douyu Stream URL] Room {} available qualities (requested rate {}): {:?}",
                self.rid, rate, variants
            );
        }

        Ok(DouyuStreamResult {
            url: final_url,
            variants,
            requested_rate: rate,
        })
    }

    pub async fn get_real_url(&self) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.get_pc_js("ws-h5", 0).await?;
        Ok(result.url)
    }

    pub async fn get_real_url_with_quality(
        &self,
        quality: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let base_result = self.get_pc_js("ws-h5", 0).await?;
        let target_rate =
            Self::resolve_rate_for_quality(quality, &base_result.variants).unwrap_or(0);
        println!(
            "[Douyu Stream URL] Requested quality '{}', resolved rate {} (variants: {:?})",
            quality, target_rate, base_result.variants
        );
        if target_rate == 0 || target_rate == base_result.requested_rate {
            return Ok(base_result.url);
        }
        let target_result = self.get_pc_js("ws-h5", target_rate).await?;
        Ok(target_result.url)
    }

    fn resolve_rate_for_quality(quality: &str, variants: &[DouyuRateVariant]) -> Option<i32> {
        if variants.is_empty() {
            return None;
        }

        let trimmed = quality.trim();
        let ascii_lower = trimmed.to_ascii_lowercase();
        let canonical = if trimmed.contains('原') || ascii_lower == "origin" {
            "原画"
        } else if trimmed.contains('高') || ascii_lower == "high" {
            "高清"
        } else if trimmed.contains('标') || ascii_lower == "standard" {
            "标清"
        } else {
            trimmed
        };

        let find_by_keywords = |keywords: &[&str], exclude_zero: bool| -> Option<i32> {
            for keyword in keywords {
                if let Some(item) = variants.iter().find(|v| v.name.contains(keyword)) {
                    if exclude_zero && item.rate == 0 {
                        continue;
                    }
                    return Some(item.rate);
                }
            }
            None
        };

        match canonical {
            "原画" => {
                if let Some(item) = variants.iter().find(|v| v.rate == 0) {
                    return Some(item.rate);
                }
                if let Some(rate) = find_by_keywords(&["原画", "蓝光8M", "蓝光"], false) {
                    return Some(rate);
                }
                variants.iter().map(|v| v.rate).min()
            }
            "高清" => {
                if let Some(item) = variants.iter().find(|v| v.rate == 4) {
                    return Some(item.rate);
                }
                if let Some(rate) = find_by_keywords(&["蓝光", "蓝光4M"], false) {
                    return Some(rate);
                }
                if let Some(rate) = find_by_keywords(&["超清"], true) {
                    return Some(rate);
                }
                if let Some(rate) = find_by_keywords(&["高清"], true) {
                    return Some(rate);
                }
                variants
                    .iter()
                    .filter(|v| v.rate != 0)
                    .max_by_key(|v| v.bit.unwrap_or(0))
                    .map(|v| v.rate)
                    .or_else(|| {
                        variants
                            .iter()
                            .filter(|v| v.rate != 0)
                            .max_by_key(|v| v.rate)
                            .map(|v| v.rate)
                    })
            }
            "标清" => {
                if let Some(item) = variants.iter().find(|v| v.rate == 3) {
                    return Some(item.rate);
                }
                if let Some(rate) = find_by_keywords(&["超清"], true) {
                    return Some(rate);
                }
                if let Some(rate) = find_by_keywords(&["流畅"], true) {
                    return Some(rate);
                }
                if let Some(rate) = find_by_keywords(&["标清"], true) {
                    return Some(rate);
                }
                if let Some(rate) = find_by_keywords(&["普清"], true) {
                    return Some(rate);
                }
                variants
                    .iter()
                    .filter(|v| v.rate != 0)
                    .min_by_key(|v| v.bit.unwrap_or(i32::MAX))
                    .map(|v| v.rate)
                    .or_else(|| {
                        variants
                            .iter()
                            .filter(|v| v.rate != 0)
                            .min_by_key(|v| v.rate)
                            .map(|v| v.rate)
                    })
            }
            _ => {
                if let Some(rate) = find_by_keywords(&[canonical], false) {
                    return Some(rate);
                }
                None
            }
        }
    }

    async fn check_room_status(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let room_api_url = format!("http://open.douyucdn.cn/api/RoomApi/room/{}", self.rid);

        let response = self.client
            .get(room_api_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;

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
                    Some(_status) => Ok(false),
                    None => Ok(false),
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

pub async fn get_stream_url_with_quality(
    room_id: &str,
    quality: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let douyu = DouYu::new(room_id).await?;
    let url = douyu.get_real_url_with_quality(quality).await?;
    Ok(url)
}
