use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::ptr::null_mut;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use winapi::um::{
    winreg::HKEY_LOCAL_MACHINE,
    winnt::KEY_READ,
};
use winapi::shared::winerror::ERROR_SUCCESS;

pub fn find_path() -> Option<String> {
    let paths = [
        sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\msedge.exe"),
        sprotect!("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\App Paths\\msedge.exe"),
    ];

    for path in &paths {
        let wide_path: Vec<u16> = OsString::from(path).encode_wide().chain(std::iter::once(0)).collect();
        let mut key: *mut std::ffi::c_void = null_mut();

        unsafe {
            if HashedAPIs::reg_open_key_ex_w(
                HKEY_LOCAL_MACHINE as *mut _,
                wide_path.as_ptr(),
                0,
                KEY_READ,
                &mut key
            ) == ERROR_SUCCESS as i32 {
                let mut buffer = vec![0u16; 512];
                let mut size = (buffer.len() * 2) as u32;

                if HashedAPIs::reg_query_value_ex_w(
                    key,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    buffer.as_mut_ptr() as *mut u8,
                    &mut size
                ) == ERROR_SUCCESS as i32 {
                    let len = (size / 2) as usize;
                    buffer.truncate(len);
                    if let Some(null_pos) = buffer.iter().position(|&x| x == 0) {
                        buffer.truncate(null_pos);
                    }
                    HashedAPIs::reg_close_key(key);
                    return Some(String::from_utf16_lossy(&buffer));
                }
                HashedAPIs::reg_close_key(key);
            }
        }
    }

    let additional_paths = [
        sprotect!("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe"),
        sprotect!("C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe"),
    ];

    for path in &additional_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}