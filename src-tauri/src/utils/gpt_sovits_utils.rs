use super::default_utils;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReferWav {
    refer_wav_path: String,
    prompt_text: String,
    prompt_language: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GptSovitsModel {
    model_name: String,
    gpt_model_path: String,
    sovits_model_path: String,
    refer_wavs: Vec<ReferWav>,
}

pub fn get_gpt_sovits_models(base_dir: &Path) -> Vec<GptSovitsModel> {
    let mut models = Vec::new();

    let dir_path_list = default_utils::get_dir_in_dir(base_dir);

    for dir_path in dir_path_list {
        let model_name = default_utils::get_dir_or_file_name(&dir_path);
        let gpt_model_path = default_utils::get_files_with_extension(&dir_path, "pth")
            .first()
            .unwrap()
            .display()
            .to_string();
        let sovits_model_path = default_utils::get_files_with_extension(&dir_path, "ckpt")
            .first()
            .unwrap()
            .display()
            .to_string();
        let wav_files: Vec<_> = default_utils::get_files_with_extension(&dir_path, "wav");
        let refer_wavs: Vec<ReferWav> = wav_files
            .into_iter()
            .map(|wav_path| ReferWav {
                refer_wav_path: wav_path.display().to_string(),
                prompt_text: default_utils::get_file_name_without_extension(&wav_path),
                prompt_language: "zh".to_string(),
            })
            .collect();

        let model = GptSovitsModel {
            model_name,
            gpt_model_path,
            sovits_model_path,
            refer_wavs,
        };

        models.push(model);
    }
    models
}
