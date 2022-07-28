extern crate core;

pub mod gamedata;

/// Reading game data repositories, such as "ffxiv" and "ex1", and so on.
pub mod repository;

pub mod bootdata;

/// Everything to do with reading SqPack files.
pub mod sqpack;

pub mod index;
pub mod dat;
mod compression;
mod model;
pub mod race;

/// Reading Excel lists (EXL).
pub mod exl;
pub mod equipment;
pub mod common;
pub mod patch;

#[macro_use]
mod macros;

pub mod blowfish;
mod blowfish_constants;
pub mod installer;
pub mod exh;
pub mod exd;