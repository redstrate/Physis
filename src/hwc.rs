// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Read};

use crate::ByteSpan;

#[derive(Debug)]
pub struct Hwc {
    pub rgba: Vec<u8>,
}

const CURSOR_WIDTH: usize = 64;
const CURSOR_HEIGHT: usize = 64;

impl Hwc {
    /// Reads an existing HWC file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);

        let mut rgba = vec![0; CURSOR_WIDTH * CURSOR_HEIGHT * 4];
        cursor.read_exact(&mut rgba).ok()?;

        Some(Self { rgba })
    }
}
