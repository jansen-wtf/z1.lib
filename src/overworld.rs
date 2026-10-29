//! The overworld in a ROM: its caves, the coast and the armos statues.

use std::collections::BTreeSet;

use crate::item::HEART_CONTAINER;
use crate::rom::Bytes;
use crate::tables::Tables;

/// The coast screen: the operand of `CPX #$5F` in the code; a mirrored
/// overworld flips it to 0x50.
pub const COAST_SCREEN: usize = 0x178aa;
/// The item on the coast: the operand of `LDA #$1A`.
pub const COAST_ITEM: usize = 0x1789a;
/// Where the coast item lies on its screen, x in pixels; measured in the
/// game, for the unmirrored overworld.
pub const COAST_X: i32 = 200;
/// Where the coast item lies on its screen, y in pixels.
pub const COAST_Y: i32 = 80;
/// The screens of the armos statues that hide stairs, the item's one first
/// (bit 7 is no part of the screen).
pub const ARMOS_SCREENS: usize = 0x10cc2;
/// Their x positions, in pixels, in the same order.
pub const ARMOS_X: usize = 0x10cc9;
/// The item under the first armos.
pub const ARMOS_ITEM: usize = 0x10d05;
/// First tile of an armos statue.
pub const ARMOS_TILE: u8 = 0xc0;

/// The cave where Link takes one of three things.
pub const TAKE_ANY_CAVE: u8 = 17;
/// The cave with the white sword.
pub const WHITE_SWORD_CAVE: u8 = 18;
/// The hearts the white sword cave asks for, as `(hearts - 1) * 16`: the
/// operand the game compares the heart containers with.
pub const WHITE_SWORD_HEARTS: usize = 0x490d;
/// The hearts the magical sword cave asks for, the same way.
pub const MAGICAL_SWORD_HEARTS: usize = 0x4916;

/// What a cave id on the overworld is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CaveKind {
    /// The entrance of a level, 1-9.
    Level(u8),
    /// The old man with the wood sword (16).
    WoodSword,
    /// Take any one of three things (17).
    TakeAny,
    /// The white sword (18).
    WhiteSword,
    /// The magical sword (19).
    MagicalSword,
    /// Take any road: the warp stairs (20).
    TakeAnyRoad,
    /// An old man or woman with a hint (21, 25).
    Hint,
    /// The money making game (22).
    MoneyMakingGame,
    /// Pay for the door repair (23).
    DoorRepair,
    /// Give the letter to the old woman (24).
    Letter,
    /// The potion shop (26).
    Potion,
    /// Pay for a hint: the waterfall's (27) or the lost woods' (28).
    PayForHint,
    /// A shop, 1-4 (29-32).
    Shop(u8),
    /// Secret rupees (33-35).
    SecretRupees,
}

/// What cave `id` is, `None` for ids no cave has.
pub fn cave_kind(id: u8) -> Option<CaveKind> {
    Some(match id {
        1..=9 => CaveKind::Level(id),
        16 => CaveKind::WoodSword,
        17 => CaveKind::TakeAny,
        18 => CaveKind::WhiteSword,
        19 => CaveKind::MagicalSword,
        20 => CaveKind::TakeAnyRoad,
        21 | 25 => CaveKind::Hint,
        22 => CaveKind::MoneyMakingGame,
        23 => CaveKind::DoorRepair,
        24 => CaveKind::Letter,
        26 => CaveKind::Potion,
        27 | 28 => CaveKind::PayForHint,
        29..=32 => CaveKind::Shop(id - 28),
        33..=35 => CaveKind::SecretRupees,
        _ => return None,
    })
}

/// The hearts the white sword cave asks for.
pub fn white_sword_hearts(rom: Bytes<'_>) -> u8 {
    rom[WHITE_SWORD_HEARTS] / 16 + 1
}

/// The hearts the magical sword cave asks for.
pub fn magical_sword_hearts(rom: Bytes<'_>) -> u8 {
    rom[MAGICAL_SWORD_HEARTS] / 16 + 1
}

/// A screen's name on the overworld's grid of 8 rows and 16 columns: row
/// letter A-H, column 1-16, as in "H8" for the start screen 0x77.
///
/// # Panics
/// When `screen` is past 0x7F.
pub fn screen_name(screen: u8) -> String {
    assert!(screen < 0x80, "screen {screen:#x} is past the overworld");
    format!("{}{}", char::from(b'A' + (screen >> 4)), (screen & 15) + 1)
}

/// Two screens the tables list with a cave that the second quest's
/// overworld has none on.
const NO_CAVE_IN_SECOND_QUEST: [u8; 2] = [11, 66];

/// The screen the coast item lies on.
pub fn coast_screen(rom: Bytes<'_>) -> u8 {
    rom[COAST_SCREEN]
}

/// The item on the coast.
pub fn coast_item(rom: Bytes<'_>) -> u8 {
    rom[COAST_ITEM] & 0x3f
}

/// The screen of the armos statue with the item.
pub fn armos_screen(rom: Bytes<'_>) -> u8 {
    rom[ARMOS_SCREENS] & 0x7f
}

