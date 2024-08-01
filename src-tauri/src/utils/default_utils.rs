use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

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

//打开路径
pub fn open_path(path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("路径不存在: {:?}", path));
    }

    let output = Command::new("explorer")
        .arg(path.as_os_str())
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "命令执行失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
