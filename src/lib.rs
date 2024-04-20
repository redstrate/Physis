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

/// Common methods and structures relating to the SqPack data format.
pub mod sqpack;

/// Reading and writing SqPack index files.
pub mod index;

mod compression;
mod dat;

/// Reading model (MDL) files.
#[cfg(feature = "visual_data")]
pub mod model;

/// All of the races in Eorzea in a nice enum package.
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

mod blowfish_constants;

/// Initializing a new retail game install from the official retail installer. No execution required!
#[cfg(feature = "game_install")]
pub mod installer;

/// Reading Excel header files (EXH).
pub mod exh;

/// Reading Excel data files (EXD).
pub mod exd;

/// Reading Havok XML sidecar files.
#[cfg(feature = "visual_data")]
pub mod skeleton;

/// Reading file info files (FIIN).
pub mod fiin;

/// Reading and writing chat logs (LOG).
pub mod log;

/// Reading textures (TEX).
#[cfg(feature = "visual_data")]
pub mod tex;

/// Reading material files (MTRL)
#[cfg(feature = "visual_data")]
pub mod mtrl;

/// Reading shader packages (SHPK)
#[cfg(feature = "visual_data")]
pub mod shpk;

/// Reading character parameter files (CMP)
pub mod cmp;

/// Reading and writing character data files (DAT) which are used in the character creator to save presets.
pub mod chardat;

/// Reading and writing the plaintext config files (CFG) used by the game to store most of it's configuration.
pub mod cfg;

#[cfg(feature = "visual_data")]
mod havok;

/// Reading bone deform matrices.
#[cfg(feature = "visual_data")]
pub mod pbd;

mod crc;
mod sha1;

#[cfg(feature = "visual_data")]
mod model_file_operations;

#[cfg(feature = "visual_data")]
mod model_vertex_declarations;

#[cfg(feature = "visual_data")]
pub mod lgb;

#[cfg(feature = "visual_data")]
pub mod tera;

/// Reading data from executables
pub mod execlookup;

mod common_file_operations;

/// Reading word dictionaries, such as the vulgar word list.
pub mod dic;

#[doc(hidden)]
pub const PHYSIS_VERSION: &str = env!("CARGO_PKG_VERSION");