/// The item under the armos.
pub fn armos_item(rom: Bytes<'_>) -> u8 {
    rom[ARMOS_ITEM] & 0x3f
}

/// The cave of each screen that has one in this quest, as z1r's spoiler
/// map counts them: screen and cave id. For an overworld that is not
/// mirrored; `caves_of` takes the ROM and knows.
pub fn caves(tables: Tables<'_>, second_quest: bool) -> Vec<(u8, u8)> {
    entrances(tables, second_quest, false)
}

/// Like `caves`, for the overworld of `rom`, mirrored or not: the two
/// screens the second quest leaves out are mirrored with it.
pub fn caves_of(rom: Bytes<'_>, tables: Tables<'_>, second_quest: bool) -> Vec<(u8, u8)> {
    entrances(tables, second_quest, is_mirrored(rom))
}

fn entrances(tables: Tables<'_>, second_quest: bool, mirrored: bool) -> Vec<(u8, u8)> {
    let flip = if mirrored { 15 } else { 0 };
    (0..128u8)
        .filter(|&s| tables.cave_counts(s))
        .filter(|s| !(second_quest && NO_CAVE_IN_SECOND_QUEST.contains(&(s ^ flip))))
        .map(|s| (s, tables.cave(s)))
        .filter(|&(_, id)| (1..=35).contains(&id))
        .collect()
}

/// Whether the overworld of `rom` is mirrored left to right: mirroring
/// turns the coast's screen from 0x5F to 0x50, and nothing else writes it.
pub fn is_mirrored(rom: Bytes<'_>) -> bool {
    rom[COAST_SCREEN] == 0x50
}

/// The screens where something is to be found: a cave, the coast, the
/// armos item.
pub fn secrets(rom: Bytes<'_>, tables: Tables<'_>, second_quest: bool) -> BTreeSet<u8> {
    let mut out: BTreeSet<u8> = caves_of(rom, tables, second_quest)
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    out.insert(coast_screen(rom));
    out.insert(armos_screen(rom));
    out
}

/// The heart containers on the overworld: in caves, on the coast, under
/// the armos.
pub fn heart_containers(rom: Bytes<'_>, tables: Tables<'_>, second_quest: bool) -> usize {
    let in_caves = caves_of(rom, tables, second_quest)
        .into_iter()
        .filter_map(|(_, id)| tables.cave_items(id))
        .filter(|items| items.contains(&HEART_CONTAINER))
        .count();
    let fixed = [coast_item(rom), armos_item(rom)]
        .iter()
        .filter(|&&i| i == HEART_CONTAINER)
        .count();
    in_caves + fixed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom_with(edit: impl Fn(&mut Vec<u8>)) -> Vec<u8> {
        let mut data = vec![0; crate::rom::FILE_LEN];
        data[..4].copy_from_slice(b"NES\x1a");
        edit(&mut data);
        data
    }

    #[test]
    fn screens_are_named_by_row_and_column() {
        assert_eq!(screen_name(0x00), "A1");
        assert_eq!(screen_name(0x77), "H8");
        assert_eq!(screen_name(0x7f), "H16");
    }

    #[test]
    fn cave_ids_by_kind() {
        assert_eq!(cave_kind(3), Some(CaveKind::Level(3)));
        assert_eq!(cave_kind(WHITE_SWORD_CAVE), Some(CaveKind::WhiteSword));
        assert_eq!(cave_kind(32), Some(CaveKind::Shop(4)));
        assert_eq!(cave_kind(0), None);
        assert_eq!(cave_kind(36), None);
    }

    #[test]
    fn a_mirrored_second_quest_skips_the_mirrored_screens() {
        // Table B holds a cave id in bits 2-7, table D's bit 7 hides it.
        let mut tables = vec![0u8; crate::tables::LEN];
        for screen in [4u8, 11, 66, 77] {
            tables[crate::tables::B + usize::from(screen)] = 5 << 2;
        }
        let tables = Tables::new(&tables).unwrap();
        let plain = rom_with(|d| d[COAST_SCREEN] = 0x5f);
        let mirrored = rom_with(|d| d[COAST_SCREEN] = 0x50);
        let screens = |rom: &[u8]| -> Vec<u8> {
            caves_of(Bytes::new(rom).unwrap(), tables, true)
                .into_iter()
                .map(|(s, _)| s)
                .collect()
        };
        assert_eq!(screens(&plain), [4, 77]);
        assert_eq!(screens(&mirrored), [11, 66]);
        assert!(!is_mirrored(Bytes::new(&plain).unwrap()));
    }

    #[test]
    #[ignore = "needs the original ROM in Z1R_ROM"]
    fn the_original_sword_caves_ask_for_5_and_12_hearts() {
        let data = std::fs::read(std::env::var_os("Z1R_ROM").expect("Z1R_ROM"))
            .expect("the ROM in Z1R_ROM");
        let rom = Bytes::new(&data).unwrap();
        assert_eq!(white_sword_hearts(rom), 5);
        assert_eq!(magical_sword_hearts(rom), 12);
        assert!(!is_mirrored(rom));
    }
}
