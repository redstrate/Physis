// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::null_mut;

use libz_rs_sys::*;

/// Decompress ZLib data that has no header.
pub fn no_header_decompress(in_data: &mut [u8], out_data: &mut [u8]) -> bool {
    unsafe {
        let mut strm = z_stream {
            next_in: null_mut(),
            avail_in: in_data.len() as u32,
            total_in: 0,
            next_out: null_mut(),
            avail_out: 0,
            total_out: 0,
            msg: null_mut(),
            state: null_mut(),
            zalloc: None, // the default alloc is fine
            zfree: None,  // the default free is fine
            opaque: null_mut(),
            data_type: 0,
            adler: 0,
            reserved: 0,
        };

        let ret = inflateInit2_(
            &mut strm,
            -15,
            zlibVersion(),
            core::mem::size_of::<z_stream>() as i32,
        );
        if ret != Z_OK {
            return false;
        }

        strm.next_in = in_data.as_mut_ptr();
        strm.avail_out = out_data.len() as u32;
        strm.next_out = out_data.as_mut_ptr();

        let ret = inflate(&mut strm, Z_NO_FLUSH);
        if ret != Z_STREAM_END {
            return false;
        }

        inflateEnd(&mut strm);

        true
    }
}

/// Decompress zlib data that has a header.
pub fn header_decompress(in_data: &mut [u8], out_data: &mut [u8]) -> Option<usize> {
    unsafe {
        let mut strm = z_stream {
            next_in: null_mut(),
            avail_in: in_data.len() as u32,
            total_in: 0,
            next_out: null_mut(),
            avail_out: 0,
            total_out: 0,
            msg: null_mut(),
            state: null_mut(),
            zalloc: None, // the default alloc is fine
            zfree: None,  // the default free is fine
            opaque: null_mut(),
            data_type: 0,
            adler: 0,
            reserved: 0,
        };

        let ret = inflateInit_(
            &mut strm,
            zlibVersion(),
            core::mem::size_of::<z_stream>() as i32,
        );
        if ret != Z_OK {
            return None;
        }

        strm.next_in = in_data.as_mut_ptr();
        strm.avail_out = out_data.len() as u32;
        strm.next_out = out_data.as_mut_ptr();

        let ret = inflate(&mut strm, Z_NO_FLUSH);
        if ret != Z_STREAM_END && ret != Z_OK {
            return None;
        }

        inflateEnd(&mut strm);

        Some(out_data.len() - strm.avail_out as usize)
    }
}
