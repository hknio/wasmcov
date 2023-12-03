# Wasmcov

Wasmcov is a Rust library and accompanying binary that provides a set of helpers for coverage analysis of WebAssembly (Wasm) modules. It allows you to collect and analyze code coverage data when running Wasm modules. Wasmcov is designed to be flexible and easy to integrate into your Wasm projects, making it a powerful tool for improving the quality of your Wasm applications.

## Installation

Add `wasmcov` as a dependency in your `Cargo.toml`, setting feature flags based on your environment:

```toml
[dependencies.wasmcov]
version = "0.0.2"
features = ["near"]
```

Or to use the binary directly, install it using `cargo install`:

```bash
cargo install wasmcov
```

You will also need to modify your WASM runtime, this is highly specific to your runtime. See the [docs](docs/README.md) for more information.

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