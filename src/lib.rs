// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![doc = include_str!("../README.md")]
#![allow(unused_assignments)] // Too many false positives caused by binrw

extern crate core;

/// A continuous block of memory which is not owned, and comes either from an in-memory location or from a file.
pub type ByteSpan<'a> = &'a [u8];

/// A continuous block of memory which is owned.
pub type ByteBuffer = Vec<u8>;

/// Parsing game repositories, such as "ffxiv", "ex1" and their version information.
pub mod repository;

/// Handling data in the "boot" directory, which contains the launcher files.
pub mod bootdata;

/// SqPack file formats - including Db, Data and Index/Index2 files.
pub mod sqpack;

mod compression;

/// Types for model (`.mdl`) files.
pub mod model;

/// Playable race and genders.
pub mod race;

/// Types for Excel list (`.exl`) files.
pub mod exl;

/// Dealing with equipment and its data.
pub mod equipment;

/// Common structures, enumerations and functions used by other modules.
pub mod common;

/// Types for ZiPatch (`.patch`) files.
pub mod patch;

/// Implementation of the variety of Blowfish ECB block ciphers used.
pub mod blowfish;

/// Types for Excel header (`.exh`) files.
pub mod exh;

/// Types for Excel data (`.exd`) files.
pub mod exd;

/// Types for skeleton (`.sklb`) files.
pub mod skeleton;

/// Types for file info (`.fiin`) files.
pub mod fiin;

/// Types for textures (`.tex`) files.
pub mod tex;

/// Types for material (`.mtrl`) files.
pub mod mtrl;

/// Types for shader packages (`.shpk`) files.
pub mod shpk;

/// Types for character make parameter (`.cmp`) files.
pub mod cmp;

/// Types for and writing various saved data formats from the game.
pub mod savedata;

/// Types for and writing the plaintext config (`.cfg`) files.
pub mod cfg;

mod havok;

/// Types for pre-bone deform (`.pbd`) files.
pub mod pbd;

mod crc;
mod sha1;

/// Types for layer-related things, used by the [lgb] and [sgb] modules.
pub mod layer;

/// Types for terrain (`.tera`) files.
pub mod tera;

mod common_file_operations;
mod exd_file_operations;

/// Types for word dictionary (`.dic`) files.
pub mod dic;

#[doc(hidden)]
pub const PHYSIS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Types for UI layout definition (`.uld`) files.
pub mod uld;

/// Types for shared group binary (`.sgb`) files.
pub mod sgb;

/// Types for `.scd` files.
pub mod scd;

/// Types for hardware cursor (`.hwc`) files.
pub mod hwc;

/// Types for `.iwc` files.
pub mod iwc;

/// Types for `.tmb` files.
pub mod tmb;

/// Types for `.skp` files.
pub mod skp;

/// Types for shader (`.shcd`) files.
pub mod shcd;

/// Types for `.phyb` files.
pub mod phyb;

/// Types for `.pap` files.
pub mod pap;

/// Types for animated VFX (`.avfx`) files.
pub mod avfx;

/// Types for staining template material (`.stm`) files.
pub mod stm;

/// Types for patch lists.
pub mod patchlist;

/// Types for `.uwb` files.
pub mod uwb;

/// Types for level collision binary (`.lcb`) files.
pub mod lcb;

/// Types for level variable binary (`.lvb`) files.
pub mod lvb;

/// Types for sky visibility binary (`.svb`) files.
pub mod svb;

pub mod resource;

/// Types for player collision binary (`.pcb`) files.
pub mod pcb;

/// Types for collision streaming (`list.pcb`) files.
pub mod pcblist;

/// Types for cutscene binary (`.cutb`) files.
pub mod cutb;

pub mod excel;

mod bcn;

mod error;
pub use error::Error;

/// Find existing installation directories
pub mod existing_dirs;

/// Reading data from executables
pub mod execlookup;

/// Types for layer group binary (`.lgb`) files.
pub mod lgb;

/// Implementation detail for some types.
pub mod string_heap;

use crate::common::Platform;

/// A file that can be parsed from its serialized byte form.
///
/// This should be implemented for all types readable from SqPack.
pub trait ReadableFile: Sized {
    /// Read an existing file.
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self>;
}

/// A file that can be written back to its serialized byte form.
///
/// This should be implemented for all types readable from SqPack, on a best-effort basis.
pub trait WritableFile: Sized {
    /// Writes data back to a buffer.
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer>;
}

/// Used for basic sanity checking tests in other modules.
#[cfg(test)]
fn pass_random_invalid<T: crate::ReadableFile>() {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests");
    d.push("random");

    // Feeding it invalid data should not panic
    // Note that we don't check the Option currently, because some types like Hwc return Some regardless.
    T::from_existing(
        Platform::Win32,
        &std::fs::read(d).expect("Could not read random test file"),
    );
}
