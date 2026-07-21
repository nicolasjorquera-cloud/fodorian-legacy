mod capture;
mod gcp;
mod sandbox;

macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            eprintln!($($arg)*);
        }
    };
}
pub(crate) use debug;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            gcp::invocar_agente_multimodal,
            sandbox::ejecutar_comando_sandbox,
            capture::capturar_screenshot_seleccion
        ])
        .run(tauri::generate_context!())
        .expect("Fodorian failed to initialize Tauri runtime");
}
