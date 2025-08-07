// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use physis::{
    common::{Language, Platform},
    exd::{ColumnData, ExcelRowKind},
    race::{Gender, Race, Tribe, build_skeleton_path},
    skeleton::Skeleton,
    resource::{Resource, SqPackResource},
};

/// Test to see if we can find the root EXL. It exists in every version, and is a pretty safe indicator whether our SqPack reading works.
#[test]
fn test_gamedata_extract() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut game_data = SqPackResource::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    );

    assert!(game_data.read("exd/root.exl").is_some());
}

/// Test reading items, by finding the "Dated Canvas Beret", an item that existed since 2.x and should be on the first sheet page
#[test]
fn test_item_read() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();
    let is_benchmark = env::var("FFXIV_IS_BENCHMARK").unwrap_or_default();
    if is_benchmark == "1" {
        // Skip this test because this benchmarks don't have item names.
        return;
    }

    let mut game_data = SqPackResource::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    );

    let exh = physis::resource::read_excel_sheet_header(&mut game_data, "Item").unwrap();
    let exd = physis::resource::read_excel_sheet(&mut game_data, "Item", &exh, Language::English, 0).unwrap();
    for row in exd.rows {
        match &row.kind {
            ExcelRowKind::SingleRow(row) => match &row.columns[9] {
                ColumnData::String(val) => {
                    if val == "Dated Canvas Beret" {
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

/// Test to see if we can parse Havok skeletons.
#[test]
fn test_parse_skeleton() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut game_data = SqPackResource::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    );

    let sklb_path = build_skeleton_path(Race::Hyur, Tribe::Midlander, Gender::Female);
    let sklb = game_data.read(&sklb_path).unwrap();
    let skeleton = Skeleton::from_existing(&sklb).unwrap();
    for bone in &skeleton.bones {
        if bone.name == "j_kosi" {
            return;
        }
    }

    panic!("Could not find bone!");
}
