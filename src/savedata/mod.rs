// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

/// Reading and writing character data (`FFXIV_CHARA_XX.DAT`) files.
///
/// These are used in the character creator to save presets.
pub mod chardat;

/// Reading and writing the content of gear set (`GEARSET.DAT`) files.
///
/// These are used to store a character's gear sets.
pub mod gearsets;

/// Reading and writing chat log (`.LOG`) files.
///
/// This contains previously recorded messages from other players
pub mod log;

/// Implementation details for dat-based files.
pub(crate) mod dat;

/// Reading and writing replay (`.dat`) files.
///
/// These are used by the Duty Recorder system.
pub mod replay;
