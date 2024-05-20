// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn input_enter(value: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", value)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build()) // 全局快捷键
        .plugin(tauri_plugin_sql::Builder::default().build()) // SQLite数据库
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![input_enter])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
