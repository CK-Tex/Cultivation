use crate::system_helpers::*;
use std::path::{Path, PathBuf};
use tauri::Manager;

fn resolve_lang_path(window: &tauri::Window, lang: &str) -> Option<PathBuf> {
    let file = format!("lang/{}.json", lang);

    if let Some(path) = window.app_handle().path_resolver().resolve_resource(&file) {
        return Some(path);
    }

    Some([&install_location(), "lang", &format!("{}.json", lang)]
        .iter()
        .collect())
}

#[tauri::command]
pub async fn get_lang(window: tauri::Window, lang: String) -> String {
    let lang = lang.to_lowercase();

    let Some(lang_path) = resolve_lang_path(&window, &lang) else {
        emit_lang_err(window, format!("Failed to resolve language file: {}", lang));
        return "".to_string();
    };

    match std::fs::read_to_string(&lang_path) {
        Ok(x) => x,
        Err(e) => {
            emit_lang_err(
                window,
                format!("Failed to read language file {}: {}", lang_path.display(), e),
            );
            "".to_string()
        }
    }
}

#[tauri::command]
pub async fn get_languages(window: tauri::Window) -> std::collections::HashMap<String, String> {
    let mut languages = std::collections::HashMap::new();

    let lang_dir = window
        .app_handle()
        .path_resolver()
        .resolve_resource("lang")
        .unwrap_or_else(|| Path::new(&install_location()).join("lang"));

    let Ok(lang_files) = std::fs::read_dir(&lang_dir) else {
        emit_lang_err(
            window,
            format!("Failed to read language dir: {}", lang_dir.display()),
        );
        return languages;
    };

    for entry in lang_files.flatten() {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                languages.insert(filename.to_string(), content);
            }
            Err(e) => {
                emit_lang_err(
                    window.clone(),
                    format!("Failed to read language file {}: {}", path.display(), e),
                );
            }
        }
    }

    languages
}

pub fn emit_lang_err(window: tauri::Window, msg: String) {
    let mut res_hash = std::collections::HashMap::new();
    res_hash.insert("error".to_string(), msg);
    let _ = window.emit("lang_error", &res_hash);
}
