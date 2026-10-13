// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{BinRead, BinReaderExt, BinWrite};

use crate::ByteSpan;

/// Byte that appears at the beginning of a payload.
const START_BYTE: u8 = 2;
/// Byte that appears at the end of a payload.
const END_BYTE: u8 = 3;

/// SeString payload.
#[derive(Debug, Clone, PartialEq)]
enum Payload {
    /// A text payload. This is guaranteed to be valid UTF-8.
    Text(String),
    /// A unknown payload.
    Unknown { chunk_type: u8, data: Vec<u8> },
}

impl Payload {
    /// Returns this payload's contents, encoded like how it was parsed.
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Payload::Text(text) => text.as_bytes().to_vec(),
            Payload::Unknown { chunk_type, data } => {
                let mut encoded_data = vec![START_BYTE, *chunk_type];
                encoded_data.extend_from_slice(&make_integer(data.len() as u32));
                encoded_data.extend_from_slice(data);
                encoded_data.push(END_BYTE);

                encoded_data
            }
        }
    }

    /// Returns the size of this payload. This is cheaper than checking `as_bytes().len()`.
    fn len(&self) -> usize {
        match self {
            Payload::Text(text) => text.len(),
            Payload::Unknown { data, .. } => data.len() + 4,
        }
    }
}

impl std::fmt::Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Payload::Text(text) => write!(f, "{}", text),
            Payload::Unknown { chunk_type, .. } => write!(f, "<?:{chunk_type}>"),
        }
    }
}

fn make_integer(value: u32) -> Vec<u8> {
    if value < 0xCF {
        return vec![(value + 1) as u8];
    }

    let bytes = value.to_le_bytes();

    let mut ret = vec![0xF0];
    for i in (0..4).rev() {
        if bytes[i] != 0 {
            ret.push(bytes[i]);
            ret[0] |= (1 << i) as u8;
        }
    }

    ret[0] -= 1;

    ret
}

fn read_integer(marker: u32, pos: &mut usize, bytes: &[u8]) -> u32 {
    if marker < 0xD0 {
        return marker - 1;
    }

    let marker = (marker as u8 + 1) & 0b1111;

    let mut ret = [0; 4];
    for i in (0..4).rev() {
        ret[i] = if (marker & (1 << i)) == 0 {
            0
        } else {
            let byte = bytes[*pos];
            *pos += 1;
            byte
        };
    }

    u32::from_le_bytes(ret)
}

/// Represents a parsed SeString, which can contain macros (in localization) or user data like location flags (in chat.)
///
/// This doesn't support any payload apart from text yet, but the string can still be displayed using [ToString::to_string].
/// Unknown payloads are labeled as `<?:X>` where X is the chunk type.
///
/// This struct is capable of passing along strings losslessly, use [Self::as_bytes] to turn a string back into its encoded form.
///
/// This struct intentionally does not implement `Serialize/Deserialize`. You should store the bytes instead, so you aren't dependent on our internal representation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeString {
    payloads: Vec<Payload>,
}

impl BinRead for SeString {
    type Args<'a> = binrw::VecArgs<()>;

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let data: Vec<u8> = reader.read_type_args(endian, args)?;
        Ok(Self::parse(&data))
    }
}

impl BinWrite for SeString {
    type Args<'a> = ();

    fn write_options<W: std::io::prelude::Write + std::io::prelude::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<()> {
        writer.write_all(&self.as_bytes())?;
        Ok(())
    }
}

