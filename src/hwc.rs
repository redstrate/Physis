// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Read};

use crate::{ByteSpan, ReadableFile, common::Platform};

/// Hardware cursor file, usually with the `.hwc` file extension.
///
/// Contains a pixmap meant to be used as a hardware cursor.
#[derive(Debug)]
pub struct Hwc {
    /// RGBA8888 data for the cursor.
    pub rgba: Vec<u8>,
}

impl ReadableFile for Hwc {
    fn from_existing(_platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);

        let mut rgba = vec![0; Self::WIDTH * Self::HEIGHT * 4];
        cursor.read_exact(&mut rgba).ok()?;

        Some(Self { rgba })
    }
}

impl Hwc {
    /// The width of all hardware cursors, in pixels.
    pub const WIDTH: usize = 64;

    /// The height of all hardware cursors, in pixels.
    pub const HEIGHT: usize = 64;
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Hwc>();
    }
}
