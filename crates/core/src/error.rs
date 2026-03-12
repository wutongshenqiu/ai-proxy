// Re-export ProxyError from prism-types (canonical source).
// The axum IntoResponse and reqwest From impls are provided
// via prism-types feature flags enabled in this crate's Cargo.toml.
pub use prism_types::error::ProxyError;
