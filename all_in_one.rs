// 单文件版本：将 macros.rs、dml.rs、huya.rs、main.rs 合并到一个文件中

pub mod macros {
    macro_rules! dmlerr {
        () => {
            anyhow::anyhow!("huya parse error")
        };
    }
    pub(crate) use dmlerr;
}

pub mod dml {
    pub struct DMLDanmaku {
        pub time: i64,
        pub text: String,
        pub nick: String,
        pub color: String,
        pub position: i32,
    }
}

pub mod huya {
    use futures::{SinkExt, stream::StreamExt};
    use log::info;
    use regex::Regex;
    use reqwest::Url;
    use std::time::Duration;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use tars_stream::prelude::*;
    use tokio::time::sleep;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message::Binary;

    use crate::macros::dmlerr;
    use crate::dml::DMLDanmaku;

    const HEARTBEAT: &'static [u8] =
        b"\x00\x03\x1d\x00\x00\x69\x00\x00\x00\x69\x10\x03\x2c\x3c\x4c\x56\x08\x6f\x6e\x6c\x69\x6e\x65\x75\x69\x66\x0f\x4f\x6e\x55\x73\x65\x72\x48\x65\x61\x72\x74\x42\x65\x61\x74\x7d\x00\x00\x3c\x08\x00\x01\x06\x04\x74\x52\x65\x71\x1d\x00\x00\x2f\x0a\x0a\x0c\x16\x00\x26\x00\x36\x07\x61\x64\x72\x5f\x77\x61\x70\x46\x00\x0b\x12\x03\xae\xf0\x0f\x22\x03\xae\xf0\x0f\x3c\x42\x6d\x52\x02\x60\x5c\x60\x01\x7c\x82\x00\x0b\xb0\x1f\x9c\xac\x0b\x8c\x98\x0c\xa8\x0c";

    struct HuyaUser {
        _uid: i64,
        _imid: i64,
        name: String,
        _gender: i32,
    }

    struct HuyaDanmaku {
        color: i32,
    }

    impl StructFromTars for HuyaUser {
        fn _decode_from(decoder: &mut TarsDecoder) -> Result<Self, DecodeErr> {
            let uid = decoder.read_int64(0, false, -1)?;
            let imid = decoder.read_int64(1, false, -1)?;
            let name = decoder.read_string(2, false, "".to_string())?;
            let gender = decoder.read_int32(3, false, -1)?;
            Ok(HuyaUser { _uid: uid, _imid: imid, name, _gender: gender })
        }
    }
    impl StructFromTars for HuyaDanmaku {
        fn _decode_from(decoder: &mut TarsDecoder) -> Result<Self, DecodeErr> {
            let color = decoder.read_int32(0, false, 16777215)?;
            Ok(HuyaDanmaku { color })
        }
    }

    pub struct Huya {}

    impl Huya {
        pub fn new() -> Self { Huya {} }

        // 仅用于调试：窥视顶层 cmd 和嵌套 cmd
        fn peek_cmds(&self, data: &[u8]) -> (Option<i32>, Option<i64>) {
            let mut ios = TarsDecoder::from(data);
            let top_cmd = ios.read_int32(0, false, -1).ok();
            let nested_cmd = ios
                .read_bytes(1, false, Default::default())
                .ok()
                .and_then(|b1| {
                    let mut inner = TarsDecoder::from(b1.as_ref());
                    inner.read_int32(1, false, -1).ok().map(|v| v as i64)
                });
            (top_cmd, nested_cmd)
        }

