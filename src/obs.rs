use std::sync::mpsc;

use anyhow::{Context, bail};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use sha2::{Digest, Sha256};
use tungstenite::connect;

use crate::configure::ObsIntegration;

/// obs-websocket v5 opcodes
const OP_HELLO: u64 = 0;
const OP_IDENTIFY: u64 = 1;
const OP_IDENTIFIED: u64 = 2;
const OP_REQUEST: u64 = 6;
const OP_REQUEST_RESPONSE: u64 = 7;

/// Commands the main thread can send to the OBS thread.
pub enum ObsCmd {
    /// Check if OBS is recording and start it if not.
    EnsureRecording,
}

type Ws = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

fn compute_auth(password: &str, salt: &str, challenge: &str) -> String {
    let mut h = Sha256::new();
    h.update(password.as_bytes());
    h.update(salt.as_bytes());
    let secret = B64.encode(h.finalize());

    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    h.update(challenge.as_bytes());
    B64.encode(h.finalize())
}

fn read_msg(ws: &mut Ws) -> anyhow::Result<serde_json::Value> {
    loop {
        let msg = ws.read()?;
        if let tungstenite::Message::Text(text) = msg {
            return Ok(serde_json::from_str(&text)?);
        }
    }
}

fn send_msg(ws: &mut Ws, value: serde_json::Value) -> anyhow::Result<()> {
    ws.send(tungstenite::Message::Text(value.to_string().into()))?;
    Ok(())
}

fn handshake(config: &ObsIntegration) -> anyhow::Result<Ws> {
    let url = format!("ws://{}:{}", config.host(), config.port());
    let (mut ws, _) = connect(&url)?;

    // Read Hello (op 0)
    let hello = read_msg(&mut ws).context("reading Hello")?;
    if hello["op"].as_u64() != Some(OP_HELLO) {
        bail!("expected Hello, got: {hello}");
    }

    // Build Identify (op 1)
    let mut identify_data = serde_json::json!({ "rpcVersion": 1 });
    let auth_obj = &hello["d"]["authentication"];
    if !auth_obj.is_null() {
        let salt = auth_obj["salt"].as_str().context("missing salt")?;
        let challenge = auth_obj["challenge"]
            .as_str()
            .context("missing challenge")?;
        let password = config.password().context(
            "OBS requires a password but none configured — set [obs] password in config.toml",
        )?;
        identify_data["authentication"] =
            serde_json::Value::String(compute_auth(password, salt, challenge));
    }

    send_msg(
        &mut ws,
        serde_json::json!({ "op": OP_IDENTIFY, "d": identify_data }),
    )
    .context("sending Identify")?;

    // Read Identified (op 2)
    let identified = read_msg(&mut ws).context("reading Identified")?;
    if identified["op"].as_u64() != Some(OP_IDENTIFIED) {
        bail!("expected Identified, got: {identified}");
    }

    Ok(ws)
}

fn ensure_recording(ws: &mut Ws) -> anyhow::Result<()> {
    // GetRecordStatus
    send_msg(
        ws,
        serde_json::json!({
            "op": OP_REQUEST,
            "d": { "requestType": "GetRecordStatus", "requestId": "get-record-status" }
        }),
    )
    .context("sending GetRecordStatus")?;

    let resp = read_msg(ws).context("reading GetRecordStatus response")?;
    if resp["op"].as_u64() != Some(OP_REQUEST_RESPONSE) {
        bail!("expected RequestResponse, got: {resp}");
    }

    let output_active = resp["d"]["responseData"]["outputActive"]
        .as_bool()
        .unwrap_or(false);

    if output_active {
        log::info!("OBS already recording");
        return Ok(());
    }

    // StartRecord
    log::info!("OBS not recording — starting");
    send_msg(
        ws,
        serde_json::json!({
            "op": OP_REQUEST,
            "d": { "requestType": "StartRecord", "requestId": "start-record" }
        }),
    )
    .context("sending StartRecord")?;

    let resp = read_msg(ws).context("reading StartRecord response")?;
    if resp["d"]["requestStatus"]["result"]
        .as_bool()
        .unwrap_or(false)
    {
        log::info!("OBS recording started");
    } else {
        let comment = resp["d"]["requestStatus"]["comment"]
            .as_str()
            .unwrap_or("unknown error");
        log::warn!("OBS StartRecord failed: {comment}");
    }

    Ok(())
}

fn obs_thread(config: ObsIntegration, rx: mpsc::Receiver<ObsCmd>) {
    let mut ws: Option<Ws> = None;

    for cmd in rx {
        match cmd {
            ObsCmd::EnsureRecording => {
                // Reconnect if the connection was dropped or not yet established
                if ws.is_none() {
                    match handshake(&config) {
                        Ok(conn) => {
                            log::info!("OBS WebSocket connected");
                            ws = Some(conn);
                        }
                        Err(e) => {
                            log::warn!("OBS connect failed: {e:#}");
                            continue;
                        }
                    }
                }

                if let Some(conn) = ws.as_mut() {
                    if let Err(e) = ensure_recording(conn) {
                        log::warn!("OBS ensure_recording failed: {e:#}");
                        ws = None; // drop broken connection, reconnect next time
                    }
                }
            }
        }
    }

    // Sender dropped — close gracefully
    if let Some(mut conn) = ws {
        let _ = conn.close(None);
    }
}

/// Spawn the OBS worker thread. Returns a sender to control it.
/// The thread exits automatically when the sender is dropped.
pub fn spawn(config: ObsIntegration) -> mpsc::Sender<ObsCmd> {
    log::info!("Starting obs thread");
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("obs-worker".into())
        .spawn(move || obs_thread(config, rx))
        .expect("failed to spawn obs thread");
    tx
}
