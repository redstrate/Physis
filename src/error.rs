// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, Clone)]
pub enum Error {
    /// The specified path was not found in any resources.
    FileNotFound {
        /// The path to the file that wasn't found.
        path: String,
    },
    /// There was an error while parsing this file.
    FileParsingFailed {
        /// The path to the file that failed to parse.
        path: String,
    },
    // TODO: Remove before release
    Unknown,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::FileNotFound { path } => write!(f, "file not found: {path}"),
            Error::FileParsingFailed { path } => write!(f, "file parsing failed: {path}"),
            Error::Unknown => write!(f, "unknown error"),
        }
    }
}
