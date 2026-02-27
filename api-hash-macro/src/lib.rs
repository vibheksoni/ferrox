extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::quote;
use rand::Rng;
use syn::{parse_macro_input, LitStr};

lazy_static! {
    static ref HASH_KEY: u64 = {
        let mut rng = rand::thread_rng();
        rng.gen_range(1000..10000)
    };
}

fn djb2_hash(input: &str, key: u64) -> u64 {
    input.bytes().fold(key, |hash, byte| {
        ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64)
    })
}

#[proc_macro]
pub fn api_hash(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as LitStr).value();
    
    let hash_input = if input.ends_with(".DLL") || input.ends_with(".dll") {
        input.to_uppercase()
    } else {
        input
    };
    
    let hash = djb2_hash(&hash_input, *HASH_KEY);
    
    TokenStream::from(quote! {
        #hash
    })
}

#[proc_macro]
pub fn hash_key(_item: TokenStream) -> TokenStream {
    let key = *HASH_KEY;
    
    TokenStream::from(quote! {
        #key
    })
}
