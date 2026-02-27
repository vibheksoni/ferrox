use crate::sprotect;
use std::process::Command;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use winreg::enums::*;
use winreg::RegKey;
use std::error::Error;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub struct DeviceFingerprint {
    fingerprint: Option<String>,
}

impl DeviceFingerprint {
    pub fn new() -> Self {
        DeviceFingerprint {
            fingerprint: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<String, Box<dyn Error>> {
        if let Ok(existing) = self.retrieve_fingerprint() {
            self.fingerprint = Some(existing.clone());
            return Ok(existing);
        }

        let new_fingerprint = self.generate_fingerprint().await?;
        self.store_fingerprint(&new_fingerprint)?;
        self.fingerprint = Some(new_fingerprint.clone());
        Ok(new_fingerprint)
    }

    async fn generate_fingerprint(&self) -> Result<String, Box<dyn Error>> {
        let cpu_id = self.get_cpu_id().await.unwrap_or_else(|_| sprotect!("NOCPU"));
        let motherboard_serial = self.get_motherboard_serial().await.unwrap_or_else(|_| sprotect!("NOMB"));
        let disk_serial = self.get_disk_serial().await.unwrap_or_else(|_| sprotect!("NODISK"));
        let machine_guid = self.get_machine_guid().unwrap_or_else(|_| sprotect!("NOGUID"));
        
        let combined = format!(
            "{}{}{}{}{}{}{}",
            sprotect!("{"),
            cpu_id,
            sprotect!("-"),
            motherboard_serial,
            sprotect!("-"),
            disk_serial,
            sprotect!("-"),
        );
        let combined_with_guid = format!("{}{}{}", combined, machine_guid, sprotect!("}"));
        
        let mut hasher = Sha256::new();
        hasher.update(combined_with_guid.as_bytes());
        let hash_result = hasher.finalize();
        
        let fingerprint_hex = format!("{:x}", hash_result);
        let encoded = general_purpose::STANDARD.encode(&fingerprint_hex);
        
        Ok(encoded)
    }

    async fn get_cpu_id(&self) -> Result<String, Box<dyn Error>> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let mut cmd = Command::new(sprotect!("powershell"));
        cmd.arg(sprotect!("-Command"))
           .arg(sprotect!("(Get-WmiObject Win32_Processor).ProcessorId"))
           .creation_flags(CREATE_NO_WINDOW);
        
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let output = cmd.output()?;
        
        let cpu_id = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        if cpu_id.is_empty() {
            return Err(sprotect!("Failed to get CPU ID").into());
        }
        
        Ok(cpu_id)
    }

    async fn get_motherboard_serial(&self) -> Result<String, Box<dyn Error>> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let mut cmd = Command::new(sprotect!("powershell"));
        cmd.arg(sprotect!("-Command"))
           .arg(sprotect!("(Get-WmiObject Win32_BaseBoard).SerialNumber"))
           .creation_flags(CREATE_NO_WINDOW);
        
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let output = cmd.output()?;
        
        let serial = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        if serial.is_empty() || serial == sprotect!("To be filled by O.E.M.") {
            return Err(sprotect!("Invalid motherboard serial").into());
        }
        
        Ok(serial)
    }

    async fn get_disk_serial(&self) -> Result<String, Box<dyn Error>> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        let mut cmd = Command::new(sprotect!("powershell"));
        cmd.arg(sprotect!("-Command"))
           .arg(sprotect!("(Get-WmiObject Win32_PhysicalMedia | Where-Object {$_.SerialNumber -ne $null} | Select-Object -First 1).SerialNumber"))
           .creation_flags(CREATE_NO_WINDOW);
        
        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        let output = cmd.output()?;
        
        let serial = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        if serial.is_empty() {
            return Err(sprotect!("Failed to get disk serial").into());
        }
        
