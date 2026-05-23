// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

/// The error type used throughout this crate.
#[derive(Debug)]
pub enum Error {
    /// The specified path was not found in any resources.
    FileNotFound {
        /// The path to the file that wasn't found.
        path: String,
    },
    /// I/O error.
    Io(std::io::Error),
    /// Binrw error.
    Binrw(binrw::Error),
    /// zlib error.
    Zlib(i32),
    /// The file header loaded correctly, but considered invalid for some other reason e.g. there are no chunks in a file that is supposed to have at least one.
    InvalidFile,
    /// While patching, if we encounted a chunk that requires TargetInfo to be present but isn't at that point in time.
    TargetInfoMissing,
    /// The hash wasn't found in any index file.
    HashNotFound { hash: crate::sqpack::Hash },
    /// Right now, this is only used when trying to find the parent during patching of this path but couldn't.'
    InvalidFilename { path: PathBuf },
    /// Right now is this a catch-all error when a resolver function fails.
    ResolverFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::FileNotFound { path } => write!(f, "file not found: {path}"),
            Error::ResolverFailed => write!(f, "resolver failed"),
            Error::Io(error) => write!(f, "io: {error}"),
            Error::Binrw(error) => write!(f, "binrw: {error}"),
            Error::Zlib(error) => write!(f, "zlib: {error}"),
            Error::InvalidFile => write!(f, "invalid file"),
            Error::TargetInfoMissing => write!(f, "target info missing"),
            Error::HashNotFound { hash } => write!(f, "hash {hash:?} not found"),
            Error::InvalidFilename { path } => write!(f, "invalid filename: {path:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<binrw::Error> for Error {
    fn from(value: binrw::Error) -> Self {
        Self::Binrw(value)
    }
}
