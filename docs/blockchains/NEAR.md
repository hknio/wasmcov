# Using `wasmcov` with NEAR Protocol

This guide explains how to use `wasmcov` to generate code coverage reports for NEAR Protocol smart contracts. Code coverage helps identify areas of code that are executed during testing, making it a valuable tool for ensuring the reliability and quality of your contracts.

## 1. Custom Profile in `.cargo/config`

To configure a custom profile for code coverage, add the following section to your `.cargo/config` file. This profile ensures that your code is optimized for coverage analysis:

```toml
[profile.coverage]
inherits = "release"
strip = true
codegen-units = 1
opt-level = "z"
debug = false
panic = "abort"
overflow-checks = true
```

## 2. Set Up Custom NEAR Sandbox Local Network

Before running tests with coverage, you need to set up a custom NEAR Protocol sandbox with modified parameters to allow for higher gas limits. Use the following commands:

Initialize the sandbox:

```bash
./neard --home ./target/wasmcov/.near init
```

Run the sandbox:

```bash
./neard --home ./target/wasmcov/.near run
```

## 3. Connect NEAR Workspaces to the Custom Sandbox

In your tests, you need to connect NEAR Workspaces to the custom test sandbox. Replace the standard `near_workspaces::sandbox().await?` code with the following:

```rust
let worker = near_workspaces::sandbox()
    .rpc_addr("http://localhost:3030")
    .validator_key(ValidatorKey::HomeDir(PathBuf::from("./target/wasmcov/.near")))
    .await?;
```

## 4. Patch NEAR-SDK to Modified near_bindgen Version

To work with coverage, you need to patch the NEAR-SDK with a modified `near_bindgen`. Add the following section to your `.cargo/config` file:

```toml
[profile.coverage.patch.crates-io]
near-sdk = { git = "https://github.com/njelich/near-sdk-rs", rev = "468b5e585dc0ce0cee3d56f446c4a6054fb08f00" }
```

## 5. Collect Coverage Data in Tests

Ensure that your tests capture coverage data. You can extract coverage information from the last log, which contains data on every function call, and write it to a `.profraw` file. Use the following code snippet:

```rust
let coverage = &context.contract.method_call().await?;
let coverage: Vec<u8> = near_sdk::base64::decode(&coverage.logs.last().unwrap()).unwrap();
std::fs::write(format!("../profraw/{random_file_name}.profraw"), coverage).unwrap();
```

## 6. Build Code with Coverage Instrumentation

Prepare your code for coverage analysis by building it with coverage instrumentation. Use the following commands to build your contract and generate the necessary files:

```bash
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime -Zlocation-detail=none --emit=llvm-ir"

rustup target add wasm32-unknown-unknown

cargo build -p contract --target wasm32-unknown-unknown --profile=coverage

cp ./target/wasm32-unknown-unknown/coverage/contract.wasm res/contract_coverage.wasm
cp ./target/wasm32-unknown-unknown/coverage/deps/contract.ll res/contract_coverage.ll

perl -i -p0e 's/(^define[^\n]*\n).*?^}\s*$/$1start:\n  unreachable\n}\n/gms' res/contract_coverage.ll

clang-17 res/contract_coverage.ll -o res/contract_coverage.o -Wno-override-module -c
```

## 7. Run External Tests Using contract_coverage.wasm

With your contract built for coverage, run your external tests using the `contract_coverage.wasm` file.

## 8. Merge Coverage Data and Generate the Report

After running the tests, you need to merge the coverage data from the `.profraw` files and generate a code coverage report. Use the following commands:

```bash
llvm-profdata merge -sparse profraw-tests/*.profraw -o output.profdata

llvm-cov-17 show --instr-profile=output.profdata res/contract_coverage.o --format=html -output-dir=wasmcov/
```

By following these steps, you can effectively use `wasmcov` to generate code coverage reports for your NEAR Protocol smart contracts. This coverage analysis can help you identify and improve the test coverage of your contracts, ensuring their reliability and correctness.