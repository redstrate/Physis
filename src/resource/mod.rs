// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! File resource handling.
//!
//! To begin with, there is the [Resource] trait which represents a collection of readable game files. The trait is intentionally small so it can represent a wide variety of sources. We provide two kinds of Resources already: [SqPackResource] and [UnpackedResource].
//!
//! # Layering multiple Resources
//!
//! [ResourceResolver] can layer multiple Resources on top of each other. This is useful if you want to normally read from SqPack, but might want to add a caching solution or read loose files from a filesystem too.
//!
//! It's recommended to only use ResourceResolver if you actually plan to use more than one Resource at a time. A big limitation with ResourceResolver is that Resources become opaque/inaccessible once added. This means you cannot access internal details like [SqPackResource::repositories], but only what is available from the [Resource] trait.
//!
//! # Helpers
//!
//! [ResourceResolver] and [SqPackResource] have various helpers to make reading game files easier:
//!
//! * To list all Excel sheet names, use [ResourceResolver::get_all_sheet_names]/[SqPackResource::get_all_sheet_names].
//! * To read and parse file at once, use [ResourceResolver::parsed]/[SqPackResource::parsed].
//! * To read an Excel sheet header (`.exh`), use [ResourceResolver::read_excel_sheet_header]/[SqPackResource::read_excel_sheet_header].
//! * To read an Excel sheet, use [ResourceResolver::read_excel_sheet]/[SqPackResource::read_excel_sheet].
//!
//! # Deriving from Resource
//!
//! If you have a use-case that isn't handled by one of our built-in Resources, you can create your own by deriving the trait:
//!
//! ```no_run
//! use physis::resource::{Resource, SqPackResource};
//!
//! #[derive(Clone)]
//! struct SqPackResourceSpy {
//!     sqpack_resource: SqPackResource,
//!     output_directory: String,
//! }
//!
//! impl Resource for SqPackResourceSpy {
//!     fn read(&mut self, path: &str) -> Option<physis::ByteBuffer> {
//!         todo!()
//!     }
//!
//!     fn exists(&mut self, path: &str) -> bool {
//!         todo!()
//!     }
//! }
//! ```
//!
//! Due of limitations in Rust traits, helpers like the ones shown above can't be implemented automatically. But you can re-use the generic ones like [generic_parsed] and wrap them in your type.

mod resolver;
pub use resolver::ResourceResolver;

mod sqpack;
pub use sqpack::{RepairAction, RepairError, SqPackRelease, SqPackResource};

mod unpacked;
pub use unpacked::UnpackedResource;

use crate::{
    ByteBuffer, Error, ReadableFile,
    common::{Language, Platform},
    excel::{Page, Sheet},
    exd::EXD,
    exh::EXH,
    exl::EXL,
};

impl Clone for Box<dyn Resource> {
    fn clone(&self) -> Box<dyn Resource> {
        self.clone_box()
    }
}

/// Implementation detail for [cloning](Clone) a [Resource] (which normally isn't dyn compatible.)
///
/// When implementing your own Resource, you do not have to care about this. Simply derive from the [Clone] trait.
pub trait ClonableResource {
    fn clone_box(&self) -> Box<dyn Resource>;
}

impl<T> ClonableResource for T
where
    T: 'static + Resource + Clone,
{
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl Default for ResourceResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// A collection of readable game files.
///
/// This abstracts away some of the nitty-gritty of where files come from. This could represent a compressed archive like SqPack, unpacked files on disk, or even a network.
pub trait Resource: Send + Sync + ClonableResource + 'static {
    /// Reads the file located at `path`. This is returned as an in-memory buffer, and will usually
    /// have to be further parsed.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease};
    /// # use std::io::Write;
    /// # use physis::Platform;
    /// let mut game = SqPackResource::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
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
    /// # use physis::Platform;
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease};
    /// let mut game = SqPackResource::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
    /// if game.exists("exd/cid.exl") {
    ///     println!("Cid really does exist!");
    /// } else {
    ///     println!("Oh noes!");
    /// }
    /// ```
    fn exists(&mut self, path: &str) -> bool;

    /// Returns the platform associated with this resource.
    fn platform(&self) -> Platform {
        Platform::Win32
    }
}

// Below are generic across all Resources. We have to do this because of limitations with dyn Traits in Rust.
// And we also don't want these solely in ResourceResolver, because these are also useful standalone.

/// Generically parse a file from a `Resource`. You most likely want to use the method in `ResourceResolver.`
pub fn generic_parsed<R: Resource + ?Sized, F: ReadableFile>(
    resource: &mut R,
    path: &str,
) -> Result<F, Error> {
    if let Some(bytes) = resource.read(path) {
        return F::from_existing(resource.platform(), &bytes).ok_or(Error::FileParsingFailed {
            path: path.to_string(),
        });
    }

    Err(Error::FileNotFound {
        path: path.to_string(),
    })
}

/// Read an excel sheet header by name (e.g. "Achievement"). You most likely want to use the method in `ResourceResolver.`
pub fn generic_read_excel_sheet_header<R: Resource + ?Sized>(
    resource: &mut R,
    name: &str,
) -> Result<EXH, Error> {
    let new_filename = name.to_lowercase();

    let path = format!("exd/{new_filename}.exh");

    generic_parsed::<R, EXH>(resource, &path)
}

/// Read an excel sheet by name (e.g. "Achievement"). You most likely want to use the method in `ResourceResolver.`
pub fn generic_read_excel_sheet<R: Resource + ?Sized>(
    resource: &mut R,
    exh: &EXH,
    name: &str,
    language: Language,
) -> Result<Sheet, Error> {
    let mut pages = Vec::with_capacity(exh.header.page_count as usize);
    for page in 0..exh.header.page_count {
        let exd = generic_read_excel_exd(resource, name, exh, language, page as usize)?;
        pages.push(Page::from_exd(exh, exd));
    }

    Ok(Sheet {
        exh: exh.clone(),
        pages,
    })
}

/// Returns all known sheet names listed in the root list. You most likely want to use the method in `ResourceResolver.`
pub fn generic_get_all_sheet_names<R: Resource + ?Sized>(
    resource: &mut R,
) -> Result<Vec<String>, Error> {
    let root_exl = generic_parsed::<R, EXL>(resource, "exd/root.exl")?;
    Ok(root_exl
        .entries
        .iter()
        .map(|(row, _)| row.clone())
        .collect())
}

fn generic_read_excel_exd<R: Resource + ?Sized>(
    resource: &mut R,
    name: &str,
    exh: &EXH,
    language: Language,
    page: usize,
) -> Result<EXD, Error> {
    let exd_path = format!(
        "exd/{}",
        EXD::calculate_filename(name, language, &exh.pages[page])
    );

    generic_parsed::<R, EXD>(resource, &exd_path)
}
