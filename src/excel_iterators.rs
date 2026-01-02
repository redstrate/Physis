// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::excel::{ExcelRowKind, ExcelSheetPage};

pub struct ExcelSheetPageIterator<'a> {
    page: &'a ExcelSheetPage,
    row_index: u32,
}

impl<'a> IntoIterator for &'a ExcelSheetPage {
    type Item = (u32, &'a ExcelRowKind);
    type IntoIter = ExcelSheetPageIterator<'a>;
    fn into_iter(self) -> ExcelSheetPageIterator<'a> {
        ExcelSheetPageIterator {
            page: self,
            row_index: 0,
        }
    }
}

impl<'a> Iterator for ExcelSheetPageIterator<'a> {
    type Item = (u32, &'a ExcelRowKind);

    fn next(&mut self) -> Option<Self::Item> {
        let row_index = self.row_index;
        self.row_index += 1;

        if row_index > self.page.row_count {
            return None;
        }

        let row = &self.page.rows.get(row_index as usize)?;

        Some((row.row_id, &row.kind))
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadableFile;
    use crate::common::Platform;
    use crate::excel::{ColumnData, ExcelSheet, ExcelSingleRow};
    use crate::exd::EXD;
    use crate::exh::EXH;
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    // super simple EXD to read, it's just a few rows of only int8's
    #[test]
    fn test_read() {
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

        let page = ExcelSheetPage::from_exd(0, &exh, exd);

        let excel = ExcelSheet {
            exh,
            pages: vec![page],
        };

        assert_eq!(excel.pages[0].rows.len(), 4);

        let expected_iterator: Vec<(u32, &ExcelRowKind)> = excel.pages[0].into_iter().collect();
        assert_eq!(
            expected_iterator,
            vec![
                (
                    1441792,
                    &ExcelRowKind::SingleRow(ExcelSingleRow {
                        columns: vec![ColumnData::Int8(0)]
                    })
                ),
                (
                    1441793,
                    &ExcelRowKind::SingleRow(ExcelSingleRow {
                        columns: vec![ColumnData::Int8(1)]
                    })
                ),
                (
                    1441794,
                    &ExcelRowKind::SingleRow(ExcelSingleRow {
                        columns: vec![ColumnData::Int8(2)]
                    })
                ),
                (
                    1441795,
                    &ExcelRowKind::SingleRow(ExcelSingleRow {
                        columns: vec![ColumnData::Int8(3)]
                    })
                )
            ]
        );
    }
}
