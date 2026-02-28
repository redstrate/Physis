// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

/// Reading and writing character data (`FFXIV_CHARA_XX.DAT`) files which are used in the character creator to save presets.
pub mod chardat;

/// Reading and writing the content of gear set (`GEARSET.DAT`) files which are used to store a character's gear sets.
pub mod gearsets;

/// Reading and writing chat log (`.LOG`) files.
pub mod log;

/// Implementation details for dat-based files.
pub(crate) mod dat;
