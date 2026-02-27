#![allow(dead_code)]
#![allow(unused_variables)]

use std::arch::asm;
use std::ffi::c_void;

// Syscall numbers (Windows 10/11 x64)
// These change per Windows version - we'll resolve dynamically
#[derive(Debug, Clone, Copy)]
pub struct SyscallNumbers {
    pub nt_allocate_virtual_memory: u16,
    pub nt_write_virtual_memory: u16,
    pub nt_protect_virtual_memory: u16,
    pub nt_create_thread_ex: u16,
    pub nt_open_process: u16,
    pub nt_close: u16,
    pub nt_query_system_information: u16,
}

// NTDLL structures for parsing
#[repr(C)]
struct IMAGE_DOS_HEADER {
    e_magic: u16,
    e_cblp: u16,
    e_cp: u16,
    e_crlc: u16,
    e_cparhdr: u16,
    e_minalloc: u16,
    e_maxalloc: u16,
    e_ss: u16,
    e_sp: u16,
    e_csum: u16,
    e_ip: u16,
    e_cs: u16,
    e_lfarlc: u16,
    e_ovno: u16,
    e_res: [u16; 4],
    e_oemid: u16,
    e_oeminfo: u16,
    e_res2: [u16; 10],
    e_lfanew: i32,
}

#[repr(C)]
struct IMAGE_NT_HEADERS64 {
    signature: u32,
    file_header: IMAGE_FILE_HEADER,
    optional_header: IMAGE_OPTIONAL_HEADER64,
}

#[repr(C)]
struct IMAGE_FILE_HEADER {
    machine: u16,
    number_of_sections: u16,
    time_date_stamp: u32,
    pointer_to_symbol_table: u32,
    number_of_symbols: u32,
    size_of_optional_header: u16,
    characteristics: u16,
}

#[repr(C)]
struct IMAGE_OPTIONAL_HEADER64 {
    magic: u16,
    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entry_point: u32,
    base_of_code: u32,
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    major_operating_system_version: u16,
    minor_operating_system_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    win32_version_value: u32,
    size_of_image: u32,
    size_of_headers: u32,
    check_sum: u32,
    subsystem: u16,
    dll_characteristics: u16,
    size_of_stack_reserve: u64,
    size_of_stack_commit: u64,
    size_of_heap_reserve: u64,
    size_of_heap_commit: u64,
    loader_flags: u32,
    number_of_rva_and_sizes: u32,
    data_directory: [IMAGE_DATA_DIRECTORY; 16],
}

#[repr(C)]
struct IMAGE_DATA_DIRECTORY {
    virtual_address: u32,
    size: u32,
}

#[repr(C)]
struct IMAGE_EXPORT_DIRECTORY {
    characteristics: u32,
    time_date_stamp: u32,
    major_version: u16,
    minor_version: u16,
    name: u32,
    base: u32,
    number_of_functions: u32,
    number_of_names: u32,
    address_of_functions: u32,
    address_of_names: u32,
    address_of_name_ordinals: u32,
}

/// Get NTDLL base address from PEB
#[inline(never)]
pub unsafe fn get_ntdll_base() -> *mut u8 {
    let peb: *mut c_void;
    
    // Get PEB from GS register (x64)
    asm!(
        "mov {}, gs:[0x60]",
        out(reg) peb,
    );
    
    if peb.is_null() {
        return std::ptr::null_mut();
    }
    
    // PEB->Ldr
    let ldr = *(peb.offset(0x18) as *const *mut c_void);
    if ldr.is_null() {
        return std::ptr::null_mut();
    }
    
    // InMemoryOrderModuleList
    let mut list_entry = *(ldr.offset(0x20) as *const *mut c_void);
    
    // Walk the list (ntdll is usually 2nd or 3rd)
    for _ in 0..10 {
        if list_entry.is_null() {
            break;
        }
        
        // Get DllBase
        let dll_base = *(list_entry.offset(0x20) as *const *mut u8);
        
        // Get BaseDllName
        let base_dll_name_buffer = *(list_entry.offset(0x50) as *const *const u16);
        
        if !base_dll_name_buffer.is_null() {
            // Check if it's ntdll.dll
            let mut name = Vec::new();
            let mut i = 0;
            loop {
                let c = *base_dll_name_buffer.offset(i);
                if c == 0 {
                    break;
                }
                name.push(c);
                i += 1;
            }
            
            let name_str = String::from_utf16_lossy(&name).to_uppercase();
            if name_str.contains("NTDLL") {
                return dll_base;
            }
        }
        
        // Next entry
        list_entry = *(list_entry as *const *mut c_void);
    }
    
    std::ptr::null_mut()
}

