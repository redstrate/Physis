// SPDX-FileCopyrightText: 2023 Rudolf Kolbe
// SPDX-License-Identifier: MIT

extern crate alloc;

mod bc1;
mod bc3;
mod bc5;
mod bc7;
mod bitreader;
mod color;
mod consts;
mod macros;

pub use bc1::decode_bc1_block;
pub use bc3::decode_bc3_block;
pub use bc5::decode_bc5_block;
pub use bc7::decode_bc7_block;

macros::block_decoder!(decode_bc1, 4, 4, 8, decode_bc1_block);
macros::block_decoder!(decode_bc3, 4, 4, 16, decode_bc3_block);
macros::block_decoder!(decode_bc5, 4, 4, 16, decode_bc5_block);
macros::block_decoder!(decode_bc7, 4, 4, 16, decode_bc7_block);
