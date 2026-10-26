//! What the Zelda Randomizer's changes to the code mean: what opens level
//! 9, what Link starts with, whether the book is an atlas.

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs the original ROM in Z1R_ROM; skipped without it.
    #[test]
    fn the_original_wants_eight_pieces() {
        let Some(data) = std::env::var_os("Z1R_ROM").and_then(|p| std::fs::read(p).ok()) else {
            return;
        };
        let rom = Bytes::new(&data).unwrap();
        assert_eq!(gate(rom), Gate::Count(8));
        assert_eq!(start_inventory(rom), None);
        assert!(!book_is_atlas(rom));
        assert_eq!(crate::overworld::coast_screen(rom), 0x5f);
    }
}
