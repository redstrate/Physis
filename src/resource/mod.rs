// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

mod resolver;
pub use resolver::ResourceResolver;

mod sqpack;
pub use sqpack::{RepairAction, RepairError, SqPackRelease, SqPackResource};

mod unpacked;
pub use unpacked::UnpackedResource;

use crate::{ByteBuffer, common::Platform};

/// Represents a source of files for reading.
///
/// This abstracts away some of the nitty-gritty of where files come from. This could represent a compressed archive like SqPack, unpacked files on disk, or even a network.
pub trait Resource {
    /// Reads the file located at `path`. This is returned as an in-memory buffer, and will usually
    /// have to be further parsed.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease};
    /// # use std::io::Write;
    /// # use physis::common::Platform;
    /// let mut game = SqPackResource::from_existing(Platform::Win32, SqPackRelease::Retail, "SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
    /// let data = game.read("exd/root.exl").unwrap();
    ///
    /// let mut file = std::fs::File::create("root.exl").unwrap();
    /// file.write(data.as_slice()).unwrap();
    /// ```
    fn read(&mut self, path: &str) -> Option<ByteBuffer>;

    /// Checks if a file exists.
    ///
    /// While you could abuse `read` to do this, in some Resources they can optimize this since it doesn't read data.
    ///
    /// # Example
    ///
    /// ```
    /// # use physis::common::Platform;
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease};
    /// let mut game = SqPackResource::from_existing(Platform::Win32, SqPackRelease::Retail, "SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
    /// if game.exists("exd/cid.exl") {
    ///     println!("Cid really does exist!");
    /// } else {
    ///     println!("Oh noes!");
    /// }
    /// ```
    fn exists(&mut self, path: &str) -> bool;

    /// Returns the `Platform` associated with this resource.
    fn platform(&self) -> Platform {
        Platform::Win32
    }
}
