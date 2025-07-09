mod resolver;
pub use resolver::ResourceResolver;

mod sqpack;
pub use sqpack::SqPackResource;

mod unpacked;
pub use unpacked::UnpackedResource;

use crate::ByteBuffer;

/// Represents a source of files for reading.
/// This abstracts away some of the nitty-gritty of where files come from. These could be coming from a compressed archive like SqPack, unpacked files on disk, or even a network.
pub trait Resource {
    /// Reads the file located at `path`. This is returned as an in-memory buffer, and will usually
    /// have to be further parsed.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::resource::{Resource, SqPackResource};
    /// # use std::io::Write;
    /// # use physis::common::Platform;
    /// let mut game = SqPackResource::from_existing(Platform::Win32, "SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
    /// let data = game.read("exd/root.exl").unwrap();
    ///
    /// let mut file = std::fs::File::create("root.exl").unwrap();
    /// file.write(data.as_slice()).unwrap();
    /// ```
    fn read(&mut self, path: &str) -> Option<ByteBuffer>;

    /// Checks if a file exists
    /// While you could abuse `read` to do this, in some Resources they can optimize this since it doesn't read data.
    ///
    /// # Example
    ///
    /// ```
    /// # use physis::common::Platform;
    /// # use physis::resource::{Resource, SqPackResource};
    /// let mut game = SqPackResource::from_existing(Platform::Win32, "SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
    /// if game.exists("exd/cid.exl") {
    ///     println!("Cid really does exist!");
    /// } else {
    ///     println!("Oh noes!");
    /// }
    /// ```
    fn exists(&mut self, path: &str) -> bool;
}
