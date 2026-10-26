//! The Zelda Randomizer's flag string: one big number in base 63 that packs the
//! state of 27 drop-down lists and 94 three-state check boxes.

use std::fmt;

const ALPHABET: &[u8; 63] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!";

/// State of a check box. `Random` is the indeterminate state, resolved
/// per seed when a game is generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Check {
    /// Not ticked.
    #[default]
    Off,
    /// Ticked.
    On,
    /// Neither: decided per seed.
    Random,
}

impl Check {
    /// The state a digit of the flag number stands for, 0-2; anything else
    /// is off.
    pub fn from_digit(digit: u32) -> Check {
        match digit {
            1 => Check::On,
            2 => Check::Random,
            _ => Check::Off,
        }
    }

    fn digit(self) -> u32 {
        match self {
            Check::Off => 0,
            Check::On => 1,
            Check::Random => 2,
        }
    }
}

/// Drop-down lists in flag string order.
pub mod combo {
    /// "Overworld Quest" on the Main tab: 0 1st-quest, 1 2nd-quest, 2
    /// mixed-quest-1st, 3 mixed-quest-2nd, 4 random.
    pub const QUEST_NUMBER: usize = 0;
    /// "Dungeon Quest" on the Main tab: 0 1st-quest, 1 2nd-quest, 2 mixed-
    /// quest, 3 shapes, 4 mixed-shapes, 5 random-no-shapes, 6 random.
    pub const DUNGEON_QUEST: usize = 1;
    /// "Hint Type" on the Main tab: 0 normal, 1 helpful, 2 community, 3
    /// deception, 4 mixed, 5 blank, 6 random.
    pub const HINT_TYPE: usize = 2;
    /// "Start Screen" on the Overworld tab: 0 normal, 1 easy-start-shuffle,
    /// 2 full-start-shuffle, 3 wood-sword-screen.
    pub const START_SCREEN: usize = 3;
    /// "Cave Shuffle" on the Overworld tab: 0 vanilla, 1 dungeons-only, 2
    /// non-dungeons-only, 3 all-caves, 4 random.
    pub const CAVES_TO_SHUFFLE: usize = 4;
    /// "Item Shuffle" on the Dungeon Items tab: 0 none, 1 item-only, 2
    /// intra-dungeon, 3 full, 4 random.
    pub const SHUFFLE_ITEMS: usize = 5;
    /// "Swords" on the Advanced tab: 0 normal, 1 wood-sword-only, 2 no-
    /// wood-sword, 3 swordless, 4 random.
    pub const WOOD_SWORD_STATE: usize = 6;
    /// "Level 9 Entry" on the Dungeons tab: 0 8, 1 7, 2 6, 3 5, 4 4, 5 3, 6
    /// 2, 7 1, 8 open, 9 triforce-range, 10 specific-triforces, 11 other-
    /// item, 12 random.
    pub const TRIFORCES_REQUIRED: usize = 7;
    /// "Enemy HP" on the Monsters tab: 0 normal, 1 plus-minus-2, 2 plus-
    /// minus-4, 3 0-hp, 4 random.
    pub const ENEMY_HP: usize = 8;
    /// "Boss HP" on the Monsters tab: 0 normal, 1 plus-minus-2, 2 plus-
    /// minus-4, 3 0-hp, 4 random.
    pub const BOSS_HP: usize = 9;
    /// "Armos Item" on the Dungeon Items tab: 0 random, 1 book, 2
    /// boomerang, 3 bow, 4 heart-container, 5 ladder, 6 magical-boomerang,
    /// 7 magical-key, 8 power-bracelet, 9 raft, 10 recorder, 11 red-candle,
    /// 12 red-ring, 13 silver-arrow, 14 wand, 15 white-sword, 16 non-heart.
    pub const ARMOS_ITEM: usize = 10;
    /// "Coast Item" on the Dungeon Items tab: 0 random, 1 book, 2
    /// boomerang, 3 bow, 4 heart-container, 5 ladder, 6 magical-boomerang,
    /// 7 magical-key, 8 power-bracelet, 9 raft, 10 recorder, 11 red-candle,
    /// 12 red-ring, 13 silver-arrow, 14 wand, 15 white-sword, 16 non-heart.
    pub const COAST_ITEM: usize = 11;
    /// "White Sword Item" on the Dungeon Items tab: 0 random, 1 book, 2
    /// boomerang, 3 bow, 4 heart-container, 5 ladder, 6 magical-boomerang,
    /// 7 magical-key, 8 power-bracelet, 9 raft, 10 recorder, 11 red-candle,
    /// 12 red-ring, 13 silver-arrow, 14 wand, 15 white-sword, 16 non-heart.
    pub const WHITE_SWORD_ITEM: usize = 12;
    /// "Dungeon Room Shuffle" on the Dungeons tab: 0 vanilla, 1 in-dungeon-
    /// shuffle, 2 full-shuffle, 3 random.
    pub const DUNGEON_ROOM_SHUFFLE: usize = 13;
    /// "Item Sprites" on the Misc tab: 0 normal, 1 fun-percent, 2 sprite-
    /// shuffle, 3 random.
    pub const ITEM_SPRITES: usize = 14;
    /// "Starting Hearts" on the Start Items tab: 0 1, 1 2, 2 3, 3 4, 4 5, 5
    /// 6, 6 7, 7 8, 8 9, 9 10, 10 11, 11 12, 12 13, 13 14, 14 15, 15 16, 16
    /// 1-5.
    pub const START_HEARTS: usize = 15;
    /// "Max Start Items" on the Start Items tab: 0 all, 1 0, 2 1, 3 2, 4 3,
    /// 5 4, 6 5, 7 6, 8 7, 9 8, 10 9, 11 10, 12 11, 13 12, 14 13, 15 14, 16
    /// 15, 17 16, 18 17, 19 18, 20 19, 21 20, 22 random.
    pub const MAX_START_ITEMS: usize = 16;
    /// "Triforce Pieces" on the Start Items tab: 0 0, 1 1, 2 2, 3 3, 4 4, 5
    /// 5, 6 6, 7 7, 8 8, 9 random.
    pub const START_TRIFORCES: usize = 17;
    /// "Red Bubble Curse" on the Monsters tab: 0 swordless, 1 inverted-
    /// control, 2 slow-speed, 3 random.
    pub const RED_BUBBLE: usize = 18;
    /// "White Sword Hearts (min)" on the Shops tab: 0 4, 1 5, 2 6.
    pub const WS_MIN: usize = 19;
    /// "White Sword Hearts (max)" on the Shops tab: 0 6, 1 5, 2 4.
    pub const WS_MAX: usize = 20;
    /// "Magical Sword Hearts (min)" on the Shops tab: 0 10, 1 11, 2 12, 3
    /// 13, 4 14.
    pub const MS_MIN: usize = 21;
    /// "Magical Sword Hearts (max)" on the Shops tab: 0 14, 1 13, 2 12, 3
    /// 11, 4 10.
    pub const MS_MAX: usize = 22;
    /// "Triforce Range (low)" on the Dungeons tab: 0 0, 1 1, 2 2, 3 3, 4 4,
    /// 5 5, 6 6, 7 7, 8 8.
    pub const TRIFORCE_RANGE_LOW: usize = 23;
    /// "Triforce Range (high)" on the Dungeons tab: 0 8, 1 7, 2 6, 3 5, 4
    /// 4, 5 3, 6 2, 7 1, 8 0.
    pub const TRIFORCE_RANGE_HIGH: usize = 24;
    /// "Items in 9 (first)" on the Dungeon Items tab: 0 random, 1 book, 2
    /// boomerang, 3 bow, 4 heart-container, 5 ladder, 6 magical-boomerang,
    /// 7 magical-key, 8 power-bracelet, 9 raft, 10 recorder, 11 red-candle,
    /// 12 red-ring, 13 silver-arrow, 14 wand, 15 white-sword, 16 non-heart.
    pub const ITEM_IN_9_1: usize = 25;
    /// "Items in 9 (second)" on the Dungeon Items tab: 0 random, 1 book, 2
    /// boomerang, 3 bow, 4 heart-container, 5 ladder, 6 magical-boomerang,
    /// 7 magical-key, 8 power-bracelet, 9 raft, 10 recorder, 11 red-candle,
    /// 12 red-ring, 13 silver-arrow, 14 wand, 15 white-sword, 16 non-heart.
    pub const ITEM_IN_9_2: usize = 26;

