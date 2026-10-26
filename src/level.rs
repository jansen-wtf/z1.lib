//! The levels in a ROM: their blocks, their maps and start rooms, which
//! rooms belong to a level, and what lies in them.

use std::collections::BTreeSet;

use crate::rom::Bytes;
use crate::tables::{self, Tables};

/// One block per place, the overworld's first, then levels 1-9: palettes,
/// start room, map, item positions. The game copies the current one to
/// battery RAM at `$6B81`.
pub const BLOCKS: usize = 0x19313;
/// The length of one block.
pub const BLOCK_LEN: usize = 0xfc;
/// In a block: the column the pause screen map starts at.
pub const MAP_START_COLUMN: usize = 0x2a;
/// In a block: the room Link enters the level in.
pub const START_ROOM: usize = 0x2c;
/// In a block: the pause screen map, one byte per column with row 0 in
/// bit 7.
pub const MAP: usize = 0x3c;

/// One block, from the ROM or from battery RAM.
pub type Block = [u8; BLOCK_LEN];

/// The block of `level`, 0 for the overworld.
///
/// # Panics
/// When `level` is past 9.
pub fn block(rom: Bytes<'_>, level: usize) -> &Block {
    let at = BLOCKS + BLOCK_LEN * level;
    rom.as_slice()[at..at + BLOCK_LEN].try_into().unwrap()
}

/// The room tables of `level` (1-9) in the ROM, of the first or second
/// quest. Levels 1-6 share theirs, as do levels 7-9; past 9 counts as
/// 7-9.
pub fn tables(rom: Bytes<'_>, level: usize, second_quest: bool) -> Tables<'_> {
    let group = usize::from(level >= 7) + if second_quest { 2 } else { 0 };
    Tables::in_rom(rom, tables::LEVELS + tables::LEN * group)
}

/// The room Link enters the level in.
pub fn start_room(block: &Block) -> u8 {
    block[START_ROOM]
}

/// The rooms on a level's map, the blue shape the game shows with the map
/// in hand.
pub fn map_rooms(block: &Block) -> BTreeSet<u8> {
    let shift = block[MAP_START_COLUMN] & 15;
    let mut out = BTreeSet::new();
    for d in 0..16u8 {
        for row in 0..8u8 {
            if block[MAP + usize::from(d)] >> (7 - row) & 1 != 0 {
                out.insert(row * 16 + (d.wrapping_sub(shift) & 15));
            }
        }
    }
    out
}

/// The rooms of a level: the ones on its map and its start room, and the
/// ones reached from them through doors and the passages under the rooms,
/// without entering a room in `others`, the maps of the levels it shares
/// its tables with. The map leaves out a room now and then (level 7's
/// staircase to the red candle, three rooms of level 9).
pub fn rooms(tables: Tables<'_>, block: &Block, others: &BTreeSet<u8>) -> BTreeSet<u8> {
    let mut links: Vec<(u8, u8)> = Vec::new();
    for r in 0..128u8 {
        let [a, b] = tables.stairs(r);
        if tables.is_cellar(r) && a != b && a < 128 && b < 128 {
            links.push((a, b));
            links.push((b, a));
        }
    }
    let mut rooms = map_rooms(block);
    rooms.insert(start_room(block));
    let mut todo: Vec<u8> = rooms.iter().copied().collect();
    while let Some(r) = todo.pop() {
        let (row, col) = (r >> 4, r & 15);
        let [north, south, west, east] = tables.doors(r);
        let doors = [
            (north, row > 0, r.wrapping_sub(16)),
            (south, row < 7, r + 16),
            (west, col > 0, r.wrapping_sub(1)),
            (east, col < 15, r + 1),
        ];
        let walls = doors
            .iter()
            .filter(|&&(door, ok, _)| ok && door != tables::WALL)
            .map(|&(_, _, n)| n);
        let below = links
            .iter()
            .filter(|&&(from, _)| from == r)
            .map(|&(_, to)| to);
        for n in walls.chain(below).collect::<Vec<_>>() {
            if !tables.is_cellar(n) && !others.contains(&n) && rooms.insert(n) {
                todo.push(n);
            }
        }
    }
    rooms
}

/// An item in a level: in a room, or in the cellar down the room's
/// staircase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// The room, or for an item down a staircase the room the staircase
    /// stands in.
    pub room: u8,
    /// The item id.
    pub item: u8,
    /// Whether the item is down the room's staircase.
    pub stairs: bool,
}

/// The items in `rooms`, on their floors and in their cellars.
pub fn items(tables: Tables<'_>, rooms: &BTreeSet<u8>) -> Vec<Placed> {
    (0..128u8)
        .filter_map(|r| {
            let stairs = tables.is_cellar(r);
            let room = if stairs {
                let [a, b] = tables.stairs(r);
                (a == b).then_some(a)?
            } else {
                r
            };
            let item = tables.item(r);
            (rooms.contains(&room) && item != tables::NO_ROOM_ITEM).then_some(Placed {
                room,
                item,
                stairs,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{HEART_CONTAINER, TRIFORCE};

    /// Needs the original ROM in Z1R_ROM.
    #[test]
    #[ignore = "needs the original ROM in Z1R_ROM"]
    fn the_original_levels_hold_their_known_items() {
        let data = std::fs::read(std::env::var_os("Z1R_ROM").expect("Z1R_ROM"))
            .expect("the ROM in Z1R_ROM");
        let rom = Bytes::new(&data).unwrap();
        let found = |level: usize| {
            let group = if level <= 6 { 1..=6 } else { 7..=9 };
            let others: BTreeSet<u8> = group
                .filter(|&l| l != level)
                .flat_map(|l| map_rooms(block(rom, l)))
                .collect();
            let t = tables(rom, level, false);
            let mut out: Vec<(u8, bool)> = items(t, &rooms(t, block(rom, level), &others))
                .into_iter()
                .filter(|p| ![HEART_CONTAINER, TRIFORCE].contains(&p.item) && p.item > 3)
                .map(|p| (p.item, p.stairs))
                .collect();
            out.sort();
            out
        };
        // Bow down the stairs, boomerang on the floor.
        assert!(found(1).contains(&(10, true)) && found(1).contains(&(29, false)));
        // The red candle's staircase room is missing from the pause map.
        assert!(found(7).contains(&(7, true)));
        // Magical key and book.
        assert!(found(8).contains(&(11, true)) && found(8).contains(&(17, true)));
        // Silver arrows and red ring.
        assert_eq!(
            found(9)
                .into_iter()
                .filter(|p| p.1)
                .map(|p| p.0)
                .collect::<Vec<_>>(),
            [9, 19]
        );
    }
}
