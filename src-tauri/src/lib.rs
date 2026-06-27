use std::env;
use std::fs;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde_json::{json, Value};
use serde::Serialize;

const EXEC_IMAGE: &str = "docker.io/library/alpine:3.20";
const MAX_COMMAND_LEN: usize = 500;
const ALLOWED_EXECUTABLES: &[&str] = &[
    "ls", "cat", "echo", "pwd", "whoami", "id", "date", "head", "tail", "wc", "sort", "uniq",
    "grep", "rg", "sed", "awk", "cut", "tr", "find", "mkdir", "touch", "cp", "mv", "rm",
    "python", "python3", "node", "npm", "pip", "pip3", "sh",
];
const BLOCKED_SNIPPETS: &[&str] = &[
    "podman", "docker", "sudo", "su ", "/proc", "/sys", "/dev", "mount", "umount",
    "iptables", "nft", "systemctl", "service ", "shutdown", "reboot",
];

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
                return Ok(token.value.clone());
            }
        }
    }

    let auth = Command::new("gcloud")
        .args(["auth", "print-access-token"])
        .output()
        .map_err(|e| format!("Error gcloud: {}", e))?;

    if !auth.status.success() {
        let stderr = String::from_utf8_lossy(&auth.stderr);
        return Err(format!("gcloud auth failed: {}", stderr));
    }

    let token = String::from_utf8_lossy(&auth.stdout).trim().to_string();
    if token.is_empty() {
        return Err("gcloud devolvio token vacio".to_string());
    }

    if let Ok(mut guard) = cache.lock() {
        *guard = Some(CachedToken {
            value: token.clone(),
            expires_at: Instant::now() + TOKEN_TTL,
        });
    }

    Ok(token)
}

fn validate_exec_command(raw: &str) -> Result<(), String> {
    let command = raw.trim();
    if command.is_empty() {
        return Err("Comando vacio.".to_string());
    }
    if command.len() > MAX_COMMAND_LEN {
        return Err(format!("Comando demasiado largo (max {} caracteres).", MAX_COMMAND_LEN));
    }

    let lowered = command.to_lowercase();
    for blocked in BLOCKED_SNIPPETS {
        if lowered.contains(blocked) {
            return Err(format!("Comando bloqueado por politica de seguridad: contiene '{}'.", blocked));
        }
    }

    // Permite pipelines/composicion, pero exige que cada segmento inicie con comando permitido.
    for segment in command.split('|') {
        for piece in segment.split("&&") {
            for part in piece.split("||") {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let executable = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_matches(';');

                if executable.is_empty() {
                    continue;
                }

                if !ALLOWED_EXECUTABLES.iter().any(|allowed| allowed == &executable) {
                    return Err(format!(
                        "Ejecutable '{}' no permitido. Usa comandos aprobados por politica local.",
                        executable
                    ));
                }
            }
        }
    }

    Ok(())
}

#[derive(Serialize)]
struct ScreenshotCaptureResult {
    path: String,
    capture_method: String,
    ocr_text: Option<String>,
}

