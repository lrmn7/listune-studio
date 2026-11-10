//! AI endpoint for agents. Listune listens on
//! 127.0.0.1 (a random port) and gives each agent session `LISTUNE_AI_URL` and its own
//! `LISTUNE_AI_TOKEN`. The token is tied to that session's chat.
//!
//!   POST $LISTUNE_AI_URL/video          { "title", "kind"?, "styleId"?, "lengthSec"?, "notes"? }
//!   GET  $LISTUNE_AI_URL/usage

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Requests are small JSON; anything bigger is refused.
const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 64 * 1024;

/// The running endpoint: its port and the token of each agent session.
pub struct AiProxy {
    pub port: u16,
    tokens: Mutex<HashMap<String, String>>,
}

impl AiProxy {
    /// A new token for `chat_id`'s session (any older one of the chat stops working), as the env
    /// vars the agent process gets.
    pub fn session_env(&self, chat_id: &str) -> Vec<(String, String)> {
        let token = format!("lst_{}", uuid::Uuid::new_v4().simple());
        let mut tokens = self.tokens.lock().unwrap_or_else(|e| e.into_inner());
        tokens.retain(|_, c| c != chat_id);
        tokens.insert(token.clone(), chat_id.to_string());
        vec![
            ("LISTUNE_AI_URL".into(), format!("http://127.0.0.1:{}", self.port)),
            ("LISTUNE_AI_TOKEN".into(), token),
        ]
    }

    fn chat_for(&self, token: &str) -> Option<String> {
        self.tokens.lock().unwrap_or_else(|e| e.into_inner()).get(token).cloned()
    }
}

/// Hands an agent's request to the app: `(chat, name, context)` with name `pick_topic` (start a video).
/// The app asks the user; the agent only asks.
pub type Actor = Arc<dyn Fn(&str, &str, Value) + Send + Sync>;

/// `POST /video`: what the agent sends, checked.
pub fn parse_action(path: &str, b: &Value) -> Option<Result<(&'static str, Value), String>> {
    let text = |k: &str| b.get(k).and_then(Value::as_str).map(str::trim).filter(|t| !t.is_empty());
    match path.split('?').next().unwrap_or("").trim_end_matches('/') {
        "/video" => Some((|| {
            let title = text("title").ok_or("title is required")?;
            if title.chars().count() > 120 {
                return Err("title must be 120 characters or fewer".into());
            }
            let mut ctx = json!({ "title": title });
            for k in ["kind", "styleId", "notes"] {
                if let Some(v) = text(k) {
                    ctx[k] = json!(v.chars().take(600).collect::<String>());
                }
            }
            if let Some(n) = b.get("lengthSec").and_then(Value::as_f64) {
                ctx["lengthSec"] = json!(n);
            }
            Ok(("pick_topic", ctx))
        })()),
        _ => None,
    }
}

/// Starts listening; `act` takes the agent's actions (/video).
pub async fn start(act: Actor) -> std::io::Result<Arc<AiProxy>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let proxy = Arc::new(AiProxy { port: listener.local_addr()?.port(), tokens: Mutex::default() });
    let p = proxy.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (p, act) = (p.clone(), act.clone());
            tokio::spawn(async move {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(300), serve(stream, &p, &act)).await;
            });
        }
    });
    Ok(proxy)
}

struct Request {
    method: String,
    path: String,
    token: Option<String>,
    body: Vec<u8>,
}

async fn read_request(s: &mut tokio::net::TcpStream) -> Option<Request> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let read = async {
        loop {
            if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buf[..i]).to_string();
                let len = head
                    .lines()
                    .find_map(|l| l.split_once(':').filter(|(k, _)| k.trim().eq_ignore_ascii_case("content-length")).map(|(_, v)| v.trim().parse::<usize>().ok()))
                    .flatten()
                    .unwrap_or(0);
                if len > MAX_BODY {
                    return None;
                }
                while buf.len() < i + 4 + len {
                    let n = s.read(&mut chunk).await.ok()?;
                    if n == 0 {
                        return None;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                let mut first = head.lines().next().unwrap_or("").split(' ');
                let method = first.next().unwrap_or("").to_string();
                let path = first.next().unwrap_or("").to_string();
                let token = head.lines().find_map(|l| {
                    let (k, v) = l.split_once(':')?;
                    k.trim().eq_ignore_ascii_case("authorization").then(|| v.trim().strip_prefix("Bearer ").map(|t| t.trim().to_string()))?
                });
                return Some(Request { method, path, token, body: buf[i + 4..i + 4 + len].to_vec() });
            }
            if buf.len() > MAX_HEAD {
                return None;
            }
            let n = s.read(&mut chunk).await.ok()?;
            if n == 0 {
                return None;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(30), read).await.ok().flatten()
}

async fn respond(s: &mut tokio::net::TcpStream, status: u16, body: &Value) {
    let text = body.to_string();
    let res = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        if status < 400 { "OK" } else { "Error" },
        text.len()
    );
    let _ = s.write_all(res.as_bytes()).await;
    let _ = s.shutdown().await;
}

fn fail(code: &str, message: &str) -> Value {
    json!({ "error": code, "message": message })
}

