# Wasmcov

Wasmcov is a Rust library and accompanying binary that provides a set of helpers for coverage analysis of WebAssembly (Wasm) modules. It allows you to collect and analyze code coverage data when running Wasm modules. Wasmcov is designed to be flexible and easy to integrate into your Wasm projects, making it a powerful tool for improving the quality of your Wasm applications.

## Installation

Add `wasmcov` as a dependency in your `Cargo.toml`, setting feature flags based on your environment:

```toml
[dependencies.wasmcov]
version = "0.1"
features = ["near"]
```

Or to use the binary directly, install it using `cargo install`:

```bash
cargo install wasmcov
```

## Coverage Data Generation

To add code coverage instrumentation to your WASM binary, you can use the `capture_coverage` utility, which facilitates LLVM instrumentation coverage for Rust projects. Here's how to do it:

1. Add the following function to your code:

   ```rust
   #[no_mangle]
   unsafe extern "C" fn generate_coverage() {
       let mut coverage = vec![];
       wasmcov::capture_coverage(&mut coverage).unwrap();
       // Call a function (e.g., `your_custom_save_coverage_function`) to save the coverage data or use `println!` for debugging.
   }
   ```

2. Setup automatic coverage data generation

Generating coverage data automatically after execution is crucial. Manually adding calls to `generate_coverage` for each function is impractical. Implementing this can vary based on your platform and may be challenging when execution fails, such as during panics.

For example, you can modify the function responsible for invoking WASM functions to call `generate_coverage` after each function call. Ensure the modification is platform-independent.

Examples of implementation for different platforms can be found [here](https://github.com/hknio/wasmcov-near-sdk-rs/compare/hknio:wasmcov-near-sdk-rs:55020df8e99057815685b75b70955cb79a9dfe28...wasmcov) and [here](https://github.com/radixdlt/radixdlt-scrypto/pull/1640/files).

3. Handle coverage data writing in tests

Once you've extracted the coverage data (either through logs, storage, file writes etc) you should invoke `wasmcov::write_profraw(coverage_data)` to write the coverage data to a `.profraw` file. This file can then be used to generate a `.profdata` file. 

**Once the coverage data is passed to `wasmcov::write_profraw`, everything else is automatically handled by the library.**

An example of the flow was implemented for NEAR protocol - the coverage data is passed through logs:

```rust
     let mut coverage = vec![];
            unsafe {
                // Note that this function is not thread-safe! Use a lock if needed.
                minicov::capture_coverage(&mut coverage).unwrap();
            };
            let base64_string = near_sdk::base64::encode(coverage);

            ::near_sdk::env::log_str(&base64_string);
```

And then picked up using `wasmcov::near::near_coverage()` after a function call:

```rust
   let result = manager
            .call(self.id(), "function_name")
            .args_json(args)
            .deposit(1)
            .transact()
            .await?
            .into_result()?;

    wasmcov::near::near_coverage(result.logs());
```

## Usage (binary)

```bash
eval $(cargo wasmcov setup)

# Your build command
cargo build -p contract --target wasm32-unknown-unknown

# Move compiled wasm files to where you need them, find them using:
cargo wasmcov post_build

make external_tests

cargo wasmcov finalize
```

## Usage (library)

Wasmcov is called in rust code in the following order

```rust
wasmcov::setup(None); // Or path to wasmcov directory

// Run your build command here (it will use env setup created by wasmcov::setup)
wasmcov::run_command("cargo build -p contract --target wasm32-unknown-unknown");

// Setup your tests and run them
// The compiled wasm file paths can be found using the wasmcov::post_build() > Vec<PathBuf> function
let wasm_file_paths = wasmcov::post_build();
// Copy all the files to where you need them
std::fs::copy(wasm_file_paths[0], "your_new_path.wasm").unwrap(); // etc etc
// Run your tests
wasmcov::run_command("your external test command");

// Run the coverage analysis
wasmcov::finalize();
```

## License

This repository is distributed under the terms of the Apache License (Version 2.0). See [LICENSE](LICENSE) for details.

## Maintainer

This repository is currently maintained by [Noah Jelich](https://www.linkedin.com/in/njelich/) from [Hacken](https://hacken.io/). Feel free to contact me with any questions or concerns.

## Contributing

Contributions are welcome! Use the `cargo build` command to build the project. Note: when testing, make sure to disable paralleliism by using the `--test-threads=1` flag. This is required to ensure that environment variables are not overwritten by other tests.

For ease of use, you can use the shorthands `make build` and `make test` to build and test the project, respectively.