/// Parse EAT and find syscall number by function name
#[inline(never)]
pub unsafe fn get_syscall_number(ntdll_base: *mut u8, function_name: &str) -> Option<u16> {
    if ntdll_base.is_null() {
        return None;
    }
    
    // Parse DOS header
    let dos_header = &*(ntdll_base as *const IMAGE_DOS_HEADER);
    if dos_header.e_magic != 0x5A4D {  // "MZ"
        return None;
    }
    
    // Parse NT headers
    let nt_headers = &*(ntdll_base.offset(dos_header.e_lfanew as isize) as *const IMAGE_NT_HEADERS64);
    if nt_headers.signature != 0x4550 {  // "PE"
        return None;
    }
    
    // Get export directory
    let export_dir_rva = nt_headers.optional_header.data_directory[0].virtual_address;
    if export_dir_rva == 0 {
        return None;
    }
    
    let export_dir = &*(ntdll_base.offset(export_dir_rva as isize) as *const IMAGE_EXPORT_DIRECTORY);
    
    // Get export tables
    let functions = ntdll_base.offset(export_dir.address_of_functions as isize) as *const u32;
    let names = ntdll_base.offset(export_dir.address_of_names as isize) as *const u32;
    let ordinals = ntdll_base.offset(export_dir.address_of_name_ordinals as isize) as *const u16;
    
    // Search for function
    for i in 0..export_dir.number_of_names {
        let name_rva = *names.offset(i as isize);
        let name_ptr = ntdll_base.offset(name_rva as isize) as *const i8;
        
        // Compare name
        let mut j = 0;
        let mut matches = true;
        let fn_bytes = function_name.as_bytes();
        
        loop {
            let c = *name_ptr.offset(j);
            if c == 0 {
                if j as usize == fn_bytes.len() {
                    break;
                } else {
                    matches = false;
                    break;
                }
            }
            if j as usize >= fn_bytes.len() || c as u8 != fn_bytes[j as usize] {
                matches = false;
                break;
            }
            j += 1;
        }
        
        if matches {
            let ordinal = *ordinals.offset(i as isize);
            let func_rva = *functions.offset(ordinal as isize);
            let func_addr = ntdll_base.offset(func_rva as isize);
            
            // Extract syscall number from function prologue
            // Typical pattern: mov r10, rcx; mov eax, <SSN>; ...
            let bytes = std::slice::from_raw_parts(func_addr, 32);
            
            // Look for: mov eax, <syscall_number>
            // Opcodes: B8 XX XX 00 00 (mov eax, imm32) or BA XX XX 00 00
            for k in 0..24 {
                if bytes[k] == 0xB8 {
                    // mov eax, imm32
                    let ssn = u16::from_le_bytes([bytes[k + 1], bytes[k + 2]]);
                    return Some(ssn);
                } else if bytes[k] == 0xBA && k + 4 < bytes.len() {
                    // mov edx, imm32 (alternative)
                    let ssn = u16::from_le_bytes([bytes[k + 1], bytes[k + 2]]);
                    return Some(ssn);
                }
            }
        }
    }
    
    None
}

/// Execute syscall with 4 parameters
#[inline(never)]
pub unsafe fn syscall_4(ssn: u16, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i32 {
    let mut result: i32;
    
    unsafe {
        asm!(
            "mov r10, rcx",
            "mov eax, {ssn:e}",
            "syscall",
            ssn = in(reg) ssn as u32,
            in("rcx") arg1,
            in("rdx") arg2,
            in("r8") arg3,
            in("r9") arg4,
            lateout("eax") result,
        );
    }
    
    result
}

/// Execute syscall with 5 parameters
#[inline(never)]
pub unsafe fn syscall_5(ssn: u16, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> i32 {
    let mut result: i32;
    
    unsafe {
        asm!(
            "push {arg5}",
            "sub rsp, 0x20",
            "mov r10, rcx",
            "mov eax, {ssn:e}",
            "syscall",
            "add rsp, 0x28",
            ssn = in(reg) ssn as u32,
            arg5 = in(reg) arg5,
            in("rcx") arg1,
            in("rdx") arg2,
            in("r8") arg3,
            in("r9") arg4,
            lateout("eax") result,
        );
    }
    
    result
}

pub fn init_syscalls() -> Option<SyscallNumbers> {
    unsafe {
        let ntdll = get_ntdll_base();
        if ntdll.is_null() {
            return None;
        }
        
        Some(SyscallNumbers {
            nt_allocate_virtual_memory: get_syscall_number(ntdll, "NtAllocateVirtualMemory")?,
            nt_write_virtual_memory: get_syscall_number(ntdll, "NtWriteVirtualMemory")?,
            nt_protect_virtual_memory: get_syscall_number(ntdll, "NtProtectVirtualMemory")?,
            nt_create_thread_ex: get_syscall_number(ntdll, "NtCreateThreadEx")?,
            nt_open_process: get_syscall_number(ntdll, "NtOpenProcess")?,
            nt_close: get_syscall_number(ntdll, "NtClose")?,
            nt_query_system_information: get_syscall_number(ntdll, "NtQuerySystemInformation")?,
        })
    }
}