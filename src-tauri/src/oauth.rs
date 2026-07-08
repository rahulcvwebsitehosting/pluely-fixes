use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const OPENAI_AUTH_URL: &str = "https://auth.openai.com/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth.openai.com/api/oauth/token";
const OAUTH_TIMEOUT_SECS: u64 = 120;

/// Shared state for in-progress OAuth flows.
pub struct OAuthFlowState {
    inner: Arc<Mutex<HashMap<String, OAuthFlowEntry>>>,
}

impl Default for OAuthFlowState {
    fn default() -> Self {
        Self {
            inner: Arc::default(),
        }
    }
}

struct OAuthFlowEntry {
    code_verifier: String,
    state: String,
    started_at: Instant,
    result: Option<OAuthResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

fn random_string(len: usize) -> String {
    let bytes: Vec<u8> = (0..len).map(|_| rand::thread_rng().gen()).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn base64_url_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

fn pkce_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    base64_url_encode(&hash)
}

fn get_secure_storage_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_data_dir.join("openai_oauth.json"))
}

fn save_oauth_token(app: &AppHandle, token: &OAuthResult) -> Result<(), String> {
    let path = get_secure_storage_path(app)?;
    let json = serde_json::to_string_pretty(token)
        .map_err(|e| format!("Failed to serialize token: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write token: {}", e))?;
    Ok(())
}

fn read_oauth_token(app: &AppHandle) -> Result<Option<OAuthResult>, String> {
    let path = get_secure_storage_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let json =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read token: {}", e))?;
    let token: OAuthResult =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse token: {}", e))?;
    Ok(Some(token))
}

fn exchange_code_for_token(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<OAuthResult, String> {
    let client = reqwest::blocking::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", "openai"),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(OPENAI_TOKEN_URL)
        .form(&params)
        .send()
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .map_err(|e| format!("Failed to read token response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "Token exchange failed ({}): {}",
            status, body
        ));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        #[allow(dead_code)]
        token_type: Option<String>,
        refresh_token: Option<String>,
        expires_in: Option<u64>,
    }

    let token_resp: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(OAuthResult {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_in: token_resp.expires_in,
    })
}

fn handle_callback_request(mut stream: TcpStream) {
    let body = "<html><body><h1>Pluely — OAuth Complete</h1>\
         <p>You can close this window and return to Pluely.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

/// Start an OpenAI OAuth authorization flow.
/// Returns the authorization URL that the frontend should open in the browser.
#[tauri::command]
pub fn start_openai_oauth(
    app: AppHandle,
    state_handle: tauri::State<'_, OAuthFlowState>,
) -> Result<String, String> {
    let code_verifier = random_string(64);
    let code_challenge = pkce_challenge(&code_verifier);
    let state = random_string(32);

    // Find a free port
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("Failed to bind: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get port: {}", e))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = format!(
        "{}?response_type=code&client_id=openai&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256",
        OPENAI_AUTH_URL,
        urlencoding(&redirect_uri),
        urlencoding(&state),
        urlencoding(&code_challenge)
    );

    // Store the flow entry
    {
        let mut flows = state_handle
            .inner
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        flows.insert(
            state.clone(),
            OAuthFlowEntry {
                code_verifier,
                state: state.clone(),
                started_at: Instant::now(),
                result: None,
            },
        );
    }

    // Spawn the callback listener on a background thread
    let flow_state = state_handle.inner.clone();
    let app_clone = app.clone();
    let captured_state = state.clone();

    std::thread::spawn(move || {
        listener
            .set_read_timeout(Some(Duration::from_secs(OAUTH_TIMEOUT_SECS)))
            .ok();

        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]);

            // Parse the HTTP GET request for the callback URL
            let (code, state_recv) = parse_callback_request(&request);

            if let (Some(code), Some(state_recv)) = (code, state_recv) {
                if state_recv == captured_state {
                    // Exchange the code for a token
                    let result = exchange_code_for_token(&code, {
                        let flows = flow_state.lock().unwrap();
                        flows
                            .get(&captured_state)
                            .map(|e| e.code_verifier.clone())
                            .unwrap_or_default()
                    }, &redirect_uri);

                    match result {
                        Ok(token) => {
                            let _ = save_oauth_token(&app_clone, &token);

                            let mut flows = flow_state.lock().unwrap();
                            if let Some(entry) = flows.get_mut(&captured_state) {
                                entry.result = Some(token);
                            }
                        }
                        Err(e) => {
                            eprintln!("OAuth token exchange failed: {}", e);
                        }
                    }

                    handle_callback_request(stream);
                    return;
                } else {
                    eprintln!("OAuth state mismatch: received {} expected {}", state_recv, captured_state);
                }
            }
        }

        // If we reach here, the callback timed out or was invalid
        let mut flows = flow_state.lock().unwrap();
        flows.remove(&captured_state);
    });

    Ok(auth_url)
}

