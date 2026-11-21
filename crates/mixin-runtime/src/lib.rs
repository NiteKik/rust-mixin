//! LLVM Mixin runtime: payload validation, planning, transformation and JIT.
//! The current backend supports trusted Windows x64 scalar components.
pub mod bundle;
pub mod diagnostic;
mod hot_reload;
pub mod llvm;
pub mod metadata;
mod scalar;
pub use diagnostic::{Diagnostic, DiagnosticResult, Stage};
pub use hot_reload::{BackgroundReload, HotReload};
pub use llvm::{Api, Jit, Plan};
pub use mixin_sdk::{EntityId, Vec2};
pub use scalar::{ScalarType, ScalarValue};
pub type Result<T> = std::result::Result<T, String>;
#[cfg(test)]
mod tests_support;
