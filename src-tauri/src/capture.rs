use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Serialize;

#[derive(Serialize)]
pub struct ScreenshotCaptureResult {
    pub path: String,
    pub capture_method: String,
    pub ocr_text: Option<String>,
}

fn has_command(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-lc", &format!("command -v {} >/dev/null 2>&1", cmd)])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn capturar_screenshot_seleccion() -> Result<ScreenshotCaptureResult, String> {
    let capture_root = "/tmp/fodorian_captures";
    std::fs::create_dir_all(capture_root).map_err(|e| e.to_string())?;

    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let output_path = format!("{}/capture_{}.png", capture_root, unique_id);

    let mut method = String::new();
    let mut captured = false;

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
