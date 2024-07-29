use std::io::Write;
use std::path::PathBuf;
use std::fs::{self, File};
use std::time::{SystemTime, UNIX_EPOCH};

//创建目录
pub fn ensure_path_exists(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(())
}

//删除长路径前缀
pub fn remove_long_path_prefix(path: &PathBuf) -> PathBuf {
    let path_str = path.to_str().unwrap();
    if path_str.starts_with(r"\\?\") {
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}

//写入音频数据到文件
pub fn write_audio_to_file(audio_data: Vec<u8>, file_path: PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    file.write_all(&audio_data)?;
    Ok(())
}

// 获取当前时间戳
pub fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

