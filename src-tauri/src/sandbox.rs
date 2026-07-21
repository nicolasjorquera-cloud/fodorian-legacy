use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    "$(", "`",
];

pub fn validate_exec_command(raw: &str) -> Result<(), String> {
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

    for segment in command.split(&[';', '\n', '|'][..]) {
        for piece in segment.split("&&") {
            for part in piece.split("||") {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let executable = trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or_default();

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

#[tauri::command]
pub fn ejecutar_comando_sandbox(comando: String) -> Result<String, String> {
    crate::debug!("[FODORIAN DEBUG] ===========================================");
    crate::debug!("[FODORIAN DEBUG] SOLICITUD DE EXEC RECIBIDA");
    crate::debug!("[FODORIAN DEBUG] Comando: {}", comando);
    crate::debug!("[FODORIAN DEBUG] ===========================================");

    validate_exec_command(&comando).map_err(|e| {
        crate::debug!("[FODORIAN DEBUG] Comando RECHAZADO por validacion: {}", e);
        e
    })?;

    crate::debug!("[FODORIAN DEBUG] Comando paso validacion.");

    let podman_check = Command::new("podman")
        .arg("--version")
        .output()
        .map_err(|e| {
            crate::debug!("[FODORIAN DEBUG] No se encontro podman: {}", e);
            e.to_string()
        })?;

    if !podman_check.status.success() {
        crate::debug!("[FODORIAN DEBUG] podman --version FAILED. stdout={} stderr={}",
            String::from_utf8_lossy(&podman_check.stdout).trim(),
            String::from_utf8_lossy(&podman_check.stderr).trim());
        return Err("Podman no esta disponible. Instala/configura podman antes de aprobar EXEC.".to_string());
    }

    crate::debug!("[FODORIAN DEBUG] Podman detectado: {}",
        String::from_utf8_lossy(&podman_check.stdout).trim());

    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();

    let workspace_dir = format!("/tmp/fodorian_exec_{}", unique_id);
    crate::debug!("[FODORIAN DEBUG] Creando workspace: {}", workspace_dir);
    std::fs::create_dir_all(&workspace_dir).map_err(|e| e.to_string())?;

    crate::debug!("[FODORIAN DEBUG] Creando contenedor efimero...");
    crate::debug!("[FODORIAN DEBUG] Imagen: {}", EXEC_IMAGE);

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
        .map_err(|e| {
            crate::debug!("[FODORIAN DEBUG] ERROR ejecutando podman: {}", e);
            format!("Error ejecutando podman: {}", e)
        })?;

    let exit_code = output.status.code().unwrap_or(-1);
    crate::debug!("[FODORIAN DEBUG] Contenedor finalizo. Exit code: {}", exit_code);

    let _ = std::fs::remove_dir_all(&workspace_dir);
    crate::debug!("[FODORIAN DEBUG] Workspace eliminado: {}", workspace_dir);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    crate::debug!("[FODORIAN DEBUG] STDOUT ({} bytes): {}", stdout.len(), &stdout[..stdout.len().min(500)]);
    crate::debug!("[FODORIAN DEBUG] STDERR ({} bytes): {}", stderr.len(), &stderr[..stderr.len().min(500)]);

    if output.status.success() {
        if stdout.trim().is_empty() {
            Ok("Comando ejecutado en contenedor efimero (sin salida).".to_string())
        } else {
            Ok(stdout)
        }
    } else if exit_code == 124 {
        Err("Timeout: el comando excedio 20 segundos en contenedor efimero.".to_string())
    } else {
        crate::debug!("[FODORIAN DEBUG] Contenedor fallo. STDERR: {}", &stderr[..stderr.len().min(500)]);
        let err = if stderr.trim().is_empty() {
            "Error de ejecucion en contenedor efimero sin detalles.".to_string()
        } else {
            "Error en el contenedor sandbox. Revisa el comando.".to_string()
        };
        Err(err)
    }
}
