// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::ptr_arg)] // binrw trips up another false positive

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::equipment::Slot;
use crate::savedata::dat::DatHeader;
use binrw::NullString;
use binrw::binrw;
use binrw::{BinRead, BinWrite};
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;

// FIXME: unclear what this is
const UNKNOWN_FLAG: u32 = 1_000_000;

fn convert_from_gear_id(id: u32) -> u32 {
    id & !UNKNOWN_FLAG
}

fn convert_to_gear_id(id: &u32) -> u32 {
    id | UNKNOWN_FLAG
}

fn convert_to_string(s: NullString) -> String {
    s.to_string()
}

fn convert_from_string(s: &String) -> NullString {
    NullString::from(s.as_str())
}

const NUMBER_OF_GEARSETS: usize = 100;

fn convert_from_gearsets(gearsets: [GearSet; NUMBER_OF_GEARSETS]) -> Vec<Option<GearSet>> {
    gearsets
        .iter()
        .cloned()
        .map(|x| if !x.name.is_empty() { Some(x) } else { None })
        .collect()
}

fn convert_to_gearsets(gearsets: &Vec<Option<GearSet>>) -> Vec<GearSet> {
    let mut result = vec![GearSet::default(); NUMBER_OF_GEARSETS];
    for (i, gearset) in gearsets.iter().enumerate() {
        if i >= NUMBER_OF_GEARSETS {
            break;
        }
        if let Some(gearset) = gearset {
            result[i] = gearset.clone();
        }
    }
    result
}

const NUMBER_OF_GEARSLOTS: usize = 14;

fn convert_from_slots(slots: [GearSlot; NUMBER_OF_GEARSLOTS]) -> HashMap<GearSlotType, GearSlot> {
    slots
        .iter()
        .cloned()
        .enumerate()
        .filter_map(|(i, x)| match x.id {
            0 => None,
            _ => Some((i.try_into().ok()?, x)),
        })
        .collect()
}

fn convert_to_slots(slots: &HashMap<GearSlotType, GearSlot>) -> Vec<GearSlot> {
    let mut result = vec![GearSlot::default(); NUMBER_OF_GEARSLOTS];
    for (idx, slot) in slots.iter() {
        result[idx.clone() as usize] = slot.clone();
    }
    result
}

fn convert_id_opt(id: u32) -> Option<u32> {
    if id == 0 { None } else { Some(id) }
}

