// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Crate for reading and writing the file formats used by FFXIV.

extern crate core;

/// Represents a continuous block of memory which is not owned, and comes either from an in-memory location or from a file.
pub type ByteSpan<'a> = &'a [u8];

/// Represents a continuous block of memory which is owned.
pub type ByteBuffer = Vec<u8>;

/// Reading and writing game data repositories, such as "ffxiv" and "ex1", and so on.
pub mod gamedata;

/// Parsing game repositories, such as "ffxiv", "ex1" and their version information.
pub mod repository;

/// Handling and updating data in the "boot" directory, which contains the launcher files.
pub mod bootdata;

/// SqPack file formats - including Db, Data and Index/Index2 files.
pub mod sqpack;

mod compression;

/// Reading model (MDL) files.
pub mod model;

/// Playable race and genders.
pub mod race;

/// Reading Excel lists (EXL).
pub mod exl;

/// Reading equipment and equipment-related data.
pub mod equipment;

/// Common structures, enumerations and functions used by many modules.
pub mod common;

/// Methods for installing game and boot patches.
pub mod patch;

/// Implementation of the Blowfish ECB block cipher used by the retail client. It's used to encrypt arguments in the launcher, to prevent login token snooping.
pub mod blowfish;

/// Reading Excel header files (EXH).
pub mod exh;

/// Reading Excel data files (EXD).
pub mod exd;

/// Reading Havok XML sidecar files.
pub mod skeleton;

/// Reading file info files (FIIN).
pub mod fiin;

/// Reading textures (TEX).
pub mod tex;

/// Reading material files (MTRL)
pub mod mtrl;

/// Reading shader packages (SHPK)
pub mod shpk;

/// Reading character parameter files (CMP)
pub mod cmp;

/// Reading and writing various saved data formats from the game.
pub mod savedata;

/// Reading and writing the plaintext config files (CFG) used by the game to store most of it's configuration.
pub mod cfg;

mod havok;

/// Reading bone deform matrices.
pub mod pbd;

mod crc;
mod sha1;

/// Reading layer information for a map (LGB)
pub mod layer;

pub mod tera;

/// Reading data from executables
pub mod execlookup;

mod common_file_operations;

/// Reading word dictionaries, such as the vulgar word list.
pub mod dic;

#[doc(hidden)]
pub const PHYSIS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reading ULD files
pub mod uld;

/// Reading SGB files
pub mod sgb;

/// Reading SCD files
pub mod scd;

/// Reading HWC files
pub mod hwc;

/// Reading IWC files
pub mod iwc;

/// Reading TMB files
pub mod tmb;

/// Reading SKP files
pub mod skp;

/// Reading SCHD files
pub mod schd;

/// Reading PHYB files
pub mod phyb;

/// Reading PAP files
pub mod pap;

/// Reading AVFX files
pub mod avfx;

/// Reading STM files
pub mod stm;

/// Find existing installation directories
pub mod existing_dirs;

/// Reading patch lists
pub mod patchlist;

mod bcn;

mod error;
pub use error::Error;
