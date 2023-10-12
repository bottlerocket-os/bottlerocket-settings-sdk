//! Provides sample settings models to be used in docstrings which would otherwise be cumbersome to
//! write inline.
//! Due to limitations in doctests, these are provided as `pub`; however, no guarantees are made to
//! their backwards compatibility.
#![doc(hidden)]
pub mod empty;
pub mod needs_migrator;

/// Error type to make examples compile.
#[derive(Debug)]
pub struct EmptyError;

impl std::fmt::Display for EmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EmptyError")
    }
}

impl std::error::Error for EmptyError {}

type Result<T> = std::result::Result<T, EmptyError>;
