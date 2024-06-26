// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[tauri::command]
fn input_enter(value: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", value)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            //系统托盘
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&quit]).build()?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(webview_window) = app.get_webview_window("main") {
                            let _ = webview_window.show();
                            let _ = webview_window.set_focus();
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build()) //更新
        .plugin(tauri_plugin_global_shortcut::Builder::new().build()) // 全局快捷键
        .plugin(tauri_plugin_sql::Builder::default().build()) // SQLite数据库
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init()) // HTTP请求
        .invoke_handler(tauri::generate_handler![input_enter])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
