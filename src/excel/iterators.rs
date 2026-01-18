// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::excel::{Page, Row, Sheet};

/// Iterator over a [Sheet].
#[derive(Clone)]
pub struct SheetIterator<'a> {
    sheet: &'a Sheet,
    page_index: u32,
    page_iterator: Option<PageIterator<'a>>,
}

impl<'a> IntoIterator for &'a Sheet {
    type Item = (u32, &'a [(u16, Row)]);
    type IntoIter = SheetIterator<'a>;

    fn into_iter(self) -> SheetIterator<'a> {
        SheetIterator {
            sheet: self,
            page_index: 0,
            page_iterator: None,
        }
    }
}

impl<'a> Iterator for SheetIterator<'a> {
    type Item = (u32, &'a [(u16, Row)]);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iterator) = &mut self.page_iterator
            && let Some(item) = iterator.next()
        {
            return Some(item);
        }

        let page_index = self.page_index;
        self.page_index += 1;

        if page_index >= self.sheet.pages.len() as u32 {
            return None;
        }

        self.page_iterator = Some(self.sheet.pages[page_index as usize].into_iter());

        if let Some(iterator) = &mut self.page_iterator {
            iterator.next()
        } else {
            None
        }
    }
}

/// Iterator over a [Page].
///
/// This includes subrows, if you know you don't have those then use [Self::flatten_subrows].
#[derive(Clone)]
pub struct PageIterator<'a> {
    page: &'a Page,
    row_index: u32,
}

impl<'a> PageIterator<'a> {
    /// Flattens this iterator, giving you one that only contains rows.
    ///
    /// If this sheet actually has subrows, then it only takes the first one in each row.
    pub fn flatten_subrows(&self) -> RowIterator<'a> {
        RowIterator {
            page: self.page,
            row_index: self.row_index,
        }
    }
}

impl<'a> IntoIterator for &'a Page {
    type Item = (u32, &'a [(u16, Row)]);
    type IntoIter = PageIterator<'a>;

    fn into_iter(self) -> PageIterator<'a> {
        PageIterator {
            page: self,
            row_index: 0,
        }
    }
}

impl<'a> Iterator for PageIterator<'a> {
    type Item = (u32, &'a [(u16, Row)]);

    fn next(&mut self) -> Option<Self::Item> {
        let row_index = self.row_index;
        self.row_index += 1;

        if row_index as usize >= self.page.row_count() {
            return None;
        }

        let row = &self.page.entries.get(row_index as usize)?;

        Some((row.id, row.subrows.as_slice()))
    }
}

/// Iterator over an [Page], but only the rows.
///
/// To create this iterator, use [PageIterator::flatten_subrows].
#[derive(Clone)]
pub struct RowIterator<'a> {
    page: &'a Page,
    row_index: u32,
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = (u32, &'a Row);

    fn next(&mut self) -> Option<Self::Item> {
        let row_index = self.row_index;
        self.row_index += 1;

        if row_index as usize >= self.page.row_count() {
            return None;
        }

        let row = &self.page.entries.get(row_index as usize)?;

        Some((row.id, &row.subrows.first()?.1))
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadableFile;
    use crate::common::Platform;
    use crate::excel::{Field, Row, Sheet};
    use crate::exd::EXD;
    use crate::exh::EXH;
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_simple_iterators() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop.exh");

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop_1441792.exd");

            exd = EXD::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        let page = Page::from_exd(&exh, exd);

        let excel = Sheet {
            exh,
            pages: vec![page],
        };

        assert_eq!(excel.pages[0].entries.len(), 4);

        // page behavior
        let expected_iterator: Vec<(u32, &[(u16, Row)])> = excel.into_iter().collect();
        assert_eq!(
            expected_iterator,
            vec![
                (
                    1441792u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(0)]
                        }
                    )]
                    .as_slice()
                ),
                (
                    1441793u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(1)]
                        }
                    )]
                    .as_slice()
                ),
                (
                    1441794u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(2)]
                        }
                    )]
                    .as_slice()
                ),
                (
                    1441795u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(3)]
                        }
                    )]
                    .as_slice()
                )
            ]
        );

        // normal behavior that includes subrows
        let expected_iterator: Vec<(u32, &[(u16, Row)])> = excel.pages[0].into_iter().collect();
        assert_eq!(
            expected_iterator,
            vec![
                (
                    1441792u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(0)]
                        }
                    )]
                    .as_slice()
                ),
                (
                    1441793u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(1)]
                        }
                    )]
                    .as_slice()
                ),
                (
                    1441794u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(2)]
                        }
                    )]
                    .as_slice()
                ),
                (
                    1441795u32,
                    [(
                        0u16,
                        Row {
                            columns: vec![Field::Int8(3)]
                        }
                    )]
                    .as_slice()
                )
            ]
        );

        // API for row-only sheets
        let expected_iterator: Vec<(u32, &Row)> =
            excel.pages[0].into_iter().flatten_subrows().collect();
        assert_eq!(
            expected_iterator,
            vec![
                (
                    1441792u32,
                    &Row {
                        columns: vec![Field::Int8(0)]
                    },
                ),
                (
                    1441793u32,
                    &Row {
                        columns: vec![Field::Int8(1)]
                    },
                ),
                (
                    1441794u32,
                    &Row {
                        columns: vec![Field::Int8(2)]
                    },
                ),
                (
                    1441795u32,
                    &Row {
                        columns: vec![Field::Int8(3)]
                    },
                )
            ]
        );
    }

    #[test]
    fn test_multiple_pages() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop.exh");

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop_1441792.exd");

            exd = EXD::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        let page = Page::from_exd(&exh, exd);

        let excel = Sheet {
            exh,
            pages: vec![page.clone(), page],
        };

        // page behavior
        let expected_iterator: Vec<(u32, &[(u16, Row)])> = excel.into_iter().collect();
        assert_eq!(expected_iterator.len(), 8);
    }
}
