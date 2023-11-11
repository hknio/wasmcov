# WebAssembly Code Coverage Guide

This guide aims to assist you in implementing code coverage for WebAssembly (WASM) programs, particularly for smart contracts within blockchain protocols. We'll break down the process into four essential steps while keeping the explanation focused on common challenges applicable to any environment.

Blockchain specific instructions:
- [NEAR Protocol](blockchains/NEAR.md)
- [Radix DLT](blockchains/Radix.md)

## Step 1: Adding Code Coverage Instrumentation to WASM Binary

To add code coverage instrumentation to your WASM binary, you can use the `minicov` library, which facilitates LLVM instrumentation coverage for Rust projects. Here's how to do it:

1. Include `minicov` in your project's dependencies:

   ```rust
   [dependencies]
   minicov = "0.3"
   ```

2. Add the following function to your code:

   ```rust
   #[no_mangle]
   unsafe extern "C" fn generate_coverage() {
       let mut coverage = vec![];
       minicov::capture_coverage(&mut coverage).unwrap();
       // Call a function (e.g., `your_custom_save_coverage_function`) to save the coverage data or use `println!` for debugging.
   }
   ```

3. Set the `RUSTFLAGS` environmental variable to include necessary flags:

   ```bash
   RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime --emit=llvm-ir"
   ```

4. Compile your project in debug/develop mode (or as a release with link-time optimizations (LTO) disabled).

One significant challenge in this step is saving the coverage data, as many blockchain VMs do not support file writes. The code example demonstrates a workaround by introducing a new native function to handle data storage.

## Step 2: Automatic Coverage Data Generation

Generating coverage data automatically after execution is crucial. Manually adding calls to `generate_coverage` for each function is impractical. Implementing this can vary based on your platform and may be challenging when execution fails, such as during panics.

For example, you can modify the function responsible for invoking WASM functions to call `generate_coverage` after each function call. Ensure the modification is platform-independent.

## Step 3: Parsing Raw Coverage Data

Executing your code will result in one or more `.profraw` files. To consolidate these into a single `.profdata` file, use the following command:

```bash
llvm-profdata merge -sparse *.profraw -o coverage.profdata
```

Ensure that the `llvm-profdata` version matches your Rust version. If they differ, you can [install the correct LLVM version](https://apt.llvm.org/) and adjust the command accordingly.

This guide outlines the process of implementing code coverage for WebAssembly programs within blockchain protocols. It focuses on common challenges that may occur in various environments, helping you to generate comprehensive code coverage reports for your WASM-based projects.

# Step 4: Generating the Coverage Report

Now that you have the `coverage.profdata` file, you are ready to generate a code coverage report. However, you may encounter an issue when using the following command:

```bash
llvm-cov-17 show --instr-profile=coverage.profdata our_binary.wasm
```

Unfortunately, this command won't work because `our_binary.wasm` lacks the `__llvm_covmap` section required by `llvm-cov` to generate a coverage report. You might receive an error like this:

```bash
$ llvm-cov-17 show --instr-profile=coverage.profdata our_binary.wasm
error: Failed to load coverage: 'our_binary.wasm': No coverage data found
```

To address this issue, you can utilize the Intermediate Representation file `our_binary.ll`, which is located in `target/wasm32-unknown-unknown/debug/deps` and is produced by the `--emit=llvm-ir` flag. Here's a workaround:

```bash
clang-17 our_binary.ll -Wno-override-module -c
llvm-cov-17 show --instr-profile=coverage.profdata our_binary.o --format=html -output-dir=coverage/
```

However, this approach may not work if your project relies on instructions that are only available in WebAssembly, such as `memory.size` or `memory.grow`. In such cases, an extra step is needed. Fortunately, the instructions in `our_binary.ll` are not necessary for generating a code coverage report, so you can remove all of them using the following command with a regular expression:

```bash
perl -i -p0e 's/(^define[^\n]*\n).*?^}\s*$/$1start:\n  unreachable\n}\n/gms' our_binary.ll
```

After performing these steps, execute the previously mentioned `clang-17` and `llvm-cov-17` commands. If you continue to encounter issues, consider disabling link-time optimizations (LTO) during your project build.

## Remarks

While this method involves some workarounds, it is functional and can help you generate code coverage reports for your WebAssembly projects. A more streamlined approach would involve integrating WebAssembly file support into `llvm-cov`. However, such an integration could be more complex and time-consuming. Therefore, the solution presented here is recommended for the time being.