// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

mod resolver;
pub use resolver::ResourceResolver;

mod sqpack;
pub use sqpack::{RepairAction, RepairError, SqPackRelease, SqPackResource};

mod unpacked;
pub use unpacked::UnpackedResource;

use crate::{
    ByteBuffer, Error, ReadableFile,
    common::{Language, Platform},
    excel::{ExcelSheet, ExcelSheetPage},
    exd::EXD,
    exh::EXH,
    exl::EXL,
};

impl Clone for Box<dyn Resource> {
    fn clone(&self) -> Box<dyn Resource> {
        self.clone_box()
    }
}

/// Workaround for allowing Resources to be clonable (which normally isn't dyn compatible.)
///
/// When impementing your own Resources, you do not need to worry about this as it's an implementation detail. You just need to derive from the Clone trait.
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

/// Represents a source of files for reading.
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
    /// # use physis::common::Platform;
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
    /// # use physis::common::Platform;
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease};
    /// let mut game = SqPackResource::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
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
) -> Result<ExcelSheet, Error> {
    let mut pages = Vec::with_capacity(exh.header.page_count as usize);
    for page in 0..exh.header.page_count {
        let exd = generic_read_excel_exd(resource, name, &exh, language, page as usize)?;
        pages.push(ExcelSheetPage::from_exd(page, &exh, exd));
    }

    Ok(ExcelSheet { exh: exh.clone(), pages })
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
