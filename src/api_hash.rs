pub use api_hash_macro::{api_hash, hash_key};

#[macro_export]
macro_rules! api_resolve {
    ($dll:literal, $func:literal) => {{
        use $crate::win_internals::{get_module_by_hash, get_proc_by_hash};
        
        const DLL_HASH: u64 = $crate::api_hash::api_hash!($dll);
        const FUNC_HASH: u64 = $crate::api_hash::api_hash!($func);
        const HASH_KEY: u64 = $crate::api_hash::hash_key!();
        
        unsafe {
            let module = get_module_by_hash(DLL_HASH, HASH_KEY)
                .expect(concat!("Failed to find module: ", $dll));
            let func = get_proc_by_hash(module, FUNC_HASH, HASH_KEY)
                .expect(concat!("Failed to find function: ", $func));
            func
        }
    }};
}