        fn find_uid_in_json(v: &serde_json::Value) -> Option<String> {
            match v {
                serde_json::Value::Object(map) => {
                    for (k, val) in map {
                        let key = k.to_lowercase();
                        if key == "ayyuid" || key == "yyuid" || key == "lp" || key == "uid" {
                            match val {
                                serde_json::Value::String(s) => if !s.is_empty() { return Some(s.clone()); },
                                serde_json::Value::Number(n) => return Some(n.to_string()),
                                _ => {}
                            }
                        }
                        if let Some(found) = Self::find_uid_in_json(val) { return Some(found); }
                    }
                    None
                }
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        if let Some(found) = Self::find_uid_in_json(item) { return Some(found); }
                    }
                    None
                }
                _ => None
            }
        }

        pub async fn get_ws_info(&self, url: &str) -> anyhow::Result<(String, Vec<u8>)> {
            let url = Url::parse(url)?;
            let rid = url.path_segments().ok_or_else(|| dmlerr!())?.last().ok_or_else(|| dmlerr!())?;
            let client = reqwest::Client::new();
            let resp = client
                .get(format!("https://www.huya.com/{}", &rid))
                .header("User-Agent", gen_ua())
                .header("Referer", "https://www.huya.com/")
                .send()
                .await?
                .text()
                .await?;
            // 依次尝试多种来源获取 ayyuid/yyuid/lp，最后回退为 rid
            let ayyuid = {
                // TT_PROFILE_INFO
                let re_prof = Regex::new(r#"var\s+TT_PROFILE_INFO\s*=\s*(\{[\s\S]*?\});"#).unwrap();
                if let Some(cap) = re_prof.captures(&resp) {
                    if let Ok(j) = serde_json::from_str::<serde_json::Value>(&cap[1]) {
                        if let Some(v) = j.pointer("/lp") { v.to_string().replace('"', "") } else { String::new() }
                    } else { String::new() }
                } else { String::new() }
            };
            let ayyuid = if !ayyuid.is_empty() { ayyuid } else {
                // 直接匹配 lp
                let re_lp = Regex::new(r#"\"lp\"\s*:\s*\"?(\d+)\"?"#).unwrap();
                if let Some(cap) = re_lp.captures(&resp) { cap.get(1).unwrap().as_str().to_string() } else { String::new() }
            };
            let mut ayyuid = if !ayyuid.is_empty() { ayyuid } else {
                // 匹配 ayyuid / yyuid
                let re_ayyuid = Regex::new(r#"\"ayyuid\"\s*:\s*\"?(\d+)\"?"#).unwrap();
                let re_yyuid = Regex::new(r#"\"yyuid\"\s*:\s*\"?(\d+)\"?"#).unwrap();
                if let Some(cap) = re_ayyuid.captures(&resp) {
                    cap.get(1).unwrap().as_str().to_string()
                } else if let Some(cap) = re_yyuid.captures(&resp) {
                    cap.get(1).unwrap().as_str().to_string()
                } else { String::new() }
            };
            if ayyuid.is_empty() {
                // 回退：调用 mp.huya.com 获取 profileRoom JSON 并尝试提取 uid/yyuid
                let url_api = format!("https://mp.huya.com/cache.php?m=Live&do=profileRoom&roomid={}", rid);
                if let Ok(text) = client.get(&url_api).header("User-Agent", gen_ua()).send().await?.text().await {
                    if let Ok(j) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(found) = Self::find_uid_in_json(&j) {
                            ayyuid = found;
                        }
                    }
                }
            }
            if ayyuid.is_empty() { ayyuid = rid.to_string(); }

            let mut t = Vec::new();
            t.push(format!("live:{}", ayyuid));
            t.push(format!("chat:{}", ayyuid));
            info!("huya reg data: {:?}", &t);
            let mut oos = TarsEncoder::new();
            oos.write_list(0, &t)?;
            oos.write_string(1, &"".to_owned())?;
            let mut wscmd = TarsEncoder::new();
            wscmd.write_int32(0, 16)?;
            wscmd.write_bytes(1, &oos.to_bytes())?;
            let b = wscmd.to_bytes();
            Ok(("wss://cdnws.api.huya.com".to_owned(), b.as_ref().to_vec()))
        }

        // panic imports are at module scope; do not import inside impl
        pub fn decode_msg(&self, data: &[u8]) -> anyhow::Result<Vec<DMLDanmaku>> {
            let mut ret = Vec::new();
            let mut ios = TarsDecoder::from(data);
            if ios.read_int32(0, false, -1)? == 7 {
                // 顶层cmd=7，读取内部结构并尝试通用解析
                let b1 = ios.read_bytes(1, false, Default::default())?;
                let mut inner = TarsDecoder::from(b1.as_ref());
                let nested = inner.read_int32(1, false, -1).unwrap_or(-1);
                let b2 = inner.read_bytes(2, false, Default::default())?;
                let mut payload = TarsDecoder::from(b2.as_ref());

                // 专门处理 nested=1400（HYMessage）：userInfo@0, content@3, bulletFormat@6(fontColor@0)
                if nested == 1400 {
                    let user = payload
                        .read_struct(0, false, HuyaUser { _uid: -1, _imid: -1, name: "".to_owned(), _gender: 1 })
                        .unwrap_or(HuyaUser { _uid: -1, _imid: -1, name: "".to_owned(), _gender: 1 });
                    let text = payload.read_string(3, false, "".to_owned()).unwrap_or_default();
                    let huya_danmaku = payload
                        .read_struct(6, false, HuyaDanmaku { color: 16777215 })
                        .unwrap_or(HuyaDanmaku { color: 16777215 });
                    if !text.is_empty() {
                        let nick = if !user.name.is_empty() { user.name } else { "匿名".to_string() };
                        let dml_dm = DMLDanmaku {
                            time: 0,
                            text,
                            nick,
                            color: format!(
                                "{:06x}",
                                if huya_danmaku.color <= 0 { 16777215 } else { huya_danmaku.color }
                            ),
                            position: 0,
                        };
                        ret.push(dml_dm);
                    }
                    return Ok(ret);
                }

                // 只保留聊天弹幕：非 1400 的一律忽略
                return Ok(ret);
            }
            Ok(ret)
        }

        pub async fn run(&self, url: &str, dtx: async_channel::Sender<DMLDanmaku>) -> anyhow::Result<()> {
            let (ws, reg_data) = self.get_ws_info(url).await?;
            let (ws_stream, _) = connect_async(&ws).await?;
            let (mut ws_write, mut ws_read) = ws_stream.split();
            ws_write.send(tokio_tungstenite::tungstenite::Message::Binary(reg_data)).await?;
            let hb_task = async {
                while let Ok(_) = ws_write.send(Binary(HEARTBEAT.to_vec())).await {
                    sleep(Duration::from_secs(20)).await;
                }
                Err(anyhow::anyhow!("send heartbeat failed!"))
            };
            let recv_task = async {
                while let Some(m) = ws_read.next().await {
                    let m = m?;
                    let data = m.into_data();
                    let (top, nested) = self.peek_cmds(&data);
                    // 仅在聊天弹幕时打印精简日志
                    if nested == Some(1400) {
                        info!("chat msg len={}, top={:?}", data.len(), top);
                    }
                    // 解析并防止潜在 panic
                    let mut dm = match catch_unwind(AssertUnwindSafe(|| self.decode_msg(&data))) {
                        Ok(Ok(v)) => v,
                        Ok(Err(e)) => { info!("decode failed: {}", e); Vec::new() },
                        Err(_) => { info!("decode panic, skipped"); Vec::new() }
                    };
                    for d in dm.drain(..) { dtx.send(d).await?; }
                }
                anyhow::Ok(())
            };
            tokio::select! {
                it = hb_task => { it?; },
                it = recv_task => { it?; },
            }
            Ok(())
        }
    }

    fn gen_ua() -> String {
        // 简化版 UA，避免引入自定义 utils
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
    }
}

use anyhow::Result;
use huya::Huya;
use dml::DMLDanmaku;
use log::{info, LevelFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    // 读取参数：可输入虎牙房间号或完整URL，例如 333003 或 https://www.huya.com/333003
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法: cargo run -- <房间号或URL>\n示例: cargo run -- 333003");
        std::process::exit(1);
    });
    let url = if arg.starts_with("http") { arg } else { format!("https://www.huya.com/{}", arg) };

    let (tx, rx) = async_channel::bounded::<DMLDanmaku>(1000);

    // 启动接收任务
    tokio::spawn(async move {
        let h = Huya::new();
        if let Err(e) = h.run(&url, tx).await {
            eprintln!("运行出错: {e}");
        }
    });

    // 消费弹幕并打印到控制台
    while let Ok(dm) = rx.recv().await {
        println!("[{}] {}: {} (#{} pos={})", dm.time, dm.nick, dm.text, dm.color, dm.position);
    }

    Ok(())
}