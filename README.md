# Wasmcov

[Crates.io](https://crates.io/crates/wasmcov)
[License](https://opensource.org/licenses/MIT)
[Documentation](https://hknio.github.io/wasmcov/)

Wasmcov is a Rust library that provides a set of helpers for coverage analysis of WebAssembly (Wasm) modules. It allows you to collect and analyze code coverage data when running Wasm modules. Wasmcov is designed to be flexible and easy to integrate into your Wasm projects, making it a powerful tool for improving the quality of your Wasm applications.

Currently supports only NEAR Protocol.

## Installation

Add `wasmcov` as a dependency in your `Cargo.toml`, setting feature flags based on your environment:

```toml
[dependencies.wasmcov]
version = "0.0.1"
features = ["near"]
```

Follow the [NEAR setup instructions](blockchains/NEAR.md) to setup the environment for running Wasm modules. The `near` feature flag is required for NEAR Protocol coverage.

## Usage

- The `WASMCOV_DIR` environment variable is used to set the directory where the coverage data will be stored. If the env var is not set, the default directory is `./wasmcov`.
- The `wasmcov/profraw` directory can be purged between runs to reset coverage. A CLI utility will be added in the future to make this easier. 


### NEAR Protocol
```rust
use wasmcov::{near_coverage};

fn main() {
    let contract: near_workspaces::Contract = near_workspaces::Contract::new();
    let result = contract.view("get_coverage").await?;
    ...
    near_coverage(&result.logs());
}
```

## License

This repository is distributed under the terms of both the MIT license and the Apache License (Version 2.0). See [LICENSE](LICENSE) and [LICENSE-APACHE](LICENSE-APACHE) for details.## License

