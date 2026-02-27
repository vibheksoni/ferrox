/// Volatile String - Auto-zeroing string that clears itself from memory after use
/// 
/// Prevents memory dumps from finding sensitive strings like "Login Data", "api.telegram.org", etc.

use std::ops::Deref;

pub struct VolatileString {
    data: Vec<u8>,
}

impl VolatileString {
    /// Create a new volatile string from a regular string
    pub fn new(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }
    
    /// Get the string as &str
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }
    
    /// Get the string as String (clones)
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Deref for VolatileString {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Drop for VolatileString {
    fn drop(&mut self) {
        // Zero the memory before dropping
        for byte in self.data.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        
        // Extra paranoia: overwrite with random data
        for byte in self.data.iter_mut() {
            *byte = rand::random::<u8>();
        }
        
        // Zero again
        self.data.fill(0);
    }
}

/// Macro for creating volatile protected strings
/// Same syntax as sprotect! but returns VolatileString that auto-zeros
#[macro_export]
macro_rules! sprotect_volatile {
    ($s:expr) => {{
        // First use regular sprotect! to decrypt
        let decrypted = sprotect!($s);
        // Wrap in VolatileString for auto-zeroing
        $crate::sprotect::VolatileString::new(decrypted)
    }};
}
