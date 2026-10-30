//! Where the running game keeps its state, as CPU addresses: what Link is
//! doing, where he is, what he holds, what he has seen and taken. For
//! programs that read the game from an emulator, such as an autotracker.
//!
//! Each address was measured in a headless emulator by doing one thing in
//! the game and comparing RAM before and after; docs/ram.md
//! says how. Both releases keep their state at the same addresses.

use crate::text::char_of_tile;

/// Work RAM, `$0000`-`$07FF`, holds everything below up to the room tables.
pub const WRAM_LEN: usize = 0x800;
/// Battery RAM copy of the six room tables of the overworld or of the
/// level Link is in (`z1::tables::Tables::new` reads it), the real ones
/// even in a race ROM.
pub const ROOM_TABLES: usize = 0x687e;
/// The block of the level Link is in, after the room tables and three
/// bytes; the same layout as `z1::level::Block`.
pub const LEVEL_BLOCK: usize = 0x6b81;

/// The level Link is in, 0 for the overworld.
pub const LEVEL: usize = 0x10;
/// The game mode; see the `MODE_*` values.
pub const MODE: usize = 0x12;
/// The step within the game mode: in a cave 0 while it is laid out, 1
/// from then on.
pub const SUBMODE: usize = 0x11;
/// One up per frame, the pause screen included; wraps at 256.
pub const FRAMES: usize = 0x15;
/// The save slot being played, 0-2.
pub const SLOT: usize = 0x16;
/// The item lying on the overworld screen: set to the coast item on every
/// screen, to the armos item when Link moves that statue.
pub const SCREEN_ITEM: usize = 0xab;
/// The overworld screen or dungeon room Link is in (row in the high
/// nibble); in a cave the overworld screen, down a staircase the room under
/// the level, whose tables A and B name the rooms at both ends.
pub const SCREEN: usize = 0xeb;
/// In a cave, 1, 2 and 3 while it fades in, 0 once its people and items
/// show.
pub const CAVE_FADE: usize = 0x407;
/// The three save slots' names, 8 font tiles each, padded with 36.
pub const NAMES: usize = 0x638;

/// The selected B item.
pub const B_ITEM: usize = 0x656;
/// Sword: 1 wood, 2 white, 3 magical.
pub const SWORD: usize = 0x657;
/// Bombs carried.
pub const BOMBS: usize = 0x658;
/// Arrows: 1 wood, 2 silver.
pub const ARROWS: usize = 0x659;
/// Bow, 1 when held.
pub const BOW: usize = 0x65a;
/// Candle: 1 blue, 2 red.
pub const CANDLE: usize = 0x65b;
/// Recorder, 1 when held.
pub const RECORDER: usize = 0x65c;
/// Bait, 1 when held.
pub const BAIT: usize = 0x65d;
/// Potion: 1 blue, 2 red.
pub const POTION: usize = 0x65e;
/// Wand, 1 when held.
pub const WAND: usize = 0x65f;
/// Raft, 1 when held.
pub const RAFT: usize = 0x660;
/// Book, 1 when held.
pub const BOOK: usize = 0x661;
/// Ring: 1 blue, 2 red.
pub const RING: usize = 0x662;
/// Ladder, 1 when held.
pub const LADDER: usize = 0x663;
/// Magical key, 1 when held.
pub const MAGICAL_KEY: usize = 0x664;
/// Power bracelet, 1 when held.
pub const POWER_BRACELET: usize = 0x665;
/// Letter, 1 when held.
pub const LETTER: usize = 0x666;
/// Compasses of levels 1-8, bit `level - 1`.
pub const COMPASSES: usize = 0x667;
/// Maps of levels 1-8, bit `level - 1`.
pub const MAPS: usize = 0x668;
/// Compass of level 9, bit 0.
pub const COMPASS_9: usize = 0x669;
/// Map of level 9, bit 0.
pub const MAP_9: usize = 0x66a;
/// Clock, 1 when held.
pub const CLOCK: usize = 0x66c;
/// Rupees carried.
pub const RUPEES: usize = 0x66d;
/// Keys carried.
pub const KEYS: usize = 0x66e;
/// Hearts: high nibble heart containers minus one, low nibble full hearts
/// minus one.
pub const HEARTS: usize = 0x66f;
/// Triforce pieces, bit `level - 1`.
pub const TRIFORCE: usize = 0x671;
/// Boomerang, 1 when held.
pub const BOOMERANG: usize = 0x674;
/// Magical boomerang, 1 when held.
pub const MAGICAL_BOOMERANG: usize = 0x675;
/// Magical shield, 1 when held.
pub const MAGICAL_SHIELD: usize = 0x676;
/// Most bombs Link can carry: 8, then 4 more per upgrade.
pub const MAX_BOMBS: usize = 0x67c;