    /// The number of lists.
    pub const COUNT: usize = 27;
    /// Radix of each list in the flag number.
    pub const RADIX: [u32; COUNT] = [
        8, 10, 10, 7, 8, 8, 8, 15, 8, 8, 20, 20, 20, 7, 7, 20, 26, 13, 7, 6, 6, 8, 8, 12, 12, 20,
        20,
    ];
    /// Number of entries in each list; larger encoded values are clamped.
    pub const ENTRIES: [u32; COUNT] = [
        5, 7, 7, 4, 5, 5, 5, 13, 5, 5, 17, 17, 17, 4, 4, 17, 23, 10, 4, 3, 3, 5, 5, 9, 9, 17, 17,
    ];
}

/// Check boxes in flag string order.
pub mod check {
    /// "Recorder To New Dungeons" on the Overworld tab.
    pub const UPDATE_WHISTLE: usize = 0;
    /// "Shuffle Wood Sword Cave" on the Overworld tab.
    pub const SHUFFLE_WOOD_SWORD: usize = 1;
    /// "Allow OW Blocks of Wood Sword" on the Overworld tab.
    pub const WOOD_BLOCKS: usize = 2;
    /// "Shuffle "Take Any Road" Caves" on the Overworld tab.
    pub const SHUFFLE_BRACELET_CAVES: usize = 3;
    /// "Change Secret Spots" on the Advanced tab.
    pub const CHANGE_SECRET_SPOTS: usize = 4;
    /// "Mirror Overworld" on the Overworld tab.
    pub const MIRROR_OW: usize = 5;
    /// "Recorder to Unbeaten Dungeons" on the Overworld tab.
    pub const REVERSE_WHISTLE: usize = 6;
    /// "Shuffle Shop Items" on the Shops tab.
    pub const SHUFFLE_SHOP_ITEMS: usize = 7;
    /// "Extra Candles" on the Shops tab.
    pub const ADD_EXTRA_CANDLES: usize = 8;
    /// "Change Sword Hearts" on the Shops tab.
    pub const CHANGE_SWORD_HEARTS: usize = 9;
    /// "Change MMG" on the Shops tab.
    pub const SHUFFLE_MMG: usize = 10;
    /// "Change Bomb Upgrades" on the Shops tab.
    pub const BOMB_UPGRADES: usize = 11;
    /// "Shuffle Armos" on the Shops tab.
    pub const SHUFFLE_ARMOS: usize = 12;
    /// "Shuffle Dungeon Items" on the Dungeon Items tab.
    pub const SHUFFLE_DUNGEON_ITEMS: usize = 13;
    /// "Shuffle Dungeon Drops" on the Dungeon Items tab.
    pub const SHUFFLE_DUNGEON_DROPS: usize = 14;
    /// "Shuffle Dungeon Hearts" on the Dungeon Items tab.
    pub const DUNGEON_HEARTS: usize = 15;
    /// "Shuffle Minor Dungeon Drops" on the Dungeon Items tab.
    pub const MINOR_DUNGEON_DROPS: usize = 16;
    /// "Force Gannon Fight" on the Dungeons tab.
    pub const FORCE_GANNON: usize = 17;
    /// "Shuffle Dungeon Text" on the Dungeons tab.
    pub const SHUFFLE_DUNGEON_TEXT: usize = 18;
    /// "Add 2nd Quest Rooms" on the Dungeons tab.
    pub const ADD_2ND_QUEST_ROOMS: usize = 19;
    /// "Add 2nd Quest Doors" on the Dungeons tab.
    pub const ADD_2ND_QUEST_DOORS: usize = 20;
    /// "Add Money or Life Rooms" on the Dungeons tab.
    pub const ADD_MONEY_OR_LIFE: usize = 21;
    /// "Change "Leave your..." Rooms" on the Dungeons tab.
    pub const CHANGE_LIFE_OR_MONEY: usize = 22;
    /// "Hide Dungeon Numbers" on the Dungeons tab.
    pub const HIDE_DUNGEON_NUMBERS: usize = 23;
    /// "Allow Important Items in 9" on the Dungeon Items tab.
    pub const ANY_ITEM_IN_9: usize = 24;
    /// "Shuffle Dungeon Start Room" on the Dungeons tab.
    pub const START_ROOM_SWAP: usize = 25;
    /// "Shuffle Dungeon Palettes" on the Dungeons tab.
    pub const DUNGEON_PALETTE: usize = 26;
    /// "Shuffle Hungry Goriya" on the Dungeons tab.
    pub const SHUFFLE_GORIYA: usize = 27;
    /// "Shuffle Bomb Upgrade Men" on the Dungeons tab.
    pub const SHUFFLE_BOMB_MEN: usize = 28;
    /// "Remove Most Open Stairs" on the Advanced tab.
    pub const REMOVE_OPEN_STAIRS: usize = 29;
    /// "Shuffle Overworld Monsters" on the Monsters tab.
    pub const SHUFFLE_OVERWORLD: usize = 30;
    /// "Shuffle Dungeon Monsters" on the Monsters tab.
    pub const SHUFFLE_DUNGEON_MONSTERS: usize = 31;
    /// "Shuffle Gannon and Zelda" on the Monsters tab.
    pub const SHUFFLE_GANON_ZELDA: usize = 32;
    /// "Shuffle Bosses" on the Monsters tab.
    pub const SHUFFLE_BOSSES: usize = 33;
    /// "Shuffle Level 9 Monsters" on the Monsters tab.
    pub const INCLUDE_LEVEL_9: usize = 34;
    /// "Shuffle Monsters Between Levels" on the Monsters tab.
    pub const SHUFFLE_MONSTERS_LEVELS: usize = 35;
    /// "Add 2nd Quest Monsters" on the Monsters tab.
    pub const ADD_2ND_QUEST_MONSTERS: usize = 36;
    /// "Shuffle Enemy Drop Groups" on the Monsters tab.
    pub const ENEMY_DROP_GROUPS: usize = 37;
    /// "Make Important Items Drops" on the Monsters tab.
    pub const ITEM_MADNESS: usize = 38;
    /// "Randomize Boss Groups" on the Monsters tab.
    pub const SHUFFLE_BOSS_GROUPS: usize = 39;
    /// "Set Gannon HP to 0" on the Monsters tab.
    pub const GANNON_HP_TO_0: usize = 40;
    /// "Shuffle Enemy Groups" on the Monsters tab.
    pub const SHUFFLE_ENEMY_GROUPS: usize = 41;
    /// "Shuffle Overworld Group" on the Monsters tab.
    pub const SHUFFLE_OVERWORLD_ENEMIES: usize = 42;
    /// "Randomize Item Drop Rate" on the Monsters tab.
    pub const ITEM_DROP_RATE: usize = 43;
    /// "Maximum Enemy Health" on the Advanced tab.
    pub const MAX_ENEMY_HEALTH: usize = 44;
    /// "Maximum Boss Health" on the Advanced tab.
    pub const MAX_BOSS_HEALTH: usize = 45;
    /// "Replace Book Fire with Explosion" on the Misc tab.
    pub const BOOK_BOMB: usize = 46;
    /// "Book To Understand Old Men" on the Misc tab.
    pub const TRANSLATE_OLD_MEN: usize = 47;
    /// "Book is an Atlas" on the Misc tab.
    pub const BOOK_ATLAS: usize = 48;
    /// "Blackout" on the Advanced tab.
    pub const BLACKOUT: usize = 49;
    /// "Print Quest Info (changes layout)" on the Misc tab.
    pub const PRINT_DEBUG: usize = 50;
    /// "Permanent Sword Beam" on the Misc tab.
    pub const PERMA_BEAM: usize = 51;
    /// "OHKO" on the Advanced tab.
    pub const OHKO: usize = 52;
    /// "Speed Up Text" on the Misc tab.
    pub const SPEED_UP_TEXT: usize = 53;
    /// "Hidden Health" on the Advanced tab.
    pub const HIDE_HEALTH: usize = 54;
    /// "Killable Drop Items" on the Advanced tab.
    pub const KILL_ITEMS: usize = 55;
    /// "Raft" on the Start Items tab.
    pub const START_RAFT: usize = 56;
    /// "Bow" on the Start Items tab.
    pub const START_BOW: usize = 57;
    /// "8 Bombs" on the Start Items tab.
    pub const START_BOMBS: usize = 58;
    /// "Recorder" on the Start Items tab.
    pub const START_RECORDER: usize = 59;
    /// "Bait" on the Start Items tab.
    pub const START_BAIT: usize = 60;
    /// "Magical Key" on the Start Items tab.
    pub const START_ANY_KEY: usize = 61;
    /// "Letter" on the Start Items tab.
    pub const START_LETTER: usize = 62;
    /// "Power Bracelet" on the Start Items tab.
    pub const START_PB: usize = 63;
    /// "Ladder" on the Start Items tab.
    pub const START_LADDER: usize = 64;
    /// "Wand" on the Start Items tab.
    pub const START_WAND: usize = 65;
    /// "Book" on the Start Items tab.
    pub const START_BOOK: usize = 66;
    /// "Wood Sword" on the Start Items tab.
    pub const START_WOOD_SWORD: usize = 67;
    /// "White Sword" on the Start Items tab.
    pub const START_WHITE_SWORD: usize = 68;
    /// "Magical Sword" on the Start Items tab.
    pub const START_MAGICAL_SWORD: usize = 69;
    /// "Wood Arrow" on the Start Items tab.
    pub const START_WOOD_ARROW: usize = 70;
    /// "Silver Arrow" on the Start Items tab.
    pub const START_SILVER_ARROW: usize = 71;
    /// "Blue Candle" on the Start Items tab.
    pub const START_BLUE_CANDLE: usize = 72;
    /// "Red Candle" on the Start Items tab.
    pub const START_RED_CANDLE: usize = 73;
    /// "Blue Ring" on the Start Items tab.
    pub const START_BLUE_RING: usize = 74;
    /// "Red Ring" on the Start Items tab.
    pub const START_RED_RING: usize = 75;
    /// "Wood Boomerang" on the Start Items tab.
    pub const START_WOOD_BOOMERANG: usize = 76;
    /// "Magical Boomerang" on the Start Items tab.
    pub const START_MAGIC_BOOMERANG: usize = 77;
    /// "Add extra bosses" on the Monsters tab.
    pub const ADD_EXTRA_BOSSES: usize = 78;
    /// "Rupees Affect Speed" on the Advanced tab.
    pub const RUPEE_SPEED: usize = 79;
    /// "Force Overworld Block" on the Overworld tab.
    pub const OVERWORLD_ITEM_BLOCK: usize = 80;
    /// "Race ROM" on the Misc tab.
    pub const TOURNEY_MODE: usize = 81;
    /// "Hidden Numbers" on the Advanced tab.
    pub const HIDE_NUMBERS: usize = 82;
    /// "Prevent Clipping" on the Advanced tab.
    pub const PREVENT_CLIPS: usize = 83;
    /// "Magical Shield" on the Start Items tab.
    pub const START_MAGIC_SHIELD: usize = 84;
    /// "Do Not Sort Shapes" on the Dungeons tab.
    pub const NO_SORT_SHAPES: usize = 85;
    /// "Add Bridges to the River" on the Overworld tab.
    pub const ADD_BRIDGES: usize = 86;
    /// "Add Lost Woods Shortcut" on the Overworld tab.
    pub const LOST_WOODS_SHORTCUT: usize = 87;
    /// "Change Hidden OW Tiles" on the Overworld tab.
    pub const OW_VISIBLE_TILES: usize = 88;
    /// "Increase Bomb Capacity Upgrade" on the Shops tab.
    pub const BOMB_CAPACITY_UP: usize = 89;
    /// "Allow Universal Drops in Shapes" on the Dungeons tab.
    pub const ALT_DROPS: usize = 90;
    /// "Remove Maps" on the Advanced tab.
    pub const NO_MAPS: usize = 91;
    /// "All Maps" on the Start Items tab.
    pub const START_MAPS: usize = 92;
    /// "All Compasses" on the Start Items tab.
    pub const START_ALL_COMPASSES: usize = 93;

