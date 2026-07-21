use std::env;
use std::fs;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use serde_json::{json, Value};

static ENV_LOADED: OnceLock<bool> = OnceLock::new();

fn load_env_contents(contents: &str) -> bool {
    let mut loaded = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            if env::var(key).is_err() {
                env::set_var(key, value);
                loaded = true;
            }
        }
    }

    loaded
}

fn load_env_file() {
    ENV_LOADED.get_or_init(|| {
        let mut loaded = false;

        if let Ok(home) = env::var("HOME") {
            let path = format!("{}/Documents/gcp-c/.env", home);
            if let Ok(contents) = fs::read_to_string(&path) {
                loaded = load_env_contents(&contents) || loaded;
            }
        }

        if !loaded {
            if let Ok(contents) = fs::read_to_string(".env") {
                loaded = load_env_contents(&contents) || loaded;
            }
        }

        loaded
    });
}

fn env_or_file(key: &str) -> Option<String> {
    if let Ok(value) = env::var(key) {
        return Some(value);
    }
    load_env_file();
    env::var(key).ok()
}

fn get_project_id() -> Result<String, String> {
    env_or_file("GOOGLE_PROJECT_ID").ok_or_else(|| "GOOGLE_PROJECT_ID not set".to_string())
}

fn get_location() -> Result<String, String> {
    env_or_file("GOOGLE_LOCATION").ok_or_else(|| "GOOGLE_LOCATION not set".to_string())
}

fn get_engine_id() -> Result<String, String> {
    env_or_file("GOOGLE_ENGINE_ID").ok_or_else(|| "GOOGLE_ENGINE_ID not set".to_string())
}

const TOKEN_TTL: Duration = Duration::from_secs(50 * 60);
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static TOKEN_CACHE: OnceLock<Mutex<Option<CachedToken>>> = OnceLock::new();

#[derive(Clone)]
struct CachedToken {
    value: String,
    expires_at: Instant,
}

fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(reqwest::Client::new)
}

fn token_cache() -> &'static Mutex<Option<CachedToken>> {
    TOKEN_CACHE.get_or_init(|| Mutex::new(None))
}

fn get_access_token() -> Result<String, String> {
    let cache = token_cache();
    if let Ok(guard) = cache.lock() {
        if let Some(token) = guard.as_ref() {
            if token.expires_at > Instant::now() {
                crate::debug!("[FODORIAN DEBUG] Token reutilizado de cache (TTL restante)");
                return Ok(token.value.clone());
            }
        }
    }

    crate::debug!("[FODORIAN DEBUG] Solicitando token con gcloud auth print-access-token...");
    let auth = Command::new("gcloud")
        .args(["auth", "print-access-token"])
        .output()
        .map_err(|e| format!("Error gcloud: {}", e))?;

    if !auth.status.success() {
        let stderr = String::from_utf8_lossy(&auth.stderr);
        crate::debug!("[FODORIAN DEBUG] gcloud auth FAILED: {}", stderr);
        return Err(format!("gcloud auth failed: {}", stderr));
    }

    let token = String::from_utf8_lossy(&auth.stdout).trim().to_string();
    if token.is_empty() {
        crate::debug!("[FODORIAN DEBUG] gcloud devolvio token vacio");
        return Err("gcloud devolvio token vacio".to_string());
    }

    crate::debug!("[FODORIAN DEBUG] Token obtenido ({} caracteres)", token.len());

    if let Ok(mut guard) = cache.lock() {
        *guard = Some(CachedToken {
            value: token.clone(),
            expires_at: Instant::now() + TOKEN_TTL,
        });
    }

    Ok(token)
}

#[tauri::command]
pub async fn invocar_agente_multimodal(
    prompt: String,
    agente: String,
    history: Vec<Value>
) -> Result<String, String> {
    let client = get_http_client();
    let token = get_access_token()?;
    let location = get_location()?;
    let engine_id = get_engine_id()?;

    let url = format!("https://{}-aiplatform.googleapis.com/v1beta1/{}:query", location, engine_id);

    let payload = json!({
        "input": {
            "user_prompt": prompt,
            "task_type": agente,
            "history": history
        }
    });

    crate::debug!("[FODORIAN DEBUG] ===========================================");
    crate::debug!("[FODORIAN DEBUG] Enviando a Google AI:");
    crate::debug!("[FODORIAN DEBUG] URL: {}", url);
    crate::debug!("[FODORIAN DEBUG] Agente: {}", agente);
    crate::debug!("[FODORIAN DEBUG] Prompt ({} chars): {}", prompt.len(), &prompt[..prompt.len().min(200)]);
    crate::debug!("[FODORIAN DEBUG] History entries: {}", history.len());
    crate::debug!("[FODORIAN DEBUG] ===========================================");

    let response = client.post(&url)
        .bearer_auth(&token)
        .header("x-goog-user-project", &get_project_id()?)
        .json(&payload)
        .send().await.map_err(|e| {
            crate::debug!("[FODORIAN DEBUG] ERROR HTTP al enviar: {}", e);
            e.to_string()
        })?;

    crate::debug!("[FODORIAN DEBUG] Status HTTP: {}", response.status());

    let response_text = response.text().await.map_err(|e| {
        crate::debug!("[FODORIAN DEBUG] ERROR al leer body: {}", e);
        e.to_string()
    })?;

    crate::debug!("[FODORIAN DEBUG] Respuesta cruda ({} chars): {}", response_text.len(), &response_text[..response_text.len().min(500)]);

    let j: Value = serde_json::from_str(&response_text).map_err(|_| {
        crate::debug!("[FODORIAN DEBUG] ERROR parseando JSON de respuesta: {}", &response_text[..response_text.len().min(200)]);
        "Error interno al procesar la respuesta de Google AI.".to_string()
    })?;

    if let Some(output) = j.get("output") {
        let respuesta = output.get("response").and_then(|r| r.as_str()).unwrap_or("Sin respuesta.");
        crate::debug!("[FODORIAN DEBUG] Respuesta extraida ({} chars)", respuesta.len());
        if respuesta.contains("<FODORIAN_EXEC>") {
            crate::debug!("[FODORIAN DEBUG] Contiene FODORIAN_EXEC!");
        }
        Ok(respuesta.to_string())
    } else {
        crate::debug!("[FODORIAN DEBUG] No se encontro 'output' en la respuesta. Respuesta cruda (200 chars): {}", &response_text[..response_text.len().min(200)]);
        Ok("GOOGLE_ERROR: La respuesta no contiene el campo 'output' esperado.".to_string())
    }
}