impl SeString {
    /// Parse the given `bytes` as a SeString.
    pub fn parse(bytes: ByteSpan) -> Self {
        let mut payloads = Vec::new();
        let mut pos = 0;
        loop {
            if pos >= bytes.len() {
                break;
            }

            let next_byte = bytes[pos];
            if next_byte != START_BYTE {
                let mut data = Vec::new();
                loop {
                    if pos >= bytes.len() {
                        break;
                    }
                    if bytes[pos] == START_BYTE {
                        break;
                    }
                    data.push(bytes[pos]);
                    pos += 1;
                }

                // Read up until the first nul terminator
                let index = data.iter().position(|b| *b == 0).unwrap_or(data.len());
                data.truncate(index);

                payloads.push(Payload::Text(String::from_utf8(data).unwrap_or_default()));
            } else {
                let mut data = Vec::new();
                pos += 1;
                let chunk_type = bytes[pos];
                pos += 1;
                let size = bytes[pos];
                pos += 1;
                let size = read_integer(size as u32, &mut pos, bytes);
                for _ in 0..size {
                    if pos >= bytes.len() {
                        break;
                    }
                    data.push(bytes[pos]);
                    pos += 1;
                }
                // skip end byte
                pos += 1;
                payloads.push(Payload::Unknown { chunk_type, data });
            }
        }

        Self { payloads }
    }

    /// Return the contents, encoded like how it was parsed.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.payloads
            .iter()
            .map(Payload::as_bytes)
            .collect::<Vec<_>>()
            .concat()
    }

    /// Returns the length of this string. This is cheaper than checking `as_bytes().len()`.
    pub fn len(&self) -> usize {
        self.payloads.iter().map(Payload::len).sum()
    }

    /// If this string is empty. This is cheaper than checking `len()`.
    pub fn is_empty(&self) -> bool {
        if self.payloads.is_empty() {
            return true;
        }

        self.len() > 0
    }
}

impl std::fmt::Display for SeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for payload in &self.payloads {
            write!(f, "{payload}")?;
        }
        Ok(())
    }
}

impl From<&str> for SeString {
    fn from(value: &str) -> Self {
        Self {
            payloads: vec![Payload::Text(value.into())],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_payload_size() {
        let data = [2, 255, 50];

        // This shouldn't panic
        let sestring = SeString::parse(&data);
        assert_eq!(sestring.to_string(), "<?:255>");
    }

    #[test]
    fn nul_terminator() {
        let data = [2, 255, 50];

        // This shouldn't panic
        let sestring = SeString::parse(&data);
        assert_eq!(sestring.to_string(), "<?:255>");
    }

    #[test]
    fn test_item_bought() {
        let data = [
            89, 111, 117, 32, 112, 117, 114, 99, 104, 97, 115, 101, 32, 2, 8, 50, 228, 232, 3, 2,
            255, 16, 2, 49, 12, 255, 5, 73, 116, 101, 109, 2, 232, 2, 2, 2, 3, 255, 27, 2, 34, 6,
            232, 3, 255, 2, 44, 3, 32, 2, 49, 13, 255, 5, 73, 116, 101, 109, 4, 232, 2, 232, 3, 2,
            3, 3, 32, 102, 111, 114, 32, 2, 34, 6, 232, 4, 255, 2, 44, 3, 32, 103, 105, 108, 46,
        ];

        let sestring = SeString::parse(&data);
        assert_eq!(sestring.to_string(), "You purchase <?:8> for <?:34> gil.");

        // round-trip shouldn't modify it
        assert_eq!(sestring.as_bytes(), data);
        assert_eq!(sestring.len(), data.len());
    }

    #[test]
    fn test_leave() {
        let data = [
            89, 111, 117, 32, 108, 101, 97, 118, 101, 32, 226, 128, 156, 2, 8, 22, 233, 9, 255, 16,
            2, 40, 12, 255, 7, 79, 98, 106, 83, 116, 114, 233, 9, 1, 3, 235, 4, 3, 46, 226, 128,
            157,
        ];

        let sestring = SeString::parse(&data);
        assert_eq!(sestring.to_string(), "You leave “<?:8>.”");

        // round-trip shouldn't modify it
        assert_eq!(sestring.as_bytes(), data);
        assert_eq!(sestring.len(), data.len());
    }
}
