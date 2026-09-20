//! Utility module: cancellation, path normalization.
//! Per `05-RUST-CORE-ENGINE.md` §5.1.

pub mod cancel;
pub mod paths;

pub use cancel::CancelToken;
