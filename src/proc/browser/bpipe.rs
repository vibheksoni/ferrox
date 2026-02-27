use crate::sprotect;
use std::ptr::null_mut;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use winapi::um::{
    namedpipeapi::{CreateNamedPipeW, ConnectNamedPipe, PeekNamedPipe},
    fileapi::{WriteFile, ReadFile},
    sysinfoapi::GetTickCount,
    errhandlingapi::GetLastError,
    handleapi::INVALID_HANDLE_VALUE,
};

pub fn generate_pipe_name() -> String {
    let pipe_base = sprotect!("\\\\.\\pipe\\");
    let uuid_str = format!("{:x}", rand::random::<u64>());
    format!("{}{}", pipe_base, uuid_str)
}

pub fn create_server(pipe_name: &str) -> Option<winapi::um::winnt::HANDLE> {
    let wide_name: Vec<u16> = OsString::from(pipe_name).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        let handle = CreateNamedPipeW(
            wide_name.as_ptr(),
            0x00000003,
            0x00000004 | 0x00000002 | 0x00000000,
            1,
            4096,
            4096,
            0,
            null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            None
        } else {
            Some(handle)
        }
    }
}

pub async fn run_communication(pipe_handle: winapi::um::winnt::HANDLE) {
    unsafe {
        if ConnectNamedPipe(pipe_handle, null_mut()) == 0 {
            let error = GetLastError();
            if error != 535 {
                return;
            }
        }

        send_message(pipe_handle, &sprotect!("VERBOSE_FALSE"));
        
        let output_path = sprotect!("C:\\temp\\extract");
        std::fs::create_dir_all(&output_path).ok();
        
        send_message(pipe_handle, &output_path);
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
        }

        let timeout_ms = 60000;
        let start_time = GetTickCount();
        let completion_signal = sprotect!("__DLL_PIPE_COMPLETION_SIGNAL__");

        loop {
            if GetTickCount() - start_time > timeout_ms {
                break;
            }

            let mut bytes_available = 0;
            if PeekNamedPipe(pipe_handle, null_mut(), 0, null_mut(), &mut bytes_available, null_mut()) == 0 {
                break;
            }

            if bytes_available == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            let mut buffer = vec![0u8; 4096];
            let mut bytes_read = 0;

            if ReadFile(pipe_handle, buffer.as_mut_ptr() as *mut _, buffer.len() as u32, &mut bytes_read, null_mut()) == 0 {
                break;
            }

            if bytes_read > 0 {
                buffer.truncate(bytes_read as usize);
                let message = String::from_utf8_lossy(&buffer);

                if message.contains(&completion_signal) {
                    break;
                }
            }
        }
    }
}

fn send_message(pipe_handle: winapi::um::winnt::HANDLE, message: &str) {
    unsafe {
        let mut message_with_null = message.to_string();
        message_with_null.push('\0');
        let message_bytes = message_with_null.as_bytes();
        let mut bytes_written = 0;
        WriteFile(
            pipe_handle,
            message_bytes.as_ptr() as *const _,
            message_bytes.len() as u32,
            &mut bytes_written,
            null_mut(),
        );
    }
}