// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn info() -> Result<String, String> {
    let client = nightd_client::Client::new().map_err(|e| e.to_string())?;
    let info = client.info().await.map_err(|e| e.to_string())?;
    Ok(format!("Status: {}", info.status))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
