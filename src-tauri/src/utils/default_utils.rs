use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

//创建目录
pub fn ensure_path_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
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

//获取目录下所有文件
pub fn get_all_in_dir(dir_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir_path.is_dir() {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                files.push(entry.path());
            }
        }
    }
    files
}

//获取目录下所有目录
pub fn get_dir_in_dir(dir_path: &Path) -> Vec<PathBuf> {
    get_all_in_dir(dir_path)
        .into_iter()
        .filter(|path| path.is_dir())
        .collect()
}

/// 获取目录下所有文件
pub fn get_files_in_dir(dir_path: &Path) -> Vec<PathBuf> {
    get_all_in_dir(dir_path)
        .into_iter()
        .filter(|path| path.is_file())
        .collect()
}

//获取目录名
pub fn get_dir_or_file_name(dir_path: &Path) -> String {
    dir_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

//获取文件名不带后缀
pub fn get_file_name_without_extension(file_path: &Path) -> String {
    file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_string()
}

//获取指定后缀的文件
pub fn get_files_with_extension(dir_path: &Path, extension: &str) -> Vec<PathBuf> {
    get_all_in_dir(dir_path)
        .into_iter()
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == extension))
        .collect()
}

// 写入字符串内容到文件
pub fn write_string_to_file(text: &str, file_path: PathBuf) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}
