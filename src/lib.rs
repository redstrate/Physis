// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![doc = include_str!("../README.md")]
#![allow(unused_assignments)] // Too many false positives caused by binrw

#[doc(hidden)]
pub const PHYSIS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parsing game repositories, such as "ffxiv", "ex1" and their version information.
pub mod repository;

/// Various SqPack file formats (`.dat`, `.index`/`.index2` and `.sqdb`).
///
/// If you're only looking for a simple way to read game files, you probably want [SqPackResource](crate::resource::SqPackResource).
/// The types in this module are implementation details for [Resources](crate::resource::Resource).
pub mod sqpack;

/// Types for model (`.mdl`) files.
pub mod model;

/// Playable race and genders.
pub mod race;

/// Types for Excel list (`.exl`) files.
///
/// If you're only looking for a simple way to read Excel data, you probably want to look at the [excel] module instead of using these types.
pub mod exl;

/// Dealing with equipment and its data.
pub mod equipment;

// NOTE: Should be brought up to top-level because these are the most used types.
mod common;
#[cfg(test)]
pub(crate) use common::pass_random_invalid;
pub use common::{
    AABB, ByteBuffer, ByteSpan, Color, ColorIntensity, Language, Platform, ReadableFile,
    TerritoryIntendedUse, Version, WritableFile, read_version,
};

/// Types for ZiPatch (`.patch`) files.
pub mod patch;

/// Implementation of the variety of Blowfish ECB block ciphers used.
pub mod blowfish;

/// Types for Excel header (`.exh`) files.
///
/// If you're only looking for a simple way to read Excel data, you probably want to look at the [excel] module instead of using these types.
pub mod exh;

/// Types for Excel data (`.exd`) files.
///
/// If you're only looking for a simple way to read Excel data, you probably want to look at the [excel] module instead of using these types.
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

/// Types for environment binary files (`.envb`) files.
pub mod envb;

/// Types for character make parameter (`.cmp`) files.
pub mod cmp;

/// Types for various saved data formats.
///
/// These files are saved to the user directory during gameplay.
pub mod savedata;

/// Types for plaintext config (`.cfg`) files.
pub mod cfg;

/// Havok file parsing currently used for reading skeletons.
mod havok;

/// Types for pre-bone deform (`.pbd`) files.
pub mod pbd;

/// We need CRC!!
mod crc;

/// Types for layer-related things, used by the [lgb] and [sgb] modules.
pub mod layer;

/// Types for terrain (`.tera`) files.
pub mod tera;

/// Implementation details for various file types.
mod common_file_operations;

/// Implementation details for EXDs.
mod exd_file_operations;

/// Types for word dictionary (`.dic`) files.
pub mod dic;

/// Types for UI layout definition (`.uld`) files.
pub mod uld;

/// Types for shared group binary (`.sgb`) files.
pub mod sgb;

/// Types for sound compressed data (`.scd`) files.
pub mod scd;

/// Types for hardware cursor (`.hwc`) files.
pub mod hwc;

/// Types for `.iwc` files.
pub mod iwc;

/// Types for timeline binary (`.tmb`) files.
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

/// Types for underwater binary (`.uwb`) files.
pub mod uwb;

/// Types for light clipping binary (`.lcb`) files.
pub mod lcb;

/// Types for level variable binary (`.lvb`) files.
pub mod lvb;

/// Types for sky visibility binary (`.svb`) files.
pub mod svb;

pub mod resource;

/// Types for player collision binary (`.pcb`) files.
pub mod pcb;

/// Types for cutscene binary (`.cutb`) files.
pub mod cutb;

pub mod excel;

/// Implementation detail for textures.
mod bcn;

// NOTE: Should be brought up to the top-level because it's a basic error type.
mod error;
pub use error::Error;

/// A type alias for `Result<T, physis::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Find existing installation directories.
pub mod existing_dirs;

/// Reading data from executables.
pub mod execlookup;

/// Types for layer group binary (`.lgb`) files.
pub mod lgb;

/// Implementation detail for some types.
pub mod string_heap;

/// Scene information section used in a few file types.
pub mod scn;

/// Types for environment sound scape files (`.essb`) files.
pub mod essb;

/// Environment information section used in a few file types.
pub mod envs;

/// Types for ambient set (`.amb`) files.
pub mod amb;

/// Types for object behavior set binary (`.obsb`) files.
pub mod obsb;

/// Types for font data table (`.fdt`) files.
pub mod fdt;

/// Types for attach offset (`.atch`) files.
pub mod atch;

/// Types for bind point (`.eid`) files.
pub mod eid;

/// Types for shader parameter map (`.spm`) files.
pub mod spm;

/// Types for grass zone data (`.gzd`) files.
pub mod gzd;

/// Types for grass grid data (`.ggd`) files.
pub mod ggd;

/// Types for cloud data binary (`.cldb`) files.
pub mod cldb;

/// Types for SeStrings, which is used for various localized strings.
pub mod sestring;