        Ok(serial)
    }

    fn get_machine_guid(&self) -> Result<String, Box<dyn Error>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let crypto_key_path = sprotect!("SOFTWARE\\Microsoft\\Cryptography");
        let crypto_key = hklm.open_subkey(&crypto_key_path)?;
        let machine_guid: String = crypto_key.get_value(sprotect!("MachineGuid"))?;
        Ok(machine_guid)
    }

    fn store_fingerprint(&self, fingerprint: &str) -> Result<(), Box<dyn Error>> {
        self.store_in_primary_location(fingerprint)?;
        let _ = self.store_in_backup_location(fingerprint);
        let _ = self.store_in_tertiary_location(fingerprint);
        Ok(())
    }

    fn store_in_primary_location(&self, fingerprint: &str) -> Result<(), Box<dyn Error>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let driver_path = sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\DriverSearching");
        
        let (key, _) = hklm.create_subkey(&driver_path)?;
        key.set_value(sprotect!("DriverVersionId"), &fingerprint)?;
        
        Ok(())
    }

    fn store_in_backup_location(&self, fingerprint: &str) -> Result<(), Box<dyn Error>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let device_path = sprotect!("SOFTWARE\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\MuiCache");
        
        let (key, _) = hkcu.create_subkey(&device_path)?;
        let cache_key = sprotect!("C:\\Windows\\System32\\DeviceProperties.exe.FriendlyName");
        key.set_value(&cache_key, &fingerprint)?;
        
        Ok(())
    }

    fn store_in_tertiary_location(&self, fingerprint: &str) -> Result<(), Box<dyn Error>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let system_path = sprotect!("SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers\\Configuration");
        
        let (key, _) = hklm.create_subkey(&system_path)?;
        key.set_value(sprotect!("Timestamp"), &fingerprint)?;
        
        Ok(())
    }

    fn retrieve_fingerprint(&self) -> Result<String, Box<dyn Error>> {
        if let Ok(fp) = self.retrieve_from_primary() {
            return Ok(fp);
        }
        
        if let Ok(fp) = self.retrieve_from_backup() {
            let _ = self.store_in_primary_location(&fp);
            return Ok(fp);
        }
        
        if let Ok(fp) = self.retrieve_from_tertiary() {
            let _ = self.store_in_primary_location(&fp);
            let _ = self.store_in_backup_location(&fp);
            return Ok(fp);
        }
        
        Err(sprotect!("No existing fingerprint found").into())
    }

    fn retrieve_from_primary(&self) -> Result<String, Box<dyn Error>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let driver_path = sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\DriverSearching");
        
        let key = hklm.open_subkey(&driver_path)?;
        let fingerprint: String = key.get_value(sprotect!("DriverVersionId"))?;
        
        if fingerprint.is_empty() {
            return Err(sprotect!("Empty fingerprint").into());
        }
        
        Ok(fingerprint)
    }

    fn retrieve_from_backup(&self) -> Result<String, Box<dyn Error>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let device_path = sprotect!("SOFTWARE\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\Shell\\MuiCache");
        
        let key = hkcu.open_subkey(&device_path)?;
        let cache_key = sprotect!("C:\\Windows\\System32\\DeviceProperties.exe.FriendlyName");
        let fingerprint: String = key.get_value(&cache_key)?;
        
        if fingerprint.is_empty() {
            return Err(sprotect!("Empty fingerprint").into());
        }
        
        Ok(fingerprint)
    }

    fn retrieve_from_tertiary(&self) -> Result<String, Box<dyn Error>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let system_path = sprotect!("SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers\\Configuration");
        
        let key = hklm.open_subkey(&system_path)?;
        let fingerprint: String = key.get_value(sprotect!("Timestamp"))?;
        
        if fingerprint.is_empty() {
            return Err(sprotect!("Empty fingerprint").into());
        }
        
        Ok(fingerprint)
    }

    pub fn device_exists(&self) -> bool {
        self.fingerprint.is_some() || self.retrieve_fingerprint().is_ok()
    }

    pub fn get_device_id(&self) -> Option<String> {
        self.fingerprint.clone().or_else(|| self.retrieve_fingerprint().ok())
    }

    pub async fn validate_device(&self, server_id: &str) -> bool {
        if let Some(local_id) = &self.fingerprint {
            return local_id == server_id;
        }
        
        if let Ok(stored_id) = self.retrieve_fingerprint() {
            return stored_id == server_id;
        }
        
        false
    }
}

pub async fn get_or_create_device_id() -> Result<String, Box<dyn Error>> {
    let mut fingerprinter = DeviceFingerprint::new();
    fingerprinter.initialize().await
}

pub fn check_device_registered() -> bool {
    let fingerprinter = DeviceFingerprint::new();
    fingerprinter.device_exists()
}

pub fn check_persistence_installed() -> Result<bool, Box<dyn Error>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let startup_path = sprotect!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run");

    if let Ok(key) = hklm.open_subkey(&startup_path) {
        if let Ok(_value) = key.get_value::<String, _>(sprotect!("DeviceUpdateDriverVersion")) {
            return Ok(true);
        }
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(&startup_path) {
        if let Ok(_value) = key.get_value::<String, _>(sprotect!("DeviceUpdateDriverVersion")) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn get_device_info() -> Result<String, Box<dyn Error>> {
    let computer_name = std::env::var(sprotect!("COMPUTERNAME")).unwrap_or_else(|_| sprotect!("Unknown"));
    let username = std::env::var(sprotect!("USERNAME")).unwrap_or_else(|_| sprotect!("Unknown"));
    let os_version = std::env::var(sprotect!("OS")).unwrap_or_else(|_| sprotect!("Windows"));

    let device_info = format!("{}|{}|{}", computer_name, username, os_version);
    Ok(device_info)
}