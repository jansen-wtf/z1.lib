//! What the Zelda Randomizer's changes to the code mean: what opens level
//! 9, what Link starts with, whether the book is an atlas; and what its
//! flags fix for every seed: how many item places each level has.

use crate::flags::{Check, Flags, check, combo};
use crate::rom::Bytes;

/// Level 9's gate: `LDA $0671; CMP #$FF` in the original.
pub const GATE: usize = 0x4ab1;
/// z1r's own gate check, when the gate jumps to `$9F00`.
pub const GATE_CHECK: usize = 0x5f10;
/// `JMP $A830` here when z1r sets a starting inventory.
pub const START_JUMP: usize = 0x9f4e;
/// The starting inventory, which the game copies to `$0657`.
pub const START_INVENTORY: usize = 0xa850;
/// Its length.
pub const START_INVENTORY_LEN: usize = 40;
/// `JMP $B8A0` here when the book is an atlas (`39 BE E6` in the original).
pub const BOOK_ATLAS: usize = 0x17614;

/// Inventory bytes (`$06xx`) the gate may ask for, as item ids.
const GATE_ITEMS: [(u8, u8); 12] = [
    (0x57, 1),
    (0x5c, 5),
    (0x5b, 6),
    (0x59, 8),
    (0x5a, 10),
    (0x64, 11),
    (0x60, 12),
    (0x63, 13),
    (0x5f, 16),
    (0x61, 17),
    (0x65, 20),
    (0x66, 21),
];

/// What opens level 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Gate {
    /// This many triforce pieces.
    Count(u8),
    /// These pieces, bit `level - 1`.
    Pieces(u8),
    /// An item instead of triforces.
    Item(u8),
    /// No gate.
    Open,
}

/// What opens level 9 in this ROM.
pub fn gate(rom: Bytes<'_>) -> Gate {
    if rom[GATE + 3..GATE + 7] == [0xea; 4] {
        return Gate::Open;
    }
    // JSR $9F00: z1r's own check.
    if rom[GATE..GATE + 3] == [0x20, 0x00, 0x9f] {
        return match rom[GATE_CHECK] {
            0xad => Gate::Pieces(rom[GATE_CHECK + 4]),
            _ => Gate::Count(rom[GATE_CHECK + 23]),
        };
    }
    match GATE_ITEMS.iter().find(|&&(at, _)| at == rom[GATE + 1]) {
        Some(&(_, item)) => Gate::Item(item),
        None => Gate::Count(8),
    }
}

/// The starting inventory as copied to `$0657`, when z1r set one.
pub fn start_inventory(rom: Bytes<'_>) -> Option<[u8; START_INVENTORY_LEN]> {
    (rom[START_JUMP..START_JUMP + 3] == [0x4c, 0x30, 0xa8]).then(|| {
        rom[START_INVENTORY..START_INVENTORY + START_INVENTORY_LEN]
            .try_into()
            .unwrap()
    })
}

/// Whether the book shows every level's map.
pub fn book_is_atlas(rom: Bytes<'_>) -> bool {
    rom[BOOK_ATLAS..BOOK_ATLAS + 3] == [0x4c, 0xa0, 0xb8]
}

/// Items with a place counted by `item_places`: heart containers and the
/// items a player collects, from the wood sword to the magical boomerang;
/// keys, maps, compasses, rupees, bombs and the like are not.
pub const PLACED_ITEMS: [u8; 22] = [
    1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 16, 17, 18, 19, 20, 21, 26, 28, 29, 30,
];

/// Places on the floor and down staircases, levels 1-9, for the first quest
/// and shapes.
const FIRST_QUEST_PLACES: [(u8, u8); 9] = [
    (2, 1),
    (2, 0),
    (1, 1),
    (1, 1),
    (1, 1),
    (1, 1),
    (1, 1),
    (1, 2),
    (0, 2),
];
/// The same for the second quest.
const SECOND_QUEST_PLACES: [(u8, u8); 9] = [
    (2, 0),
    (2, 0),
    (1, 1),
    (1, 1),
    (1, 2),
    (1, 1),
    (1, 2),
    (1, 1),
    (0, 2),
];

/// How many `PLACED_ITEMS` each level 1-9 holds on its floor and down its
/// staircases, where `flags` fix it for every seed: dungeon quest first or
/// second quest or shapes (shapes as the first quest), whatever the room
/// and item shuffle, unless minor drops join an item shuffle. `None` where
/// the seed decides: mixed quests, minor drops in an item shuffle.
///
/// Measured with the Zelda Randomizer's generator (z1r.rs): 12 seeds of
/// every combination of dungeon quest, room shuffle, item shuffle and minor
/// drops, counting the items in each level's rooms of the finished ROM.
pub fn item_places(flags: &Flags) -> Option<[(u8, u8); 9]> {
    let places = match flags.combo(combo::DUNGEON_QUEST) {
        0 | 3 => FIRST_QUEST_PLACES,
        1 => SECOND_QUEST_PLACES,
        _ => return None,
    };
    let fixed = flags.check(check::MINOR_DUNGEON_DROPS) == Check::Off
        || flags.combo(combo::SHUFFLE_ITEMS) == 0;
    fixed.then_some(places)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flags_fix_the_item_places_unless_minor_drops_join_a_shuffle() {
        let places = |dungeons, minor, items| {
            let mut f = Flags::default();
            f.set_combo(combo::DUNGEON_QUEST, dungeons);
            f.set_check(check::MINOR_DUNGEON_DROPS, minor);
            f.set_combo(combo::SHUFFLE_ITEMS, items);
            item_places(&f).map(|p| p[0])
        };
        assert_eq!(places(0, Check::Off, 3), Some((2, 1)));
        assert_eq!(places(3, Check::Off, 3), Some((2, 1)));
        assert_eq!(places(1, Check::Off, 3), Some((2, 0)));
        assert_eq!(places(3, Check::On, 0), Some((2, 1)));
        assert_eq!(places(2, Check::Off, 0), None);
        assert_eq!(places(3, Check::On, 1), None);
        // The 2026 Fall Tournament's flags: shapes, full room shuffle.
        let fall = Flags::parse("SQ8gqsY5mwtBby7Zo3Mu5TQQ1XhJWDIOUGvInVj");
        assert_eq!(item_places(&fall), Some(FIRST_QUEST_PLACES));
    }

    /// Needs the original ROM in Z1R_ROM.
    #[test]
    #[ignore = "needs the original ROM in Z1R_ROM"]
    fn the_original_wants_eight_pieces() {
        let data = std::fs::read(std::env::var_os("Z1R_ROM").expect("Z1R_ROM"))
            .expect("the ROM in Z1R_ROM");
        let rom = Bytes::new(&data).unwrap();
        assert_eq!(gate(rom), Gate::Count(8));
        assert_eq!(start_inventory(rom), None);
        assert!(!book_is_atlas(rom));
        assert_eq!(crate::overworld::coast_screen(rom), 0x5f);
    }
}
