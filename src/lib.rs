use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
    Nonce,
    Key,
};
use rand::{Rng, thread_rng};

//#genkey(name="BLOB_KEY_ARRAY")
//#create_var(name="NETWORK_TIMEOUT_MS", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[0]
const NETWORK_TIMEOUT_MS: u8 = 0x51;
//#create_var(name="MAX_CONNECTION_RETRIES", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[1]
const MAX_CONNECTION_RETRIES: u8 = 0x1f;
//#create_var(name="BUFFER_ALLOCATION_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[2]
const BUFFER_ALLOCATION_SIZE: u8 = 0x17;
//#create_var(name="CONNECTION_ID_SEED", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[3]
const CONNECTION_ID_SEED: u8 = 0x8a;
//#create_var(name="DEFAULT_THREAD_COUNT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[4]
const DEFAULT_THREAD_COUNT: u8 = 0xbd;
//#create_var(name="INITIAL_HEAP_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[5]
const INITIAL_HEAP_SIZE: u8 = 0x9f;
//#create_var(name="VERSION_MAJOR_NUM", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[6]
const VERSION_MAJOR_NUM: u8 = 0xc9;
//#create_var(name="VERSION_MINOR_NUM", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[7]
const VERSION_MINOR_NUM: u8 = 0x43;
//#create_var(name="BUILD_NUMBER_CONST", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[8]
const BUILD_NUMBER_CONST: u8 = 0x39;
//#create_var(name="REVISION_NUMBER", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[9]
const REVISION_NUMBER: u8 = 0x07;
//#create_var(name="OPTIMAL_CPU_CORES", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[10]
const OPTIMAL_CPU_CORES: u8 = 0x84;
//#create_var(name="L2_CACHE_SIZE_KB", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[11]
const L2_CACHE_SIZE_KB: u8 = 0x18;
//#create_var(name="PACKET_SIZE_LIMIT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[12]
const PACKET_SIZE_LIMIT: u8 = 0xf5;
//#create_var(name="TCP_WINDOW_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[13]
const TCP_WINDOW_SIZE: u8 = 0x57;
//#create_var(name="CHUNK_SIZE_BYTES", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[14]
const CHUNK_SIZE_BYTES: u8 = 0x0c;
//#create_var(name="BLOCK_SIZE_DEFAULT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[15]
const BLOCK_SIZE_DEFAULT: u8 = 0x8b;
//#create_var(name="MEMORY_POOL_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[16]
const MEMORY_POOL_SIZE: u8 = 0x3b;
//#create_var(name="STACK_SIZE_LIMIT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[17]
const STACK_SIZE_LIMIT: u8 = 0x14;
//#create_var(name="HASH_FUNCTION_SEED", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[18]
const HASH_FUNCTION_SEED: u8 = 0x5a;
//#create_var(name="CRYPTO_SALT_VALUE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[19]
const CRYPTO_SALT_VALUE: u8 = 0xa2;
//#create_var(name="INITIALIZATION_VECTOR_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[20]
const INITIALIZATION_VECTOR_SIZE: u8 = 0x17;
//#create_var(name="AUTH_TAG_LENGTH", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[21]
const AUTH_TAG_LENGTH: u8 = 0xa3;
//#create_var(name="IO_QUEUE_DEPTH", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[22]
const IO_QUEUE_DEPTH: u8 = 0x7b;
//#create_var(name="BATCH_PROCESSING_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[23]
const BATCH_PROCESSING_SIZE: u8 = 0xec;
//#create_var(name="CIPHER_ROUND_COUNT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[24]
const CIPHER_ROUND_COUNT: u8 = 0xb4;
//#create_var(name="KEY_DERIVATION_STRETCH", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[25]
const KEY_DERIVATION_STRETCH: u8 = 0xc4;
//#create_var(name="NONCE_SIZE_BYTES", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[26]
const NONCE_SIZE_BYTES: u8 = 0x38;
//#create_var(name="BLOCK_CIPHER_MODE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[27]
const BLOCK_CIPHER_MODE: u8 = 0x40;
//#create_var(name="THREAD_POOL_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[28]
const THREAD_POOL_SIZE: u8 = 0x65;
//#create_var(name="IO_BUFFER_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[29]
const IO_BUFFER_SIZE: u8 = 0x94;
//#create_var(name="STREAM_IDENTIFIER", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[30]
const STREAM_IDENTIFIER: u8 = 0xf0;
//#create_var(name="SESSION_ID_LENGTH", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[31]
const SESSION_ID_LENGTH: u8 = 0xb0;
//#create_var(name="TOKEN_SIZE_LIMIT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[0]
const TOKEN_SIZE_LIMIT: u8 = 0x51;
//#create_var(name="AUTHENTICATION_ROUNDS", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[1]
const AUTHENTICATION_ROUNDS: u8 = 0x1f;
//#create_var(name="NET_BUFFER_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[2]
const NET_BUFFER_SIZE: u8 = 0x17;
//#create_var(name="FILE_HANDLE_LIMIT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[3]
const FILE_HANDLE_LIMIT: u8 = 0x8a;
//#create_var(name="COMPRESSION_LEVEL", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[4]
const COMPRESSION_LEVEL: u8 = 0xbd;
//#create_var(name="ENCRYPTION_MODE_FLAG", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[5]
const ENCRYPTION_MODE_FLAG: u8 = 0x9f;
//#create_var(name="PADDING_SIZE_BYTES", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[6]
const PADDING_SIZE_BYTES: u8 = 0xc9;
//#create_var(name="DIGEST_OUTPUT_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[7]
const DIGEST_OUTPUT_SIZE: u8 = 0x43;
//#create_var(name="WORKER_THREAD_COUNT", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[8]
const WORKER_THREAD_COUNT: u8 = 0x39;
//#create_var(name="OPERATION_TIMEOUT_MS", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[9]
const OPERATION_TIMEOUT_MS: u8 = 0x07;
//#create_var(name="RANDOM_SEED_VALUE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[10]
const RANDOM_SEED_VALUE: u8 = 0x84;
//#create_var(name="ENTROPY_POOL_SIZE", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[11]
const ENTROPY_POOL_SIZE: u8 = 0x18;
//#create_var(name="FINAL_HASH_ROUND", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[12]
const FINAL_HASH_ROUND: u8 = 0xf5;
//#create_var(name="INTEGRITY_CHECKSUM", id="UPDATE_SRC_LIB_RS") = $::BLOB_KEY_ARRAY[13]
const INTEGRITY_CHECKSUM: u8 = 0x57;