/// One byte per overworld screen, `TAKEN` when its cave's item is taken.
pub const OVERWORLD_FLAGS: usize = 0x67f;
/// One byte per room, shared by levels 1-6.
pub const ROOM_FLAGS_1_6: usize = 0x6ff;
/// One byte per room, shared by levels 7-9.
pub const ROOM_FLAGS_7_9: usize = 0x77f;
/// Room flag: Link has been in the room.
pub const VISITED: u8 = 0x20;
/// Room and screen flag: the item is taken. Taking an item down a
/// staircase sets it in the cellar's room, also when Link already held it.
pub const TAKEN: u8 = 0x10;
/// Room flags: the east, west, south and north wall bombed open (bits 0-3),
/// set in the rooms on both sides.
pub const DOORS_OPEN: u8 = 0x0f;

/// Mode right after power-on.
pub const MODE_POWER_ON: u8 = 0xff;
/// Mode on the title screen.
pub const MODE_TITLE: u8 = 0x00;
/// Mode on the file select screen.
pub const MODE_FILE_SELECT: u8 = 0x01;
/// Mode while Link plays.
pub const MODE_PLAY: u8 = 0x05;
/// Mode in a cellar, down a staircase.
pub const MODE_CELLAR: u8 = 0x09;
/// Mode in a cave.
pub const MODE_CAVE: u8 = 0x0b;
/// Mode in a take-any-road cave, the warp.
pub const MODE_WARP: u8 = 0x0c;
/// Mode while registering a name.
pub const MODE_REGISTER: u8 = 0x0e;
/// Mode while Link dies.
pub const MODE_DYING: u8 = 0x11;
/// Mode from reaching Zelda through the credits.
pub const MODE_ENDING: u8 = 0x13;

/// Whether Link stands in a cave whose people and items show on the
/// screen, from work RAM read from `$0000`: in the cave's mode, laid out
/// and faded in. Before that the room tables already name what lies there.
/// False when `wram` is shorter than `WRAM_LEN`.
pub fn cave_shown(wram: &[u8]) -> bool {
    wram.len() >= WRAM_LEN && wram[MODE] == MODE_CAVE && wram[SUBMODE] != 0 && wram[CAVE_FADE] == 0
}

/// The room flags of `level`: levels 1-6 share one set, 7-9 the other.
/// `None` for the overworld and levels past 9.
pub fn room_flags(level: u8) -> Option<usize> {
    match level {
        1..=6 => Some(ROOM_FLAGS_1_6),
        7..=9 => Some(ROOM_FLAGS_7_9),
        _ => None,
    }
}

/// The name of the save slot being played, from work RAM read from `$0000`,
/// as the file select screen shows it. `None` when `wram` is shorter than
/// `WRAM_LEN`, the slot is past 2 or the name is blank.
pub fn player_name(wram: &[u8]) -> Option<String> {
    if wram.len() < WRAM_LEN {
        return None;
    }
    let slot = usize::from(wram[SLOT]);
    if slot > 2 {
        return None;
    }
    let at = NAMES + 8 * slot;
    let name: String = wram[at..at + 8]
        .iter()
        .map(|&t| char_of_tile(t).unwrap_or(' '))
        .collect();
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_of_the_slot_being_played() {
        let mut wram = vec![0x24; WRAM_LEN];
        wram[SLOT] = 1;
        wram[NAMES + 8..NAMES + 13].copy_from_slice(&[0x0a, 0x0b, 0x24, 0x01, 0x29]);
        assert_eq!(player_name(&wram).as_deref(), Some("AB 1!"));
        wram[SLOT] = 0;
        assert_eq!(player_name(&wram), None);
        wram[SLOT] = 3;
        assert_eq!(player_name(&wram), None);
        assert_eq!(player_name(&wram[..10]), None);
    }

    #[test]
    fn a_cave_shows_once_laid_out_and_faded_in() {
        let mut wram = vec![0; WRAM_LEN];
        wram[MODE] = MODE_CAVE;
        wram[CAVE_FADE] = 1;
        assert!(!cave_shown(&wram));
        wram[SUBMODE] = 1;
        assert!(!cave_shown(&wram));
        wram[CAVE_FADE] = 0;
        assert!(cave_shown(&wram));
        wram[MODE] = MODE_PLAY;
        assert!(!cave_shown(&wram));
        assert!(!cave_shown(&wram[..10]));
    }

    #[test]
    fn levels_share_their_room_flags_in_two_groups() {
        assert_eq!(room_flags(0), None);
        assert_eq!(room_flags(6), Some(ROOM_FLAGS_1_6));
        assert_eq!(room_flags(7), Some(ROOM_FLAGS_7_9));
        assert_eq!(room_flags(10), None);
    }
}
