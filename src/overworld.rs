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
/// map counts them: screen and cave id.
pub fn caves(tables: Tables<'_>, second_quest: bool) -> Vec<(u8, u8)> {
    (0..128u8)
        .filter(|&s| tables.cave_counts(s))
        .filter(|s| !(second_quest && NO_CAVE_IN_SECOND_QUEST.contains(s)))
        .map(|s| (s, tables.cave(s)))
        .filter(|&(_, id)| (1..=35).contains(&id))
        .collect()
}

/// The screens where something is to be found: a cave, the coast, the
/// armos item.
pub fn secrets(rom: Bytes<'_>, tables: Tables<'_>, second_quest: bool) -> BTreeSet<u8> {
    let mut out: BTreeSet<u8> = caves(tables, second_quest)
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
    let in_caves = caves(tables, second_quest)
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
