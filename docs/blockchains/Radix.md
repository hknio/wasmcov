# Code Coverage for WebAssembly

This repository shows how to generate a code coverage report for WebAssembly programs written in Rust, specifically for smart contracts in blockchain protocols. As of October 18, 2023, generating such reports wasn't possible for blockchain protocols using the WebAssembly VM. This repository aims to guide protocols using the WebAssembly VM on implementing code coverage functionality. While this technique focuses on blockchain protocols, it's applicable to any project using the WASM VM.

## 1. Adding code coverage instrumentation to WASM binary

[Minicov](https://github.com/Amanieu/minicov/) provides an easy way to add LLVM instrumentation coverage to Rust projects. First, include minicov in your dependencies:

```rust
[dependencies]
minicov = "0.3"
```

Then, add the function below to your code:
```rust
#[no_mangle]
unsafe extern "C" fn dump_coverage() {
    let mut coverage = vec![];
    minicov::capture_coverage(&mut coverage).unwrap();
    ScryptoVmV1Api::dump_coverage(coverage); // function saving coverage data, you can also use println!
}
```

Set the RUSTFLAGS to:
```bash
RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime --emit=llvm-ir"
```
And then compile your project in debug/develop mode (or as release with LTO disabled). Note: `minicov` doesn't mention `--emit=llvm-ir`, but it's needed later.

One challenge is saving the `minicov::capture_coverage` data to a file. Most blockchain VMs don't have this capability. In this example I introduced a new native function, `ScryptoVmV1Api::dump_coverage`, to handle this:
```rust
fn dump_coverage(&mut self, data: Vec<u8>) -> Result<(), RuntimeError> {        
    if let Some(dir) = env::var_os("COVERAGE_DIRECTORY") {
        // in this case blueprint_id is the name of project
        let blueprint_id = self.current_actor()
            .blueprint_id()
            .ok_or(RuntimeError::SystemError(SystemError::NoBlueprintId))?;
        let mut file_path = Path::new(&dir).to_path_buf();
        file_path.push(blueprint_id.blueprint_name);
        // Check if the directory exists, if not create it
        if !file_path.exists() {
            fs::create_dir(&file_path).unwrap();
        }
        // file name is hash of its data, so there's no chance of collison
        let file_name = hash(&data);
        let file_name: String = file_name.0[..16]
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        file_path.push(format!("{}.profraw", file_name));
        let mut file = File::create(file_path).unwrap();
        file.write_all(&data).unwrap();
    }
    Ok(())
}
```

## 2. Automatic coverage data generation

To generate coverage data, we need the VM to call `dump_coverage` post-execution. Manually adding this to every function isn't practical. Implementing this might differ based on your platform and could be challenging, especially when execution fails, e.g., due to panics.

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
    if let Ok(func) = self.get_export_func("dump_coverage") {
        // the code contains dump_coverage, we call it with no arguments
        match func.call(self.store.as_context_mut(), &vec![], &mut vec![]).map_err(|e| {
            let err: InvokeError<WasmRuntimeError> = e.into();
            err
        }) {
            Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented)) => {
                // code is not really executed, this error can be ignored
            }
            Err(err) => {
                panic!("dump_coverage failed with error {err:?}");
            }
            Ok(_) => {}
        };
    }

    // return the result of the call
    result
}
```

After these changes and setting `COVERAGE_DIRECTORY` environmental variable, running your code will generate `.profraw` files containing coverage data.

## 3. Parsing raw coverage data

Executing your code will result in one or more `.profraw` files. Merge these into a single `.profdata` file:
```bash
llvm-profdata merge -sparse *.profraw -o coverage.profdata
```

Ensure `llvm-profdata` matches your Rust version. Check your Rust version with `rustc --version --verbose` and compare the LLVM version. If they differ, [install the correct LLVM version](https://apt.llvm.org/) and adjust the command accordingly (in my case it is `llvm-profdata-17`).

### 4. Generating the coverage report

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

## Upcoming Implementations

In the coming days, we'll add an example code coverage implementations for [radixdlt-scrypto](https://github.com/radixdlt/radixdlt-scrypto/) and [nearcore](https://github.com/near/nearcore).

## Remarks

While this method utilizes some workarounds, it's functional. A more streamlined approach would involve integrating WASM file support into `llvm-cov`. However, this would be more challenging and time-consuming. Thus, this solution is recommended for now.
