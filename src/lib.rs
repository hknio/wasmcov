#[cfg(target_family = "wasm")]
pub use minicov;
#[cfg(target_family = "wasm")]
pub mod near;

#[cfg(not(target_family = "wasm"))]
pub mod build;
#[cfg(not(target_family = "wasm"))]
pub mod dir;
#[cfg(not(target_family = "wasm"))]
pub mod llvm;
#[cfg(not(target_family = "wasm"))]
pub mod report;
#[cfg(not(target_family = "wasm"))]
pub mod utils;

#[cfg(all(not(target_family = "wasm"), feature = "near_sandbox"))]
pub mod near_sandbox;
