# Code Coverage for Radix

### Step 1: Adding code coverage instrumentation to WASM binary

[Minicov](https://github.com/Amanieu/minicov/) provides an easy way to add LLVM instrumentation coverage to Rust projects. First, include minicov in your dependencies:

```rust
[dependencies]
minicov = "0.3"
```

Then, add the function below to your code:
```rust
#[no_mangle]
pub unsafe extern "C" fn dump_coverage() -> types::Slice {
    let mut coverage = vec![];
    minicov::capture_coverage(&mut coverage).unwrap();
    engine::wasm_api::forget_vec(coverage)
}
```

Set the RUSTFLAGS to:
```bash
RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime --emit=llvm-ir"
```
And then compile your project in debug/develop mode (or as release with LTO disabled). Note: `minicov` doesn't mention `--emit=llvm-ir`, but it's needed later.

### Step 2: Automatic coverage data generation

To generate coverage data, we need the VM to call dump_coverage after every execution and store generated coverage data. Implementing this might differ based on your platform and could be challenging, especially when execution fails, e.g., due to panics.

For instance, in the [radix wasmi](https://github.com/radixdlt/radixdlt-scrypto/blob/v1.0.0/radix-engine/src/vm/wasm/wasmi.rs#L1516) VM, the function that invokes the WASM function is `invoke_export`. Here's how to modify it to call `dump_coverage` after each call:
```rust
fn invoke_export<'r>(
    &mut self,
    func_name: &str,
    args: Vec<Buffer>,
    runtime: &mut Box<dyn WasmRuntime + 'r>,
) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
    ...
    let func = self.get_export_func(func_name).unwrap();
    let input: Vec<Value> = args
        .into_iter()
        .map(|buffer| Value::I64(buffer.as_i64()))
        .collect();
    let mut ret = [Value::I64(0)];

    let call_result = func
        .call(self.store.as_context_mut(), &input, &mut ret)
        .map_err(|e| {
            let err: InvokeError<WasmRuntimeError> = e.into();
            err
        });

    // store result of execution to be returned after call to dump_coverage
    let result = match call_result {
        Ok(_) => { 
            match i64::try_from(ret[0]) {
                Ok(ret) => read_slice(
                    self.store.as_context_mut(),
                    self.memory,
                    Slice::transmute_i64(ret),
                ),
                _ => Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer)),
            }
        },
        Err(err) => {
            Err(err)
        }
    };

    // now it checks if there's dump_coverage function in the code
        if let Ok(dump_coverage) = self.get_export_func("dump_coverage") {
            if let Ok(blueprint_buffer) = runtime.actor_get_blueprint_name() {
                let blueprint_name =
                    String::from_utf8(runtime.buffer_consume(blueprint_buffer.id()).unwrap())
                        .unwrap();

                let mut ret = [Value::I64(0)];
                dump_coverage
                    .call(self.store.as_context_mut(), &[], &mut ret)
                    .unwrap();
                let coverage_data = read_slice(
                    self.store.as_context_mut(),
                    self.memory,
                    Slice::transmute_i64(i64::try_from(ret[0]).unwrap()),
                )
                .unwrap();
                save_coverage_data(&blueprint_name, &coverage_data);
            }
        }

    // return the result of the call
    result
}
```

The function `save_coverage_data` has the following implementation:
```rust
pub fn save_coverage_data(blueprint_name: &String, coverage_data: &Vec<u8>) {
    if let Some(dir) = env::var_os("COVERAGE_DIRECTORY") {
        let mut file_path = Path::new(&dir).to_path_buf();
        file_path.push(blueprint_name);

        // Check if the blueprint directory exists, if not create it
        if !file_path.exists() {
            // error is ignored because when multiple tests are running it may fail
            fs::create_dir(&file_path).ok();
        }

        // Write .profraw binary data
        let file_name = hash(&coverage_data);
        let file_name: String = file_name.0[..16]
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        file_path.push(format!("{}.profraw", file_name));
        let mut file = File::create(file_path).unwrap();
        file.write_all(&coverage_data).unwrap();
    }
}
```

After these changes and setting `COVERAGE_DIRECTORY` environmental variable, running your code will generate `.profraw` files containing coverage data.

### Step 3: Parsing raw coverage data

Executing your code will result in one or more `.profraw` files. Merge these into a single `.profdata` file:
```bash
llvm-profdata merge -sparse *.profraw -o coverage.profdata
```

Ensure `llvm-profdata` matches your Rust version. Check your Rust version with `rustc --version --verbose` and compare the LLVM version. If they differ, [install the correct LLVM version](https://apt.llvm.org/) and adjust the command accordingly (in my case it is `llvm-profdata-17`).

### Step 4: Generating the coverage report

Now with the `coverage.profdata` we're ready to generate coverate report. First, let's try to use the following command:
```bash
llvm-cov-17 show --instr-profile=coverage.profdata our_binary.wasm
```
Unfortunetlly, it's not going to work because `our_binary.wasm` doesn't have `__llvm_covmap` section required by `llvm-cov` to generate coverage report.
```bash
$ llvm-cov-17 show --instr-profile=coverage.profdata our_binary.wasm
error: Failed to load coverage: 'our_binary.wasm': No coverage data found
```

To fix this, utilize the Intermediate Representation file `our_binary.ll` (located in `target/wasm32-unknown-unknown/debug/deps`) produced by the `--emit=llvm-ir` flag. Here's the workaround:
```bash
clang-17 our_binary.ll -Wno-override-module -c
llvm-cov-17 show --instr-profile=coverage.profdata our_binary.o --format=html -output-dir=coverage/
```

However, this approach is not going to work if the project is using instructions only available in WebAssembly, like `memory.size` or `memory.grow`. In that case we need to do an extra step. Fortunately, instructions in `our_binary.ll` are not needed to generate code coverage report, so you can just remove all of them by using the following command with regular expression:
```bash
perl -i -p0e 's/(^define[^\n]*\n).*?^}\s*$/$1start:\n  unreachable\n}\n/gms' our_binary.ll
```

Then, execute the previously mentioned `clang-17` and `llvm-cov-17` commands. If problems persist, try disabling link-time optimizations (LTO) during your project build.

## Implementation in radixdlt-scrypto

This feature was implemented in `radixdlt-scrypto` in [PR #1640](https://github.com/radixdlt/radixdlt-scrypto/pull/1640). 

## Remarks

While this method utilizes some workarounds, it's functional. A more streamlined approach would involve integrating WASM file support into `llvm-cov`. However, this would be more challenging and time-consuming. Thus, this solution is recommended for now.
