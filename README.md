# Wasmcov

Wasmcov comprises a Rust library and an associated binary that offer a range of utilities for coverage analysis of WebAssembly (Wasm) modules. This tool empowers you to gather and scrutinize code coverage data while executing Wasm modules. Wasmcov stands out for its adaptability and seamless integration into your Wasm projects, serving as a potent instrument to elevate the standard of your Wasm applications.

If you would like to apply this method to non-Rust WASM binaries, see our [General guide](https://hknio.github.io/wasmcov/docs/General).

## Installation

Install the `cargo-wasmcov` command by running:

```bash
cargo install wasmcov
```

## Usage

`cargo-wasmcov` provides several subcommands to build, run, test, and generate coverage reports for your WASM projects.

### Integrating Coverage Capture

Before using `cargo-wasmcov`, you need to integrate the coverage capture function into your WebAssembly project:

1. Add `wasmcov` as a dependency in your `Cargo.toml`:
   ```toml
   wasmcov = "0.2"
   ```

2. Add the following function to your WebAssembly project:

   ```rust
   #[cfg(target_family = "wasm")]
   #[no_mangle]
   pub unsafe extern "C" fn capture_coverage() {
       const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
       let mut coverage = vec![];
       wasmcov::minicov::capture_coverage(&mut coverage).unwrap();
       // Invoke a function to preserve the coverage data or use `println!` for debugging.
   }
   ```

   For NEAR Protocol projects, use this macro in `lib.rs` of your smart contract:

   ```rust
   #[cfg(target_family = "wasm")]
   wasmcov::near::add_coverage!();
   ```

### Build

Build your project with WASM coverage instrumentation:

```bash
cargo wasmcov build [-- <additional cargo arguments>]
```

Example:
```bash
cargo wasmcov build -- --all --target wasm32-unknown-unknown --release
```

### Run

Run your project with WASM coverage:

```bash
cargo wasmcov run [--near <VERSION>] [-- <additional cargo arguments>]
```

The `--near` option allows you to specify a NEAR sandbox version (e.g., 1.35.0) if needed. It is required for near projects.

### Test

Run tests with WASM coverage:

```bash
cargo wasmcov test [--near <VERSION>] [-- <additional cargo arguments>]
```

The `--near` option allows you to specify a NEAR sandbox version (e.g., 1.35.0) if needed. It is required for near projects.

### Generate Coverage Report

Generate a coverage report:

```bash
cargo wasmcov report [-- <additional llvm-cov arguments>]
```

This command will process all collected coverage data and generate reports for each target.

### Clean

Clean coverage data:

```bash
cargo wasmcov clean [--all]
```

Use the `--all` flag to remove the entire wasmcov directory content.

## Notes

- The tool uses the nightly Rust toolchain for building and running.
- Coverage reports are generated using LLVM coverage tools.
- For NEAR-specific projects, you must specify the NEAR sandbox version using the `--near` option with the `run` and `test` subcommands.

## License

This repository is distributed under the terms of the Apache License (Version 2.0). Refer to [LICENSE](LICENSE) for details.

When using the Wasmcov workaround (llvm-ir to .o file for linking) on its own, an attribution is required.

## Maintainer

This repository is currently managed by [Bartosz Barwikowski](https://www.linkedin.com/in/bbarwik/) from [Hacken](https://hacken.io/). It was originally created by [Noah Jelich](https://www.linkedin.com/in/njelich/), while the method itself was co-created with Bartosz. Please don't hesitate to reach out for any queries or concerns.

## Contributing

Contributions are encouraged! Employ the `cargo build` command to build the project. Note: during testing, deactivate parallelism by using the `--test-threads=1` flag. This ensures environment variables remain unaffected by other tests.

For convenience, utilize the shorthands `make build` and `make test` for building and testing the project, respectively.
