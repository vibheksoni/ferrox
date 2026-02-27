#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::ffi::c_void;

#[repr(C)]
struct LIST_ENTRY {
    Flink: *mut LIST_ENTRY,
    Blink: *mut LIST_ENTRY,
}

#[repr(C)]
struct UNICODE_STRING {
    Length: u16,
    MaximumLength: u16,
    Buffer: *mut u16,
}

#[repr(C)]
struct PEB_LDR_DATA {
    Reserved1: [u8; 8],
    Reserved2: [*mut c_void; 3],
    InMemoryOrderModuleList: LIST_ENTRY,
}

#[repr(C)]
struct PEB {
    Reserved1: [u8; 2],
    BeingDebugged: u8,
    Reserved2: [u8; 1],
    Reserved3: [*mut c_void; 2],
    Ldr: *mut PEB_LDR_DATA,
}

#[repr(C)]
struct LDR_DATA_TABLE_ENTRY {
    Reserved1: [*mut c_void; 2],
    InMemoryOrderLinks: LIST_ENTRY,
    Reserved2: [*mut c_void; 2],
    DllBase: *mut c_void,
    EntryPoint: *mut c_void,
    Reserved3: *mut c_void,
    FullDllName: UNICODE_STRING,
    Reserved4: [u8; 8],
    Reserved5: [*mut c_void; 3],
    Reserved6: *mut c_void,
    TimeDateStamp: u32,
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn get_peb() -> *mut PEB {
    let peb: *mut PEB;
    std::arch::asm!(
        "mov {}, gs:[0x60]",
        out(reg) peb,
        options(nostack, preserves_flags)
    );
    peb
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn get_peb() -> *mut PEB {
    let peb: *mut PEB;
    std::arch::asm!(
        "mov {}, fs:[0x30]",
        out(reg) peb,
        options(nostack, preserves_flags)
    );
    peb
}

pub unsafe fn get_module_by_hash(module_hash: u64, hash_key: u64) -> Option<*mut c_void> {
    let peb = get_peb();
    if peb.is_null() {
        return None;
    }

    let ldr = (*peb).Ldr;
    if ldr.is_null() {
        return None;
    }

    let list_head = &(*ldr).InMemoryOrderModuleList as *const LIST_ENTRY as *mut u8;
    let mut current_entry = (*ldr).InMemoryOrderModuleList.Flink;

    while current_entry != list_head as *mut LIST_ENTRY {
        let table_entry = (current_entry as *mut u8).offset(-16) as *mut LDR_DATA_TABLE_ENTRY;
        
        let dll_name = &(*table_entry).FullDllName;
        if !dll_name.Buffer.is_null() && dll_name.Length > 0 {
            let name_len = (dll_name.Length / 2) as usize;
            
            // Extract just the filename from full path (after last backslash)
            let mut filename_start = 0;
            for i in 0..name_len {
                let ch = *dll_name.Buffer.add(i);
                if ch == 92 { // backslash '\\'
                    filename_start = i + 1;
                }
            }
            
            // Convert filename to uppercase
            let mut name_upper = Vec::with_capacity(name_len - filename_start);
            for i in filename_start..name_len {
                let ch = *dll_name.Buffer.add(i);
                name_upper.push(if ch >= 97 && ch <= 122 { ch - 32 } else { ch });
            }

            let hash = name_upper.iter().fold(hash_key, |h, &b| {
                ((h << 5).wrapping_add(h)).wrapping_add(b as u64)
            });

            if hash == module_hash {
                return Some((*table_entry).DllBase);
            }
        }

        current_entry = (*current_entry).Flink;
        
        if current_entry == list_head as *mut LIST_ENTRY {
            break;
        }
    }

    None
}

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
struct IMAGE_FILE_HEADER {
    Machine: u16,
    NumberOfSections: u16,
    TimeDateStamp: u32,
    PointerToSymbolTable: u32,
    NumberOfSymbols: u32,
    SizeOfOptionalHeader: u16,
    Characteristics: u16,
}

#[repr(C)]
struct IMAGE_DATA_DIRECTORY {
    VirtualAddress: u32,
    Size: u32,
}

#[repr(C)]
struct IMAGE_OPTIONAL_HEADER64 {
    Magic: u16,
    MajorLinkerVersion: u8,
    MinorLinkerVersion: u8,
    SizeOfCode: u32,
    SizeOfInitializedData: u32,
    SizeOfUninitializedData: u32,
    AddressOfEntryPoint: u32,
    BaseOfCode: u32,
    ImageBase: u64,
    SectionAlignment: u32,
    FileAlignment: u32,
    MajorOperatingSystemVersion: u16,
    MinorOperatingSystemVersion: u16,
    MajorImageVersion: u16,
    MinorImageVersion: u16,
    MajorSubsystemVersion: u16,
    MinorSubsystemVersion: u16,
    Win32VersionValue: u32,
    SizeOfImage: u32,
    SizeOfHeaders: u32,
    CheckSum: u32,
    Subsystem: u16,
    DllCharacteristics: u16,
    SizeOfStackReserve: u64,
    SizeOfStackCommit: u64,
    SizeOfHeapReserve: u64,
    SizeOfHeapCommit: u64,
    LoaderFlags: u32,
    NumberOfRvaAndSizes: u32,
    DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}

#[repr(C)]
struct IMAGE_NT_HEADERS64 {
    Signature: u32,
    FileHeader: IMAGE_FILE_HEADER,
    OptionalHeader: IMAGE_OPTIONAL_HEADER64,
}

#[repr(C)]
struct IMAGE_EXPORT_DIRECTORY {
    Characteristics: u32,
    TimeDateStamp: u32,
    MajorVersion: u16,
    MinorVersion: u16,
    Name: u32,
    Base: u32,
    NumberOfFunctions: u32,
    NumberOfNames: u32,
    AddressOfFunctions: u32,
    AddressOfNames: u32,
    AddressOfNameOrdinals: u32,
}

pub unsafe fn get_proc_by_hash(
    module_base: *mut c_void,
    func_hash: u64,
    hash_key: u64
) -> Option<*mut c_void> {
    if module_base.is_null() {
        return None;
    }

    let base = module_base as *mut u8;

    let dos_header = base as *mut IMAGE_DOS_HEADER;
    if (*dos_header).e_magic != 0x5A4D {
        return None;
    }

    let nt_headers = base.add((*dos_header).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
    if (*nt_headers).Signature != 0x4550 {
        return None;
    }

    let export_dir_rva = (*nt_headers).OptionalHeader.DataDirectory[0].VirtualAddress;
    if export_dir_rva == 0 {
        return None;
    }

    let export_dir = base.add(export_dir_rva as usize) as *mut IMAGE_EXPORT_DIRECTORY;

    let names = base.add((*export_dir).AddressOfNames as usize) as *const u32;
    let functions = base.add((*export_dir).AddressOfFunctions as usize) as *const u32;
    let ordinals = base.add((*export_dir).AddressOfNameOrdinals as usize) as *const u16;

    for i in 0..(*export_dir).NumberOfNames {
        let name_rva = *names.add(i as usize);
        let name_ptr = base.add(name_rva as usize) as *const u8;

        let mut hash = hash_key;
        let mut j = 0;
        loop {
            let byte = *name_ptr.add(j);
            if byte == 0 {
                break;
            }
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
            j += 1;
        }

        if hash == func_hash {
            let ordinal = *ordinals.add(i as usize) as usize;
            let func_rva = *functions.add(ordinal);
            return Some(base.add(func_rva as usize) as *mut c_void);
        }
    }

    None
}
