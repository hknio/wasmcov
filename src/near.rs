#[macro_export]
macro_rules! add_coverage {
    () => {
        extern "C" {
            fn sandbox_capture_coverage(
                binary_len: u64,
                binary_ptr: u64,
                value_len: u64,
                value_ptr: u64,
            );
        }

        #[no_mangle]
        pub unsafe extern "C" fn capture_coverage() {
            const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
            let mut coverage = vec![];
            wasmcov::minicov::capture_coverage(&mut coverage).unwrap();
            sandbox_capture_coverage(
                BINARY_NAME.len() as u64,
                BINARY_NAME.as_ptr() as u64,
                coverage.len() as u64,
                coverage.as_ptr() as u64,
            );
        }
    };
}

pub use add_coverage;