fn convert_opt_id(id: &Option<u32>) -> u32 {
    id.unwrap_or(0)
}

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct GearSlot {
    #[br(map = convert_from_gear_id)]
    #[bw(map = convert_to_gear_id)]
    /// The ID of the item.
    pub id: u32,
    #[br(map = convert_id_opt)]
    #[bw(map = convert_opt_id)]
    /// The ID of the item used as glamour.
    pub glamour_id: Option<u32>,
    // FIXME: one of those is most likely dyes, no idea about the rest
    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
    unknown4: u32,
    unknown5: u32,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum GearSlotType {
    MainHand = 0,
    SecondaryHand,
    Head,
    Body,
    Hands,
    Waist, // legacy
    Legs,
    Feet,
    Bracelets,
    Necklace,
    Earrings,
    Ring1,
    Ring2,
    Soul,
}

impl GearSlotType {
    pub fn to_slot(&self) -> Option<Slot> {
        match self {
            GearSlotType::Head => Some(Slot::Head),
            GearSlotType::Body => Some(Slot::Body),
            GearSlotType::Hands => Some(Slot::Hands),
            GearSlotType::Legs => Some(Slot::Legs),
            GearSlotType::Feet => Some(Slot::Feet),
            GearSlotType::Bracelets => Some(Slot::Wrists),
            GearSlotType::Necklace => Some(Slot::Neck),
            GearSlotType::Earrings => Some(Slot::Earring),
            GearSlotType::Ring1 => Some(Slot::RingLeft),
            GearSlotType::Ring2 => Some(Slot::RingRight),
            _ => None,
        }
    }
}

impl TryFrom<Slot> for GearSlotType {
    type Error = ();

    fn try_from(v: Slot) -> Result<Self, Self::Error> {
        match v {
            Slot::Head => Ok(GearSlotType::Head),
            Slot::Body => Ok(GearSlotType::Body),
            Slot::Hands => Ok(GearSlotType::Hands),
            Slot::Legs => Ok(GearSlotType::Legs),
            Slot::Feet => Ok(GearSlotType::Feet),
            Slot::Wrists => Ok(GearSlotType::Bracelets),
            Slot::Neck => Ok(GearSlotType::Necklace),
            Slot::Earring => Ok(GearSlotType::Earrings),
            Slot::RingLeft => Ok(GearSlotType::Ring1),
            Slot::RingRight => Ok(GearSlotType::Ring2),
        }
    }
}

impl TryFrom<usize> for GearSlotType {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == GearSlotType::MainHand as usize => Ok(GearSlotType::MainHand),
            x if x == GearSlotType::SecondaryHand as usize => Ok(GearSlotType::SecondaryHand),
            x if x == GearSlotType::Head as usize => Ok(GearSlotType::Head),
            x if x == GearSlotType::Body as usize => Ok(GearSlotType::Body),
            x if x == GearSlotType::Hands as usize => Ok(GearSlotType::Hands),
            x if x == GearSlotType::Waist as usize => Ok(GearSlotType::Waist),
            x if x == GearSlotType::Legs as usize => Ok(GearSlotType::Legs),
            x if x == GearSlotType::Feet as usize => Ok(GearSlotType::Feet),
            x if x == GearSlotType::Bracelets as usize => Ok(GearSlotType::Bracelets),
            x if x == GearSlotType::Necklace as usize => Ok(GearSlotType::Necklace),
            x if x == GearSlotType::Earrings as usize => Ok(GearSlotType::Earrings),
            x if x == GearSlotType::Ring1 as usize => Ok(GearSlotType::Ring1),
            x if x == GearSlotType::Ring2 as usize => Ok(GearSlotType::Ring2),
            x if x == GearSlotType::Soul as usize => Ok(GearSlotType::Soul),
            _ => Err(()),
        }
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct GearSet {
    /// The index of the gear set in the list of gear sets.
    pub index: u8,
    #[brw(pad_size_to = 47)]
    #[br(map = convert_to_string)]
    #[bw(map = convert_from_string)]
    /// The name of the gear set.
    pub name: String,
    // FIXME: no clue what this is
    unknown1: u64,
    #[br(map = convert_from_slots)]
    #[bw(map = convert_to_slots)]
    /// The slots of the gear set.
    pub slots: HashMap<GearSlotType, GearSlot>,
    #[br(map = convert_id_opt)]
    #[bw(map = convert_opt_id)]
    /// The ID of the facewear item.
    pub facewear: Option<u32>,
}

#[binrw]
#[br(little)]
#[derive(Debug, Clone)]
pub struct GearSets {
    // FIXME: can't be a version because it's always 0
    unknown1: u8,
    /// The index of the current active geat set.
    pub current_gearset: u8,
    // FIXME: no clue what this is
    unknown3: u16,
    #[br(map = convert_from_gearsets)]
    #[bw(map = convert_to_gearsets)]
    /// The list of gear sets.
    pub gearsets: Vec<Option<GearSet>>,
}

const GEARSET_KEY: u8 = 0x73;

impl GearSets {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<GearSets> {
        let mut cursor = Cursor::new(buffer);

        let header = DatHeader::read(&mut cursor).ok()?;

        let mut buffer = vec![0; header.content_size as usize - 1];
        cursor.read_exact(&mut buffer).ok()?;

        let decoded = buffer.iter().map(|x| *x ^ GEARSET_KEY).collect::<Vec<_>>();
        let mut cursor = Cursor::new(decoded);

        GearSets::read(&mut cursor).ok()
    }

    /// Writes data back to a buffer.
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        // header
        {
            let mut cursor = Cursor::new(&mut buffer);

            let header = DatHeader {
                file_type: crate::savedata::dat::DatFileType::Gearset,
                file_version: 109,
                max_size: 45205,
                content_size: 45205,
            };
            header.write_le(&mut cursor).ok()?
        }

        // buffer contents encoded
        {
            let mut cursor = Cursor::new(ByteBuffer::new());
            self.write_le(&mut cursor).ok()?;

            buffer.extend_from_slice(
                &cursor
                    .into_inner()
                    .iter()
                    .map(|x| *x ^ GEARSET_KEY)
                    .collect::<Vec<_>>(),
            );
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        GearSets::from_existing(&read(d).unwrap());
    }

    fn common_setup(name: &str) -> GearSets {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests/gearsets");
        d.push(name);

        GearSets::from_existing(&read(&d).unwrap()).unwrap()
    }

    #[test]
    fn read_simple() {
        let gearsets = common_setup("simple.dat");

        assert_eq!(gearsets.current_gearset, 0);
        let gearset = gearsets.gearsets[0].as_ref().unwrap();
        for i in 1..gearsets.gearsets.len() {
            assert!(gearsets.gearsets[i].is_none());
        }

        assert_eq!(gearset.index, 0);
        assert_eq!(gearset.name, "White Mage");
        assert!(gearset.facewear.is_none());
        assert_eq!(gearset.slots.len(), 2);
        let slot = gearset.slots.get(&GearSlotType::MainHand).unwrap();
        assert_eq!(slot.id, 5269);
        assert_eq!(slot.glamour_id, Some(2453));
        let slot = gearset.slots.get(&GearSlotType::Body).unwrap();
        assert_eq!(slot.id, 8395913);
        assert_eq!(slot.glamour_id, None);
    }

    #[test]
    fn write_simple() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests/gearsets");
        d.push("simple.dat");

        let gearset_bytes = &read(d).unwrap();
        let gearset = GearSets::from_existing(gearset_bytes).unwrap();
        assert_eq!(*gearset_bytes, gearset.write_to_buffer().unwrap());
    }
}
