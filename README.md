# Wasmcov

Wasmcov comprises a Rust library and an associated binary that offer a range of utilities for coverage analysis of WebAssembly (Wasm) modules. This tool empowers you to gather and scrutinize code coverage data while executing Wasm modules. Wasmcov stands out for its adaptability and seamless integration into your Wasm projects, serving as a potent instrument to elevate the standard of your Wasm applications.

## Installation

Include `wasmcov` as a dependency in your `Cargo.toml` file. Tailor feature flags according to your environment:

```toml
[dependencies.wasmcov]
version = "0.1"
features = ["near"]
```

Alternatively, install the binary directly using `cargo install`:

```bash
cargo install wasmcov
```

## Coverage Data Generation

To incorporate code coverage instrumentation into your WASM binary, utilize the `capture_coverage` utility. This functionality streamlines LLVM instrumentation coverage for Rust projects. Here’s a step-by-step guide:

### Integrate the following function into your code generation pipeline

```rust
#[no_mangle]
unsafe extern "C" fn generate_coverage() {
    let mut coverage = vec![];
    wasmcov::capture_coverage(&mut coverage).unwrap();
    // Invoke a function (e.g., `your_custom_save_coverage_function`) to preserve the coverage data or utilize `println!` for debugging.
}
```

Automating coverage data generation post-execution is critical. Manually inserting calls to `generate_coverage` for each function is impractical. Implementation can vary depending on your platform and might pose challenges when execution encounters failures, such as panics.

For instance, modify the function responsible for invoking WASM functions to call `generate_coverage` after each function call. Ensure this modification remains platform-independent.

Find examples of such implementations for different platforms [here](https://github.com/hknio/wasmcov-near-sdk-rs/compare/hknio:wasmcov-near-sdk-rs:55020df8e99057815685b75b70955cb79a9dfe28...wasmcov) and [here](https://github.com/radixdlt/radixdlt-scrypto/pull/1640/files).

### Managing coverage data in tests

After extracting the coverage data (via logs, storage, file writes, etc.), invoke `wasmcov::write_profraw(coverage_data)` to save it to a `.profraw` file. This file becomes the basis for generating a `.profdata` file.

**Once the coverage data reaches `wasmcov::write_profraw`, the library handles the rest automatically.**

An example flow was implemented for the NEAR protocol, passing coverage data through logs:

```rust
let mut coverage = vec![];
unsafe {
    // Note: This function isn't thread-safe! Employ a lock if necessary.
    wasmcov::capture_coverage(&mut coverage).unwrap();
};
let base64_string = near_sdk::base64::encode(coverage);

::near_sdk::env::log_str(&base64_string);
```

Then, after a function call, retrieve it using `wasmcov::near::near_coverage()`:

```rust
let result = manager
    .call(self.id(),"function_name")
    .args_json(args)
    .deposit(1)
    .transact()
    .await?
    .into_result()?;

// Extract the coverage data from the last log
let coverage: Vec<u8> = near_sdk::base64::decode(&result.logs().last().unwrap()).unwrap();
wasmcov::write_profraw(coverage);
```

## Usage (binary)

```bash
eval $(cargo wasmcov setup)

# Your build command
cargo build -p contract --target wasm32-unknown-unknown


# Set up and run your tests
cargo wasmcov post_build # Find the compiled .wasm files
make external_tests # Run your external tests

cargo wasmcov finalize
```

## Usage (library)

In Rust code, Wasmcov is invoked in the following sequence:

```rust
wasmcov::setup(None); // Or specify the path to the wasmcov directory

// Execute your build command here (it utilizes the environment setup created by wasmcov::setup)
wasmcov::run_command("cargo build -p contract --target wasm32-unknown-unknown");

// Set up and run your tests
// Obtain compiled wasm file paths via wasmcov::post_build() > Vec<PathBuf> function
let wasm_file_paths = wasmcov::post_build();
// Copy these files to your intended location
std::fs::copy(wasm_file_paths[0], "your_new_path.wasm").unwrap(); // and so on
// Run your tests
wasmcov::run_command("your external test command");

// Perform coverage analysis
wasmcov::finalize();
```

## License

This repository is distributed under the terms of the Apache License (Version 2.0). Refer to [LICENSE](LICENSE) for details.

## Maintainer

This repository is currently managed by [Noah Jelich](https://www.linkedin.com/in/njelich/) from [Hacken](https://hacken.io/). Please don't hesitate to reach out for any queries or concerns.

## Contributing

Contributions are encouraged! Employ the `cargo build` command to build the project. Note: during testing, deactivate parallelism by using the `--test-threads=1` flag. This ensures environment variables remain unaffected by other tests.

For convenience, utilize the shorthands `make build` and `make test` for building and testing the project, respectively.