    /// The number of check boxes.
    pub const COUNT: usize = 94;
}

/// The settings a flag string holds: a value per list, a state per check
/// box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flags {
    combos: [u32; combo::COUNT],
    checks: [Check; check::COUNT],
}

impl Default for Flags {
    fn default() -> Self {
        Flags {
            combos: [0; combo::COUNT],
            checks: [Check::Off; check::COUNT],
        }
    }
}

/// Arbitrary-precision unsigned integer, little-endian in base 2^32;
/// just what the flag number needs.
#[derive(Default)]
struct BigNum(Vec<u32>);

impl BigNum {
    fn mul_add(&mut self, factor: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.0 {
            let v = u64::from(*limb) * u64::from(factor) + carry;
            *limb = v as u32;
            carry = v >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }

    fn div_rem(&mut self, divisor: u32) -> u32 {
        let mut rem = 0u64;
        for limb in self.0.iter_mut().rev() {
            let v = (rem << 32) | u64::from(*limb);
            *limb = (v / u64::from(divisor)) as u32;
            rem = v % u64::from(divisor);
        }
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
        rem as u32
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}

impl Flags {
    /// Decodes a flag string the way the randomizer's text box does: the
    /// first whitespace-separated word counts, unknown characters count as
    /// digit 63, and list values beyond the last entry select the last one.
    pub fn parse(text: &str) -> Flags {
        let word = text.split_whitespace().next().unwrap_or("");
        let mut n = BigNum::default();
        for c in word.bytes() {
            let digit = ALPHABET
                .iter()
                .position(|&a| a == c)
                .unwrap_or(ALPHABET.len());
            n.mul_add(ALPHABET.len() as u32, digit as u32);
        }
        let mut flags = Flags::default();
        for (i, value) in flags.combos.iter_mut().enumerate() {
            *value = n.div_rem(combo::RADIX[i]).min(combo::ENTRIES[i] - 1);
        }
        for state in &mut flags.checks {
            *state = Check::from_digit(n.div_rem(3));
        }
        flags
    }

    /// The flag string as the form shows it after reading `text`: re-encoded
    /// once reading it changed any option, as typed otherwise.
    pub fn displayed(text: &str) -> String {
        let flags = Flags::parse(text);
        if flags == Flags::default() {
            text.split_whitespace().next().unwrap_or("").to_string()
        } else {
            flags.to_string()
        }
    }

    /// The value of list `index`, one of `combo`'s constants.
    ///
    /// # Panics
    /// When `index` is `combo::COUNT` or more.
    pub fn combo(&self, index: usize) -> u32 {
        self.combos[index]
    }

    /// The state of check box `index`, one of `check`'s constants.
    ///
    /// # Panics
    /// When `index` is `check::COUNT` or more.
    pub fn check(&self, index: usize) -> Check {
        self.checks[index]
    }

    /// Sets a list as it is, even past its last entry: the flag string
    /// keeps such a value, and the original reads it as the last entry.
    ///
    /// # Panics
    /// When `index` is `combo::COUNT` or more.
    pub fn set_combo(&mut self, index: usize, value: u32) {
        self.combos[index] = value;
    }

    /// Sets the state of check box `index`.
    ///
    /// # Panics
    /// When `index` is `check::COUNT` or more.
    pub fn set_check(&mut self, index: usize, state: Check) {
        self.checks[index] = state;
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n = BigNum::default();
        for state in self.checks.iter().rev() {
            n.mul_add(3, state.digit());
        }
        for (i, value) in self.combos.iter().enumerate().rev() {
            n.mul_add(combo::RADIX[i], *value);
        }
        let mut digits = Vec::new();
        loop {
            digits.push(ALPHABET[n.div_rem(ALPHABET.len() as u32) as usize]);
            if n.is_zero() {
                break;
            }
        }
        digits.reverse();
        f.write_str(std::str::from_utf8(&digits).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_everything_off() {
        let flags = Flags::parse("0");
        assert_eq!(flags, Flags::default());
        assert_eq!(flags.to_string(), "0");
    }

    #[test]
    fn real_flag_strings_round_trip() {
        for text in [
            "1zuh3gHaClLJcHHNapqQB1pT23b50Y1NhXkgO863",
            "1zuiw1gkUcxhf4DWAPnlYjsnr!5JrnSX6WH6O865",
            "oIbnPfPb01Hll3D295IGDxxjR4UwfEok8P4MD",
            "SQ8gqsY5mwtBby7Zo3Mu5TQQ1XhJWDIOUGvInVj",
            "1yt195laz1xduprsClC3iamRy5jTxnNbVHlyO863",
        ] {
            assert_eq!(Flags::parse(text).to_string(), text);
        }
    }

    #[test]
    fn first_list_is_least_significant() {
        let mut flags = Flags::default();
        flags.set_combo(combo::QUEST_NUMBER, 3);
        assert_eq!(flags.to_string(), "3");
        flags.set_check(check::UPDATE_WHISTLE, Check::On);
        assert_eq!(Flags::parse(&flags.to_string()), flags);
    }

    #[test]
    fn displayed_text_is_reencoded_unless_all_off() {
        assert_eq!(Flags::displayed("00"), "00");
        assert_eq!(Flags::displayed(" 07 x"), "4");
    }

    #[test]
    fn values_beyond_a_list_are_clamped() {
        // 7 in the first list, which has five entries.
        assert_eq!(Flags::parse("7").combo(combo::QUEST_NUMBER), 4);
    }
}
