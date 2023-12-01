//! Contains the definition of the command line interface for settings extensions.
//!
//! The default implementation of this interface is provided in the
//! [`extension` module](crate::extension).
#![allow(missing_docs)]
pub mod proto1;

use argh::FromArgs;
use std::fmt::Display;

/// Provides a CLI interface to the settings extension.
#[derive(FromArgs, Debug)]
pub struct Cli {
    /// the Bottlerocket Settings CLI protocol to use
    #[argh(subcommand)]
    pub protocol: Protocol,
}

/// The CLI protocol to use when invoking the extension.
#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum Protocol {
    #[cfg(feature = "proto1")]
    /// Settings extension protocol 1
    Proto1(proto1::Protocol1),
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Proto1(_) => "proto1",
        })
    }
}
