//! Item ids, as the game numbers its items in caves, rooms and sprites.

/// Bombs.
pub const BOMBS: u8 = 0;
/// The wood sword.
pub const WOOD_SWORD: u8 = 1;
/// The white sword.
pub const WHITE_SWORD: u8 = 2;
/// The magical sword.
pub const MAGICAL_SWORD: u8 = 3;
/// Bait (meat).
pub const BAIT: u8 = 4;
/// The recorder.
pub const RECORDER: u8 = 5;
/// The blue candle.
pub const BLUE_CANDLE: u8 = 6;
/// The red candle.
pub const RED_CANDLE: u8 = 7;
/// The wood arrow.
pub const WOOD_ARROW: u8 = 8;
/// The silver arrow.
pub const SILVER_ARROW: u8 = 9;
/// The bow.
pub const BOW: u8 = 10;
/// The magical key.
pub const MAGICAL_KEY: u8 = 11;
/// The raft.
pub const RAFT: u8 = 12;
/// The ladder.
pub const LADDER: u8 = 13;
/// The triforce of power, in Gannon's room.
pub const TRIFORCE_OF_POWER: u8 = 14;
/// Five rupees.
pub const FIVE_RUPEES: u8 = 15;
/// The wand.
pub const WAND: u8 = 16;
/// The book of magic.
pub const BOOK: u8 = 17;
/// The blue ring.
pub const BLUE_RING: u8 = 18;
/// The red ring.
pub const RED_RING: u8 = 19;
/// The power bracelet.
pub const POWER_BRACELET: u8 = 20;
/// The letter for the old woman.
pub const LETTER: u8 = 21;
/// A level's compass.
pub const COMPASS: u8 = 22;
/// A level's map.
pub const MAP: u8 = 23;
/// One rupee.
pub const RUPEE: u8 = 24;
/// A key.
pub const KEY: u8 = 25;
/// A heart container.
pub const HEART_CONTAINER: u8 = 26;
/// A piece of the triforce.
pub const TRIFORCE: u8 = 27;
/// The magical shield.
pub const MAGICAL_SHIELD: u8 = 28;
/// The boomerang.
pub const BOOMERANG: u8 = 29;
/// The magical boomerang.
pub const MAGICAL_BOOMERANG: u8 = 30;
/// The blue potion.
pub const BLUE_POTION: u8 = 31;
/// The red potion.
pub const RED_POTION: u8 = 32;
/// The clock.
pub const CLOCK: u8 = 33;
/// A heart.
pub const HEART: u8 = 34;
/// A fairy.
pub const FAIRY: u8 = 35;
/// Ids past this are no item.
pub const LAST: u8 = FAIRY;

const NAMES: [&str; 36] = [
    "Bombs",
    "Wood Sword",
    "White Sword",
    "Magical Sword",
    "Bait",
    "Recorder",
    "Blue Candle",
    "Red Candle",
    "Wood Arrow",
    "Silver Arrow",
    "Bow",
    "Magical Key",
    "Raft",
    "Ladder",
    "Triforce of Power",
    "5 Rupees",
    "Wand",
    "Book",
    "Blue Ring",
    "Red Ring",
    "Power Bracelet",
    "Letter",
    "Compass",
    "Map",
    "Rupee",
    "Key",
    "Heart Container",
    "Triforce",
    "Magical Shield",
    "Boomerang",
    "Magical Boomerang",
    "Blue Potion",
    "Red Potion",
    "Clock",
    "Heart",
    "Fairy",
];

/// The item's name in English, as the game's manual calls it. The Zelda
/// Randomizer's own list spells some apart ("Wooden Sword", "Magic Key",
/// "Weird Moblin" for 14), and its log and hash use that list. `None` for
/// ids past `LAST`.
pub fn name(item: u8) -> Option<&'static str> {
    NAMES.get(usize::from(item)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_has_its_name() {
        assert_eq!(name(BOMBS), Some("Bombs"));
        assert_eq!(name(POWER_BRACELET), Some("Power Bracelet"));
        assert_eq!(name(LAST), Some("Fairy"));
        assert_eq!(name(LAST + 1), None);
        assert_eq!(name(0x3f), None);
    }
}
