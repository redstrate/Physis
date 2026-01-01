// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::ByteBuffer;

use super::Resource;

/// Allows chaining multiple Resources together.
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
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: Box<dyn Resource + Send + Sync>) {
        self.resolvers.push(source);
    }
}

impl Resource for ResourceResolver {
    fn read(&mut self, path: &str) -> Option<ByteBuffer> {
        for resolver in &mut self.resolvers {
            if let Some(bytes) = resolver.read(path) {
                return Some(bytes);
            }
        }

        None
    }

    fn exists(&mut self, path: &str) -> bool {
        for resolver in &mut self.resolvers {
            if resolver.exists(path) {
                return true;
            }
        }

        false
    }
}
