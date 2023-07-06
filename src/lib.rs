extern crate core;

/// Reading and writing game data repositories, such as "ffxiv" and "ex1", and so on.
pub mod gamedata;

/// Parsing game repositories, such as "ffxiv", "ex1" and their version information.
pub mod repository;

/// Reading and writing the boot data repository.
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

/// Common structures used by other modules.
pub mod common;

/// Methods for installing game and boot patches.
pub mod patch;

#[macro_use]
mod macros;

/// Implementation of the Blowfish ECB block cipher used by the retail client.
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

/// Reading file into files (FIIN).
pub mod fiin;

/// Reading and writing chat logs (LOG).
pub mod log;

/// Reading textures (TEX).
#[cfg(feature = "visual_data")]
pub mod tex;

/// Reading material files (MTRL)
#[cfg(feature = "visual_data")]
pub mod mtrl;

/// Reading character parameter files (CMP)
pub mod cmp;

mod crc;
mod sha1;