/// Check the status of an in-progress OAuth flow.
/// Returns the access token if the flow completed successfully, or null if still pending.
#[tauri::command]
pub fn poll_openai_oauth(
    app: AppHandle,
    state_handle: tauri::State<'_, OAuthFlowState>,
    state: String,
) -> Result<Option<OAuthResult>, String> {
    // First, check in-memory flow state
    {
        let mut flows = state_handle
            .inner
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(entry) = flows.get(&state) {
            if entry.result.is_some() {
                let result = entry.result.take();
                flows.remove(&state);
                return Ok(result);
            }

            // Check timeout
            if entry.started_at.elapsed() > Duration::from_secs(OAUTH_TIMEOUT_SECS) {
                flows.remove(&state);
                return Err("OAuth flow timed out".to_string());
            }

            return Ok(None); // Still pending
        }
    }

    // If not in memory, try to read from secure storage (completed in a previous session)
    read_oauth_token(&app)
}

/// Retrieve the stored OAuth access token (for use as API key).
#[tauri::command]
pub fn get_openai_oauth_token(app: AppHandle) -> Result<Option<String>, String> {
    let token = read_oauth_token(&app)?;
    Ok(token.map(|t| t.access_token))
}

/// Cancel an in-progress OAuth flow.
#[tauri::command]
pub fn cancel_openai_oauth(
    state_handle: tauri::State<'_, OAuthFlowState>,
    state: String,
) -> Result<(), String> {
    let mut flows = state_handle
        .inner
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    flows.remove(&state);
    Ok(())
}

/// Refresh the stored OAuth token using the refresh token.
#[tauri::command]
pub fn refresh_openai_token(app: AppHandle) -> Result<String, String> {
    let stored = read_oauth_token(&app)?
        .ok_or_else(|| "No OAuth token found".to_string())?;

    let refresh_token = stored
        .refresh_token
        .ok_or_else(|| "No refresh token available".to_string())?;

    let client = reqwest::blocking::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id", "openai"),
    ];

    let resp = client
        .post(OPENAI_TOKEN_URL)
        .form(&params)
        .send()
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .map_err(|e| format!("Failed to read refresh response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Token refresh failed ({}): {}", status, body));
    }

    #[derive(Deserialize)]
    struct RefreshResponse {
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<u64>,
    }

    let refresh_resp: RefreshResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

    let new_token = OAuthResult {
        access_token: refresh_resp.access_token,
        refresh_token: refresh_resp.refresh_token,
        expires_in: refresh_resp.expires_in,
    };

    save_oauth_token(&app, &new_token)?;

    Ok(new_token.access_token)
}

/// Parses the authorization code and state from an HTTP GET request.
fn parse_callback_request(request: &str) -> (Option<String>, Option<String>) {
    // Look for GET /callback?code=...&state=...
    let start = match request.find("GET /callback?") {
        Some(i) => i + 14,
        None => return (None, None),
    };
    let end = match request[start..].find(' ') {
        Some(i) => start + i,
        None => return (None, None),
    };
    let query = &request[start..end];

    let mut code = None;
    let mut state = None;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts
            .next()
            .map(|v| urlencoding_decode(v))
            .unwrap_or_default();
        match key {
            "code" => code = Some(value),
            "state" => state = Some(value),
            _ => {}
        }
    }

    (code, state)
}

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else {
            result.push(c);
        }
    }
    result
}
