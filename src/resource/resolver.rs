// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    ByteBuffer, Error, ReadableFile,
    common::Language,
    excel::{ExcelSheet, ExcelSheetPage},
    exd::EXD,
    exh::EXH,
    exl::EXL,
};

use super::Resource;

/// Allows chaining multiple Resources together. Also contains helper functions useful for extracting higher-level data and files.
///
/// # Example
///
/// ```
/// # use physis::resource::{ResourceResolver, SqPackResource, UnpackedResource, SqPackRelease};
/// # use physis::common::Platform;
/// let sqpack_source = SqPackResource::from_existing(Platform::Win32, SqPackRelease::Retail, "SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
/// let file_source = UnpackedResource::from_existing("unpacked/");
/// let mut resolver = ResourceResolver::new();
/// resolver.add_source(Box::new(file_source)); // first has most priority
/// resolver.add_source(Box::new(sqpack_source)); // this is the fallback
/// ```
pub struct ResourceResolver {
    resolvers: Vec<Box<dyn Resource + Send + Sync>>,
}

impl Default for ResourceResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceResolver {
    /// Create a new, empty resolver.
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    /// Adds a new source to this resolver, and makes it the least prioritized.
    pub fn add_source(&mut self, source: Box<dyn Resource + Send + Sync>) {
        self.resolvers.push(source);
    }

    // TODO: add documentation
    pub fn read(&mut self, path: &str) -> Option<ByteBuffer> {
        for resolver in &mut self.resolvers {
            if let Some(bytes) = resolver.read(path) {
                return Some(bytes);
            }
        }

        None
    }

    /// Reads and parses the file located at `path`. This avoids having to call both `read` and `from_existing`.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease, ResourceResolver};
    /// # use physis::exl::EXL;
    /// # use std::io::Write;
    /// # use physis::common::Platform;
    /// let mut resolver = ResourceResolver::new();
    /// resolver.add_source(Box::new(SqPackResource::from_existing(Platform::Win32, SqPackRelease::Retail, "SquareEnix/Final Fantasy XIV - A Realm Reborn/game")));
    ///
    /// let exl = resolver.parsed::<EXL>("exd/root.exl").unwrap();
    /// ```
    pub fn parsed<T: ReadableFile>(&mut self, path: &str) -> Result<T, Error> {
        for resolver in &mut self.resolvers {
            if let Some(bytes) = resolver.read(path) {
                return T::from_existing(resolver.platform(), &bytes).ok_or(
                    Error::FileParsingFailed {
                        path: path.to_string(),
                    },
                );
            }
        }

        Err(Error::FileNotFound {
            path: path.to_string(),
        })
    }

    // TODO: add documentation
    pub fn exists(&mut self, path: &str) -> bool {
        for resolver in &mut self.resolvers {
            if resolver.exists(path) {
                return true;
            }
        }

        false
    }

    /// Read an excel sheet header by name (e.g. "Achievement")
    pub fn read_excel_sheet_header(&mut self, name: &str) -> Result<EXH, Error> {
        let new_filename = name.to_lowercase();

        let path = format!("exd/{new_filename}.exh");

        self.parsed::<EXH>(&path)
    }

    /// Read an excel sheet by name (e.g. "Achievement")
    pub fn read_excel_sheet(
        &mut self,
        exh: EXH,
        name: &str,
        language: Language,
    ) -> Result<ExcelSheet, Error> {
        let mut pages = Vec::new();
        for page in 0..exh.header.page_count {
            let exd = self.read_excel_exd(name, &exh, language, page as usize)?;
            pages.push(ExcelSheetPage::from_exd(page, &exh, exd));
        }

        Ok(ExcelSheet { exh, pages })
    }

    /// Returns all known sheet names listed in the root list
    pub fn get_all_sheet_names(&mut self) -> Result<Vec<String>, Error> {
        let root_exl = self.parsed::<EXL>("exd/root.exl")?;

        let mut names = vec![];
        for (row, _) in root_exl.entries {
            names.push(row);
        }

        Ok(names)
    }

    fn read_excel_exd(
        &mut self,
        name: &str,
        exh: &EXH,
        language: Language,
        page: usize,
    ) -> Result<EXD, Error> {
        let exd_path = format!(
            "exd/{}",
            EXD::calculate_filename(name, language, &exh.pages[page])
        );

        self.parsed::<EXD>(&exd_path)
    }
}
