// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ptr::null_mut;

use libz_sys::*;

// This module's functions are licensed under MIT from https://github.com/rust-lang/flate2-rs
mod flate2_zallocation {
    use std::alloc::{self, Layout};
    use std::ffi::c_void;
    use std::ptr::null_mut;

    const ALIGN: usize = std::mem::align_of::<usize>();

    fn align_up(size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }

    pub extern "C" fn zalloc(_ptr: *mut c_void, items: u32, item_size: u32) -> *mut c_void {
        // We need to multiply `items` and `item_size` to get the actual desired
        // allocation size. Since `zfree` doesn't receive a size argument we
        // also need to allocate space for a `usize` as a header so we can store
        // how large the allocation is to deallocate later.
        let size = match items
            .checked_mul(item_size)
            .and_then(|i| usize::try_from(i).ok())
            .map(|size| align_up(size, ALIGN))
            .and_then(|i| i.checked_add(std::mem::size_of::<usize>()))
        {
            Some(i) => i,
            None => return null_mut(),
        };

        // Make sure the `size` isn't too big to fail `Layout`'s restrictions
        let layout = match Layout::from_size_align(size, ALIGN) {
            Ok(layout) => layout,
            Err(_) => return null_mut(),
        };

        unsafe {
            // Allocate the data, and if successful store the size we allocated
            // at the beginning and then return an offset pointer.
            let ptr = alloc::alloc(layout) as *mut usize;
            if ptr.is_null() {
                return ptr as *mut c_void;
            }
            *ptr = size;
            ptr.add(1) as *mut c_void
        }
    }

    pub extern "C" fn zfree(_ptr: *mut c_void, address: *mut c_void) {
        unsafe {
            // Move our address being freed back one pointer, read the size we
            // stored in `zalloc`, and then free it using the standard Rust
            // allocator.
            let ptr = (address as *mut usize).offset(-1);
            let size = *ptr;
            let layout = Layout::from_size_align_unchecked(size, ALIGN);
            alloc::dealloc(ptr as *mut u8, layout)
        }
    }
}

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
            zalloc,
            zfree,
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
