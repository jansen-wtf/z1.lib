//! The six room tables A-F that the game keeps for the overworld and for
//! each group of levels: one byte per screen or room in each table, in the
//! ROM and, for the place Link is in, copied to battery RAM at `$687E`.

use crate::rom::Bytes;

/// The six tables together.
pub const LEN: usize = 0x300;
/// Table A: in levels the north and south doors, under a room the room of
/// one staircase.
pub const A: usize = 0x000;
/// Table B: in levels the west and east doors, under a room the room of the
/// other staircase; on the overworld the screen's cave.
pub const B: usize = 0x080;
/// Table C: in levels the room's monster code in bits 0-5, with bit 7 of
/// table D for the codes from 0x40 on.
pub const C: usize = 0x100;
/// Table D: in levels the room's layout.
pub const D: usize = 0x180;
/// Table E: in levels the room's item; on the overworld, from cave 16 on,
/// the caves' items and prices.
pub const E: usize = 0x200;
/// Table F: on the overworld, bit 7 set when the screen's cave does not
/// count in this quest.
pub const F: usize = 0x280;

/// The overworld's tables in the ROM.
pub const OVERWORLD: usize = 0x18410;
/// The levels' tables in the ROM: levels 1-6, then levels 7-9, of the first
/// quest, then the same of the second quest, `LEN` apart.
pub const LEVELS: usize = 0x18710;

/// A room's item (table E) when it has none.
pub const NO_ROOM_ITEM: u8 = 3;
/// A cave's item when it has none.
pub const NO_CAVE_ITEM: u8 = 0x3f;
/// The room layout (table D) of a cellar with an item down a staircase.
pub const ITEM_CELLAR: u8 = 0x3e;
/// The room layout of a passage between two rooms, down a staircase.
pub const PASSAGE: u8 = 0x3f;
/// The door type (three bits in table A or B) of a plain wall.
pub const WALL: u8 = 1;
/// The door type of a wall to bomb.
pub const BOMB_WALL: u8 = 4;

/// Overworld: per cave from cave 16 on, its three items.
pub const CAVE_ITEMS: usize = 0x200;
/// Overworld: per cave from cave 16 on, the prices of its three items.
pub const CAVE_PRICES: usize = 0x23c;
const FIRST_CAVE_WITH_ITEMS: u8 = 16;
const LAST_CAVE: u8 = 35;

/// One set of six tables, of the overworld or of a group of levels.
#[derive(Clone, Copy)]
pub struct Tables<'a>(&'a [u8; LEN]);

impl<'a> Tables<'a> {
    /// The tables at the start of `data`, as the game copies them to
    /// battery RAM; `None` when `data` is shorter.
    pub fn new(data: &'a [u8]) -> Option<Tables<'a>> {
        data.get(..LEN)?.try_into().ok().map(Tables)
    }

    /// The tables in a ROM at `at`: `OVERWORLD`, or one of `LEVELS`.
    ///
    /// # Panics
    /// When `at` is no place in the ROM with six tables after it.
    pub fn in_rom(rom: Bytes<'a>, at: usize) -> Tables<'a> {
        Tables(rom.as_slice()[at..at + LEN].try_into().unwrap())
    }

    /// The six tables, `LEN` bytes.
    pub fn as_slice(&self) -> &'a [u8] {
        self.0
    }

    /// The byte of screen or room `at` (0x00-0x7F) in table `table`.
    pub fn byte(&self, table: usize, at: u8) -> u8 {
        self.0[table + usize::from(at & 0x7f)]
    }

    /// The item in a level's room, or `NO_ROOM_ITEM`.
    pub fn item(&self, room: u8) -> u8 {
        self.byte(E, room) & 0x1f
    }

    /// A level room's layout: its walls inside, or `ITEM_CELLAR` or
    /// `PASSAGE` for the rooms down a staircase.
    pub fn layout(&self, room: u8) -> u8 {
        self.byte(D, room) & 0x3f
    }

    /// Whether `room` is one a staircase leads down into.
    pub fn is_cellar(&self, room: u8) -> bool {
        matches!(self.layout(room), ITEM_CELLAR | PASSAGE)
    }

    /// Under a room: the rooms the staircases at both ends stand in, the
    /// same room twice for a cellar with an item.
    pub fn stairs(&self, cellar: u8) -> [u8; 2] {
        [self.byte(A, cellar), self.byte(B, cellar)]
    }

    /// The door types of a room's walls: north, south, west, east.
    pub fn doors(&self, room: u8) -> [u8; 4] {
        let (a, b) = (self.byte(A, room), self.byte(B, room));
        [a >> 5, (a >> 2) & 7, b >> 5, (b >> 2) & 7]
    }

    /// The cave an overworld screen leads into, 0 for none.
    pub fn cave(&self, screen: u8) -> u8 {
        self.byte(B, screen) >> 2
    }

    /// Whether the overworld screen's cave counts in this quest: z1r's
    /// spoiler map leaves out the ones with bit 7 of table F set.
    pub fn cave_counts(&self, screen: u8) -> bool {
        self.byte(F, screen) & 0x80 == 0
    }

    /// The three items a cave offers, `NO_CAVE_ITEM` for none; caves 16-35.
    pub fn cave_items(&self, cave: u8) -> Option<[u8; 3]> {
        let at = CAVE_ITEMS + self.cave_index(cave)?;
        Some([0, 1, 2].map(|k| self.0[at + k] & 0x3f))
    }

    /// The prices of a cave's three items; caves 16-35.
    pub fn cave_prices(&self, cave: u8) -> Option<[u8; 3]> {
        let at = CAVE_PRICES + self.cave_index(cave)?;
        Some([0, 1, 2].map(|k| self.0[at + k]))
    }

    fn cave_index(&self, cave: u8) -> Option<usize> {
        (FIRST_CAVE_WITH_ITEMS..=LAST_CAVE)
            .contains(&cave)
            .then(|| 3 * usize::from(cave - FIRST_CAVE_WITH_ITEMS))
    }
}

impl std::fmt::Debug for Tables<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Tables")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms_and_caves() {
        let mut data = vec![0u8; 0x400];
        data[A + 0x22] = (3 << 5) | (2 << 2);
        data[B + 0x22] = (1 << 5) | (4 << 2);
        data[D + 0x7f] = ITEM_CELLAR;
        data[A + 0x7f] = 0x22;
        data[B + 0x7f] = 0x22;
        data[E + 0x7f] = 10 | 0x40;
        data[B + 0x77] = 17 << 2;
        data[CAVE_ITEMS + 3..CAVE_ITEMS + 6].copy_from_slice(&[0x40 | 26, 1, 0x3f]);
        let t = Tables::new(&data).unwrap();
        assert_eq!(t.doors(0x22), [3, 2, 1, 4]);
        assert!(t.is_cellar(0x7f) && !t.is_cellar(0x22));
        assert_eq!((t.stairs(0x7f), t.item(0x7f)), ([0x22, 0x22], 10));
        assert_eq!(t.cave(0x77), 17);
        assert_eq!(t.cave_items(17), Some([26, 1, NO_CAVE_ITEM]));
        assert_eq!(t.cave_items(15), None);
        assert!(Tables::new(&data[..LEN - 1]).is_none());
    }
}
