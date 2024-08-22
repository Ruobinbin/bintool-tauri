use crate::utils;
use std::path::PathBuf;
use tauri::command;

use crate::GPT_SOVITS_MODEL_DIR;
use crate::NOVEL_OUTPUT_DIR;

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

    // let cmd = vec![
    //     "-i", "/workspace/novel_output/audio_1723745132049.wav",
    //     "-c:v", "libx264",
    //     "/workspace/novel_output/output.mp4"
    // ];

    // utils::bollard_utils::create_and_run_ffmpeg_container(cmd)
    //     .await
    //     .map_err(|e| e.to_string())
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
pub async fn get_gpt_sovits_models() -> Result<Vec<utils::gpt_sovits_utils::GptSovitsModel>, String>
{
    let path = GPT_SOVITS_MODEL_DIR.get().unwrap();
    let models = utils::gpt_sovits_utils::get_gpt_sovits_models(path);
    Ok(models)
}

//写入字符串到文件
#[command]
pub fn write_string_to_file(text: &str, file_path: String) -> Result<(), String> {
    let file_path = PathBuf::from(file_path);
    let _ = utils::default_utils::write_string_to_file(text, file_path).map_err(|e| e.to_string());
    Ok(())
}

//运行ffmpeg命令
#[command]
pub async fn run_ffmpeg_cmd(cmd: Vec<&str>) -> Result<(), String> {
    utils::bollard_utils::create_and_run_ffmpeg_container(cmd)
        .await
        .map_err(|e| e.to_string())
}

//运行aeneas命令
#[command]
pub async fn run_aeneas_cmd(
    audio_path: String,
    text_path: String,
    output_path: String,
) -> Result<(), String> {
    utils::bollard_utils::create_and_run_aeneas_container(audio_path, text_path, output_path)
        .await
        .map_err(|e| e.to_string())
}