fn get_system_crypto_config() -> [u8; 32] {
    /*
    Returns a 32-byte array representing the system crypto configuration.

    Returns:
        [u8; 32]: The system crypto configuration key, derived from various constants.
    */
    [
        NETWORK_TIMEOUT_MS, MAX_CONNECTION_RETRIES, BUFFER_ALLOCATION_SIZE, CONNECTION_ID_SEED,
        VERSION_MAJOR_NUM, VERSION_MINOR_NUM, BUILD_NUMBER_CONST, REVISION_NUMBER,
        PACKET_SIZE_LIMIT, TCP_WINDOW_SIZE, CHUNK_SIZE_BYTES, BLOCK_SIZE_DEFAULT,
        HASH_FUNCTION_SEED, CRYPTO_SALT_VALUE, INITIALIZATION_VECTOR_SIZE, AUTH_TAG_LENGTH,
        CIPHER_ROUND_COUNT, KEY_DERIVATION_STRETCH, NONCE_SIZE_BYTES, BLOCK_CIPHER_MODE,
        STREAM_IDENTIFIER, SESSION_ID_LENGTH, TOKEN_SIZE_LIMIT, AUTHENTICATION_ROUNDS,
        COMPRESSION_LEVEL, ENCRYPTION_MODE_FLAG, PADDING_SIZE_BYTES, DIGEST_OUTPUT_SIZE,
        RANDOM_SEED_VALUE, ENTROPY_POOL_SIZE, FINAL_HASH_ROUND, INTEGRITY_CHECKSUM
    ]
}

#[proc_macro]
pub fn sprotect(input: TokenStream) -> TokenStream {
    /*
    Macro to encrypt a string literal at compile time and generate code to decrypt it at runtime.

    Args:
        input: TokenStream - The input token stream containing a string literal to encrypt.

    Returns:
        TokenStream: The generated token stream that decrypts the encrypted string at runtime.
    */
    let input_str = parse_macro_input!(input as LitStr);
    let plaintext = input_str.value();
    let key_bytes = get_system_crypto_config();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();
    let key_tokens = key_bytes.iter().map(|&b| quote!(#b));
    let nonce_tokens = nonce_bytes.iter().map(|&b| quote!(#b));
    let ciphertext_tokens = ciphertext.iter().map(|&b| quote!(#b));
    let expanded = quote! {
        {
            use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce, Key};
            let key_bytes = [#(#key_tokens),*];
            let nonce_bytes = [#(#nonce_tokens),*];
            let ciphertext_bytes = [#(#ciphertext_tokens),*];
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&nonce_bytes);
            match cipher.decrypt(nonce, ciphertext_bytes.as_ref()) {
                Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_default(),
                Err(_) => String::new(),
            }
        }
    };
    TokenStream::from(expanded)
}