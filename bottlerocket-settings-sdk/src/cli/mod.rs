//! Contains the definition of the command line interface for settings extensions.
//!
//! The default implementation of this interface is provided in the
//! [`extension` module](crate::extension).
#![allow(missing_docs)]
pub mod proto1;

use std::fmt::Display;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The Bottlerocket Settings CLI protocol to use
    #[command(subcommand)]
    pub protocol: Protocol,
}

#[derive(Subcommand, Debug)]
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
