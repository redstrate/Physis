// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use physis::{
    common::{Language, Platform},
    exd::{ColumnData, ExcelRowKind},
};

/// Test to see if we can find the root EXL. It exists in every version, and is a pretty safe indicator whether our SqPack reading works.
#[test]
fn test_gamedata_extract() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut game_data = physis::gamedata::GameData::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    )
    .unwrap();

    assert!(game_data.extract("exd/root.exl").is_some());
}

/// Test reading items, by finding the "Dated Canvas Beret", an item that existed since 2.x and should be on the first sheet page
#[test]
fn test_item_read() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut game_data = physis::gamedata::GameData::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    )
    .unwrap();

    let exh = game_data.read_excel_sheet_header("Item").unwrap();
    let exd = game_data
        .read_excel_sheet("Item", &exh, Language::English, 0)
        .unwrap();
    for row in exd.rows {
        match &row.kind {
            ExcelRowKind::SingleRow(row) => match &row.columns[9] {
                ColumnData::String(val) => {
                    if (val == "Dated Canvas Beret") {
                        return;
                    }
                }
                _ => panic!("Expected a string column!"),
            },
            _ => panic!("Expected a single row!"),
        }
    }

    panic!("Item not found!");
}
