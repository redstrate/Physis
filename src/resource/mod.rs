// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

mod resolver;
pub use resolver::ResourceResolver;

mod sqpack;
pub use sqpack::{SqPackResource, RepairAction, RepairError};

mod unpacked;
pub use unpacked::UnpackedResource;

use crate::{ByteBuffer, common::Language, exd::EXD, exh::EXH, exl::EXL};

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

/// Read an excel sheet by name (e.g. "Achievement")
pub fn read_excel_sheet_header<T: Resource>(resource: &mut T, name: &str) -> Option<EXH> {
    let root_exl_file = resource.read("exd/root.exl")?;

    let root_exl = EXL::from_existing(&root_exl_file)?;

    for (row, _) in root_exl.entries {
        if row == name {
            let new_filename = name.to_lowercase();

            let path = format!("exd/{new_filename}.exh");

            return EXH::from_existing(&resource.read(&path)?);
        }
    }

    None
}

/// Returns all known sheet names listed in the root list
pub fn get_all_sheet_names<T: Resource>(resource: &mut T) -> Option<Vec<String>> {
    let root_exl_file = resource.read("exd/root.exl")?;

    let root_exl = EXL::from_existing(&root_exl_file)?;

    let mut names = vec![];
    for (row, _) in root_exl.entries {
        names.push(row);
    }

    Some(names)
}

/// Read an excel sheet
pub fn read_excel_sheet<T: Resource>(
    resource: &mut T,
    name: &str,
    exh: &EXH,
    language: Language,
    page: usize,
) -> Option<EXD> {
    let exd_path = format!(
        "exd/{}",
        EXD::calculate_filename(name, language, &exh.pages[page])
    );

    let exd_file = resource.read(&exd_path)?;

    EXD::from_existing(exh, &exd_file)
}
