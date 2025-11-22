// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Read};

use crate::ByteSpan;

/// Hardware cursor file, usually with the `.hwc` file extension.
///
/// Contains a pixmap meant to be used as a hardware cursor.
#[derive(Debug)]
pub struct Hwc {
    /// RGBA8888 data for the cursor.
    pub rgba: Vec<u8>,
}

impl Hwc {
    /// Reads an existing HWC file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);

        let mut rgba = vec![0; Self::WIDTH * Self::HEIGHT * 4];
        cursor.read_exact(&mut rgba).ok()?;

        Some(Self { rgba })
    }

    /// The width of all hardware cursors.
    pub const WIDTH: usize = 64;

    /// The height of all hardware cursors.
    pub const HEIGHT: usize = 64;
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Hwc::from_existing(&read(d).unwrap());
    }
}