fn has_command(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-lc", &format!("command -v {} >/dev/null 2>&1", cmd)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tauri::command]
fn capturar_screenshot_seleccion() -> Result<ScreenshotCaptureResult, String> {
    let capture_root = "/tmp/fodorian_captures";
    std::fs::create_dir_all(capture_root).map_err(|e| e.to_string())?;

    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let output_path = format!("{}/capture_{}.png", capture_root, unique_id);

    let mut method = String::new();
    let mut captured = false;

    // Wayland recomendado: grim + slurp.
    if !captured && has_command("grim") && has_command("slurp") {
        let status = Command::new("sh")
            .args(["-lc", &format!("grim -g \"$(slurp)\" \"{}\"", output_path)])
            .status()
            .map_err(|e| format!("No se pudo ejecutar grim/slurp: {}", e))?;
        if status.success() {
            captured = true;
            method = "grim+slurp".to_string();
        }
    }

    // Fallback universal en desktop Linux: flameshot.
    if !captured && has_command("flameshot") {
        let status = Command::new("sh")
            .args(["-lc", &format!("flameshot gui -r > \"{}\"", output_path)])
            .status()
            .map_err(|e| format!("No se pudo ejecutar flameshot: {}", e))?;
        if status.success() {
            captured = true;
            method = "flameshot".to_string();
        }
    }

    // Fallback X11: maim.
    if !captured && has_command("maim") {
        let status = Command::new("maim")
            .args(["-s", &output_path])
            .status()
            .map_err(|e| format!("No se pudo ejecutar maim: {}", e))?;
        if status.success() {
            captured = true;
            method = "maim".to_string();
        }
    }

    if !captured {
        return Err(
            "No hay backend de captura disponible. Instala grim+slurp (Wayland), flameshot o maim (X11)."
                .to_string(),
        );
    }

    let metadata = std::fs::metadata(&output_path).map_err(|e| e.to_string())?;
    if metadata.len() == 0 {
        return Err("La captura resulto vacia o fue cancelada.".to_string());
    }

    let ocr_text = if has_command("tesseract") {
        let ocr_out = Command::new("tesseract")
            .args([&output_path, "stdout"])
            .output()
            .map_err(|e| format!("Error ejecutando OCR: {}", e))?;

        if ocr_out.status.success() {
            let text = String::from_utf8_lossy(&ocr_out.stdout).trim().to_string();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(ScreenshotCaptureResult {
        path: output_path,
        capture_method: method,
        ocr_text,
    })
}

#[tauri::command]
async fn invocar_agente_multimodal(
    prompt: String, 
    agente: String,
    history: Vec<Value>
) -> Result<String, String> {
    let client = get_http_client();
    let token = get_access_token()?;
    let location = get_location()?;
    let engine_id = get_engine_id()?;
    
    let url = format!("https://{}-aiplatform.googleapis.com/v1beta1/{}:query", location, engine_id);

    // 2. Payload con Memoria Inyectada
    let payload = json!({
        "input": { 
            "user_prompt": prompt, 
            "task_type": agente, 
            "history": history 
        }
    });

    // 3. Petición a Google Cloud
    let response = client.post(&url)
        .bearer_auth(&token)
        .header("x-goog-user-project", &get_project_id()?)
        .json(&payload)
        .send().await.map_err(|e| e.to_string())?;

    let response_text = response.text().await.map_err(|e| e.to_string())?;
    let j: Value = serde_json::from_str(&response_text).map_err(|_| format!("Error JSON: {}", response_text))?;
    
    // 4. Extracción de Respuesta
    if let Some(output) = j.get("output") {
        let respuesta = output.get("response").and_then(|r| r.as_str()).unwrap_or("Sin respuesta.");
        Ok(respuesta.to_string())
    } else {
        Ok(format!("GOOGLE_ERROR: {}", response_text))
    }
}

#[tauri::command]
fn ejecutar_comando_sandbox(comando: String) -> Result<String, String> {
    validate_exec_command(&comando)?;

    let podman_check = Command::new("podman")
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;

    if !podman_check.status.success() {
        return Err("Podman no esta disponible. Instala/configura podman antes de aprobar EXEC.".to_string());
    }

    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let workspace_dir = format!("/tmp/fodorian_exec_{}", unique_id);
    std::fs::create_dir_all(&workspace_dir).map_err(|e| e.to_string())?;

    // Ejecuta cada comando en un contenedor efimero con limites fuertes.
    let output = Command::new("timeout")
        .args(["--kill-after=5s", "20s", "podman", "run", "--rm"])
        .args(["--network", "none"])
        .args(["--read-only"])
        .args(["--pull", "never"])
        .args(["--cap-drop", "ALL"])
        .args(["--security-opt", "no-new-privileges"])
        .args(["--security-opt", "label=disable"])
        .args(["--pids-limit", "64"])
        .args(["--memory", "256m"])
        .args(["--cpus", "0.5"])
        .args(["--user", "65532:65532"])
        .args(["--tmpfs", "/tmp:rw,noexec,nosuid,size=64m"])
        .args(["-v", &format!("{}:/workspace:rw,z", workspace_dir)])
        .args(["-w", "/workspace"])
        .args([EXEC_IMAGE, "sh", "-lc", &comando])
        .output()
        .map_err(|e| format!("Error ejecutando podman: {}", e))?;

    let _ = std::fs::remove_dir_all(&workspace_dir);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        if stdout.trim().is_empty() {
            Ok("Comando ejecutado en contenedor efimero (sin salida).".to_string())
        } else {
            Ok(stdout)
        }
    } else if output.status.code() == Some(124) {
        Err("Timeout: el comando excedio 20 segundos en contenedor efimero.".to_string())
    } else {
        let err = if stderr.trim().is_empty() {
            "Error de ejecucion en contenedor efimero sin detalles.".to_string()
        } else {
            stderr
        };
        Err(err)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            invocar_agente_multimodal, 
            ejecutar_comando_sandbox,
            capturar_screenshot_seleccion
        ])
        .run(tauri::generate_context!())
        .expect("Kernel Panic");
}
