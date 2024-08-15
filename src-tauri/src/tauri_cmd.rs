use crate::utils;
use std::path::PathBuf;
use tauri::command;

use crate::NOVEL_OUTPUT_DIR;
use crate::GPT_SOVITS_MODEL_DIR;

//测试命令
#[command]
pub fn input_enter(value: &str) -> String {
    println!("input_enter: {}", value);
    value.to_string()
}

//判断容器是否运行
#[command]
pub async fn is_container_running(container_name: &str) -> Result<bool, String> {
    utils::bollard_utils::is_container_running(container_name)
        .await
        .map_err(|e| e.to_string())
}

//启动gpt-sovits-api
#[command]
pub async fn start_gpt_sovits_api() -> Result<(), String> {
    utils::bollard_utils::start_gpt_sovits_api()
        .await
        .map_err(|e| e.to_string())
}

//保存小说音频
#[command]
pub async fn save_novel_audio(audio_data: Vec<u8>, audio_name: &str) -> Result<String, String> {
    let file_path = NOVEL_OUTPUT_DIR.get().unwrap().join(audio_name);
    match utils::default_utils::write_audio_to_file(audio_data, file_path.clone()) {
        Ok(_) => Ok(file_path.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

//打开路径
#[command]
pub async fn open_path(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    utils::default_utils::open_path(path_buf).map_err(|e| e.to_string())
}

//获取gpt_sovits模型列表
#[command]
pub async fn get_gpt_sovits_models() -> Result<Vec<utils::gpt_sovits_utils::GptSovitsModel>, String> {
    let path = GPT_SOVITS_MODEL_DIR.get().unwrap();
    let models = utils::gpt_sovits_utils::get_gpt_sovits_models(path);
    Ok(models)
}