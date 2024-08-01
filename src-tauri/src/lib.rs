// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod utils;

use once_cell::sync::OnceCell;
use std::path::PathBuf;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

static APP_RESOURCE_DIR: OnceCell<PathBuf> = OnceCell::new(); //app所在目录
static USER_FILES_DIR: OnceCell<PathBuf> = OnceCell::new(); //用户文件所在目录
pub static GPT_SOVITS_MODEL_DIR: OnceCell<PathBuf> = OnceCell::new(); //gpt-sovits模型所在目录
pub static NOVEL_OUTPUT_DIR: OnceCell<PathBuf> = OnceCell::new(); //小说输出目录

#[tauri::command]
fn input_enter(value: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", value)
}

//判断容器是否运行
#[tauri::command]
async fn is_container_running(container_name: &str) -> Result<bool, String> {
    utils::bollard_utils::is_container_running(container_name)
        .await
        .map_err(|e| e.to_string())
}

//开启gpt-sovits容器
#[tauri::command]
async fn start_gpt_sovits() -> Result<String, String> {
    utils::bollard_utils::start_gpt_sovits()
        .await
        .map_err(|e| e.to_string())
}

//保存小说音频
#[tauri::command]
async fn save_novel_audio(audio_data: Vec<u8>,audio_name: &str) -> Result<String, String> {
    let file_path = NOVEL_OUTPUT_DIR.get().unwrap().join(audio_name);
    match utils::default_utils::write_audio_to_file(audio_data, file_path.clone()) {
        Ok(_) => Ok(file_path.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

//打开路径
#[tauri::command]
async fn open_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    utils::default_utils::open_path(path_buf).map_err(|e| e.to_string())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let resources_dir = app.path().resource_dir().unwrap();
            // 初始化数据目录
            APP_RESOURCE_DIR
                .set(utils::default_utils::remove_long_path_prefix(
                    &resources_dir,
                ))
                .unwrap(); //app所在目录
            USER_FILES_DIR
                .set(APP_RESOURCE_DIR.get().unwrap().join("user_files"))
                .unwrap(); //用户文件所在目录
            GPT_SOVITS_MODEL_DIR
                .set(USER_FILES_DIR.get().unwrap().join("gpt_sovits_model"))
                .unwrap(); //gpt-sovits模型所在目录
            NOVEL_OUTPUT_DIR
                .set(USER_FILES_DIR.get().unwrap().join("novel_output"))
                .unwrap();
            utils::default_utils::ensure_path_exists(USER_FILES_DIR.get().unwrap()).unwrap();
            utils::default_utils::ensure_path_exists(GPT_SOVITS_MODEL_DIR.get().unwrap()).unwrap();
            utils::default_utils::ensure_path_exists(NOVEL_OUTPUT_DIR.get().unwrap()).unwrap();
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
        .invoke_handler(tauri::generate_handler![
            input_enter,
            is_container_running,
            start_gpt_sovits,
            save_novel_audio,
            open_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
