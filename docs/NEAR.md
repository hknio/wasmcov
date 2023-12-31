# Using `wasmcov` with NEAR Protocol

This guide explains how to use `wasmcov` to generate code coverage reports for NEAR Protocol smart contracts. Code coverage helps identify areas of code that are executed during testing, making it a valuable tool for ensuring the reliability and quality of your contracts.

## 1. Patch NEAR-SDK to Modified near_bindgen Version

To work with coverage, you need to patch the NEAR-SDK with a modified `near_bindgen`. Add the following section to your `Cargo.toml` file:

```toml
[patch.crates-io]
near-sdk = { git = "https://github.com/hknio/near-sdk-rs", rev = "468b5e585dc0ce0cee3d56f446c4a6054fb08f00" }
```

## 2. Build Code with Coverage Instrumentation

Prepare your code for coverage analysis by building it with coverage instrumentation. Use the following commands to build your contract and generate the necessary files:

```bash
eval $(cargo wasmcov setup) // Sets up the CARGO_TARGET_DIR and WASMCOV_DIR environment variables

# Your build command
cargo build -p contract --target wasm32-unknown-unknown
```

## 3. Set Up Custom NEAR Sandbox Local Network

Before running tests with coverage, you need to set up a custom NEAR Protocol sandbox with modified parameters to allow for higher gas limits. Use the following commands. You can find the `neard` precompiled binary [in this repo](https://github.com/hknio/wasmcov/tree/main/bin) or [compile it yourself](https://github.com/hknio/wasmcov-nearcore/tree/1.36.0).

Initialize the sandbox:

```bash
./neard --home $WASMCOV_DIR/bin/.near init
```

Run the sandbox:

```bash
./neard --home $WASMCOV_DIR/bin/.near run
```

## 4. Connect NEAR Workspaces to the Custom Sandbox

In your tests, you need to connect NEAR Workspaces to the custom test sandbox. Replace the standard `near_workspaces::sandbox().await?` code with the following:

```rust
use wasmcov::dir::get_wasmcov_dir;

...
let worker = near_workspaces::sandbox()
    .rpc_addr("http://localhost:3030")
    .validator_key(ValidatorKey::HomeDir(get_wasmcov_dir().join("bin").join(".near")))
    .await?;
```

## 5. Collect Coverage Data in Tests

Ensure that your tests capture coverage data. You can extract coverage information from the last log, which contains data on every function call, and write it to a `.profraw` file. Use the following code snippet:

```rust
use wasmcov::near::near_coverage;

fn main() {
    let contract: near_workspaces::Contract = near_workspaces::Contract::new();
    let result = contract.view("get_coverage").await?;
    ...
    near_coverage(&result.logs());
}
```

## 6. Run External Tests Using the Compiled Contract

You will be able to find the compiled `.wasm` files in the `wasmcov` directory using `cargo post_build`. Copy these files to your intended location and run your external tests.

## 7. Merge Coverage Data and Generate the Report

After running the tests, you need to merge the coverage data from the `.profraw` files and generate a code coverage report. Use the following commands:

```bash
cargo wasmcov finalize
```

By following these steps, you can effectively use `wasmcov` to generate code coverage reports for your NEAR Protocol smart contracts. This coverage analysis can help you identify and improve the test coverage of your contracts, ensuring their reliability and correctness.
