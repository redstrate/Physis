// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    ByteBuffer, Error, Language, ReadableFile,
    excel::Sheet,
    exh::EXH,
    resource::{
        generic_get_all_sheet_names, generic_parsed, generic_read_excel_sheet,
        generic_read_excel_sheet_header,
    },
};

use super::Resource;

/// Allows layering multiple Resources on top of each other.
///
/// See the [module-level documentation](crate::resource) for more information.
///
/// # Example
///
/// ```
/// # use physis::resource::{ResourceResolver, SqPackResource, UnpackedResource, SqPackRelease};
/// # use physis::Platform;
/// let sqpack_source = SqPackResource::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game");
/// let file_source = UnpackedResource::from_existing("unpacked/");
/// let mut resolver = ResourceResolver::new();
/// resolver.add_source(file_source); // first has most priority
/// resolver.add_source(sqpack_source); // this is the fallback
/// ```
#[derive(Clone)]
pub struct ResourceResolver {
    resources: Vec<Box<dyn Resource>>,
}

impl ResourceResolver {
    /// Create a new, empty resolver.
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Adds a new source to this resolver, and makes it the least prioritized.
    pub fn add_source(&mut self, source: impl Resource) {
        self.resources.push(Box::new(source));
    }

    // TODO: add documentation
    pub fn read(&mut self, path: &str) -> crate::Result<ByteBuffer> {
        for resolver in &mut self.resources {
            if let Ok(bytes) = resolver.read(path) {
                return Ok(bytes);
            }
        }

        Err(crate::Error::FileNotFound {
            path: path.to_string(),
        })
    }

    /// Reads and parses the file located at `path`. This avoids having to call both `read` and `from_existing`.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::resource::{Resource, SqPackResource, SqPackRelease, ResourceResolver};
    /// # use physis::exl::EXL;
    /// # use std::io::Write;
    /// # use physis::Platform;
    /// let mut resolver = ResourceResolver::new();
    /// resolver.add_source(SqPackResource::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game"));
    ///
    /// let exl = resolver.parsed::<EXL>("exd/root.exl").unwrap();
    /// ```
    pub fn parsed<F: ReadableFile>(&mut self, path: &str) -> crate::Result<F> {
        self.execute_first_found(
            |resource| generic_parsed(resource, path),
            Error::FileNotFound {
                path: path.to_string(),
            },
        )
    }

    /// Read an excel sheet header by name (e.g. "Achievement").
    pub fn read_excel_sheet_header(&mut self, name: &str) -> crate::Result<EXH> {
        self.execute_first_found(
            |resource| generic_read_excel_sheet_header(resource, name),
            Error::ResolverFailed,
        )
    }

    /// Read an excel sheet by name (e.g. "Achievement").
    pub fn read_excel_sheet(
        &mut self,
        exh: &EXH,
        name: &str,
        language: Language,
    ) -> crate::Result<Sheet> {
        self.execute_first_found(
            |resource| generic_read_excel_sheet(resource, exh, name, language),
            Error::ResolverFailed,
        )
    }

    /// Returns all known sheet names listed in the root list.
    pub fn get_all_sheet_names(&mut self) -> crate::Result<Vec<String>> {
        self.execute_first_found(generic_get_all_sheet_names, Error::ResolverFailed)
    }

    // TODO: add documentation
    pub fn exists(&mut self, path: &str) -> bool {
        for resolver in &mut self.resources {
            if resolver.exists(path) {
                return true;
            }
        }

        false
    }

    /// Executes the given function `f`, continuing past "FileNotFound" errors and ultimately returns `error` if everything failed.
    fn execute_first_found<T, F>(&mut self, f: F, error: Error) -> crate::Result<T>
    where
        F: Fn(&mut dyn Resource) -> crate::Result<T>,
    {
        for resource in &mut self.resources {
            let result = f(resource.as_mut());
            match result {
                Ok(t) => return Ok(t),
                Err(err) => {
                    if let Error::FileNotFound { .. } = err {
                        continue; // continue even if the file wasn't found in *this* resolver
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        // TODO: maybe return the last error instead?
        Err(error)
    }
}