async fn serve(mut s: tokio::net::TcpStream, proxy: &AiProxy, act: &Actor) {
    let Some(req) = read_request(&mut s).await else {
        return respond(&mut s, 400, &fail("invalid", "Send a request with a JSON body under 64 KB")).await;
    };
    let Some(chat) = req.token.as_deref().and_then(|t| proxy.chat_for(t)) else {
        return respond(&mut s, 401, &fail("unauthorized", "Use the LISTUNE_AI_TOKEN this session was given")).await;
    };
    let clean_path = req.path.split('?').next().unwrap_or("").trim_end_matches('/');
    if clean_path == "/usage" {
        if req.method != "GET" {
            return respond(&mut s, 405, &fail("invalid", "Use GET for /usage")).await;
        }
        return respond(&mut s, 200, &json!({
            "mode": "local",
            "message": "Local mode: all operations run locally on this computer",
        })).await;
    }
    if req.method == "POST" {
        if let Ok(body @ Value::Object(_)) = serde_json::from_slice::<Value>(&req.body) {
            match parse_action(&req.path, &body) {
                Some(Ok((name, context))) => {
                    act(&chat, name, context);
                    let message = "Listune is starting this video in its own chat. End your turn now: the video is made there.";
                    return respond(&mut s, 200, &json!({ "ok": true, "message": message })).await;
                }
                Some(Err(m)) => return respond(&mut s, 400, &fail("invalid", &m)).await,
                None => {}
            }
        }
    }
    respond(&mut s, 400, &fail("disabled", "Cloud AI generation is disabled. Use local tools or workspace assets.")).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_are_parsed() {
        let (name, ctx) = parse_action("/video", &json!({ "title": "  Moon  " })).unwrap().unwrap();
        assert_eq!((name, ctx), ("pick_topic", json!({ "title": "Moon" })));
        assert!(parse_action("/video", &json!({})).unwrap().is_err());
        assert!(parse_action("/schedule", &json!({ "items": [] })).is_none());
        assert!(parse_action("/autopilot", &json!({ "name": "Kopi" })).is_none());
        assert!(parse_action("/autopilot-chat", &json!({})).is_none());
    }

    async fn post(port: u16, path: &str, token: Option<&str>, body: &str) -> (u16, Value) {
        request(port, "POST", path, token, body).await
    }

    async fn request(port: u16, method: &str, path: &str, token: Option<&str>, body: &str) -> (u16, Value) {
        let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let auth = token.map(|t| format!("Authorization: Bearer {t}\r\n")).unwrap_or_default();
        let req = format!("{method} {path} HTTP/1.1\r\nHost: x\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len());
        s.write_all(req.as_bytes()).await.unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).await.unwrap();
        let status = out.split(' ').nth(1).unwrap().parse().unwrap();
        let body = out.split("\r\n\r\n").nth(1).unwrap_or("null");
        (status, serde_json::from_str(body).unwrap())
    }

    #[tokio::test]
    async fn only_a_session_token_gets_in_and_actions_work() {
        let acted: Arc<Mutex<Vec<(String, String, Value)>>> = Arc::default();
        let rec_act = acted.clone();
        let act: Actor = Arc::new(move |chat, name, ctx| rec_act.lock().unwrap().push((chat.into(), name.into(), ctx)));
        let proxy = start(act).await.unwrap();
        let env = proxy.session_env("c1");
        assert_eq!(env[0].1, format!("http://127.0.0.1:{}", proxy.port));
        let token = env[1].1.clone();

        assert_eq!(post(proxy.port, "/video", None, "{}").await.0, 401);
        assert_eq!(post(proxy.port, "/video", Some("tai_wrong"), r#"{"title":"x"}"#).await.0, 401);

        // A new session of the chat replaces its token.
        let again = proxy.session_env("c1")[1].1.clone();
        assert_eq!(post(proxy.port, "/video", Some(&token), r#"{"title":"x"}"#).await.0, 401);

        // Usage is a GET with no body
        let (status, body) = request(proxy.port, "GET", "/usage", Some(&again), "").await;
        assert_eq!(status, 200);
        assert_eq!(body["mode"], "local");
        assert_eq!(post(proxy.port, "/usage", Some(&again), "{}").await.0, 405);
        assert_eq!(request(proxy.port, "GET", "/usage", None, "").await.0, 401);

        // The agent asks Listune to start a video: the app gets it, the agent a note to stop.
        let (status, body) = post(proxy.port, "/video", Some(&again), r#"{"title":" What if the Moon vanished ","kind":"explainer","lengthSec":60,"styleId":"cartoon-3d-world"}"#).await;
        assert_eq!(status, 200);
        assert!(body["message"].as_str().unwrap().contains("End your turn"));
        assert_eq!(post(proxy.port, "/video", Some(&again), r#"{"kind":"explainer"}"#).await.0, 400);
        {
            let acted = acted.lock().unwrap();
            assert_eq!(acted.len(), 1);
            assert_eq!((acted[0].0.as_str(), acted[0].1.as_str()), ("c1", "pick_topic"));
            assert_eq!(acted[0].2, json!({ "title": "What if the Moon vanished", "kind": "explainer", "lengthSec": 60.0, "styleId": "cartoon-3d-world" }));
        }

        // Disabled cloud image/voice calls return 400 disabled
        let (status, body) = post(proxy.port, "/image", Some(&again), r#"{"prompt":"a cat"}"#).await;
        assert_eq!(status, 400);
        assert_eq!(body["error"], "disabled");
    }
}
