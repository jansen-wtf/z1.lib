//! The original's flag string: one big number in base 63 that packs the
//! state of 27 drop-down lists and 91 three-state check boxes.

use std::fmt;

const ALPHABET: &[u8; 63] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!";

/// State of a check box. `Random` is the indeterminate state, resolved
/// per seed when a game is generated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Check {
    #[default]
    Off,
    On,
    Random,
}

impl Check {
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
    pub const QUEST_NUMBER: usize = 0;
    pub const DUNGEON_QUEST: usize = 1;
    pub const HINT_TYPE: usize = 2;
    pub const START_SCREEN: usize = 3;
    pub const CAVES_TO_SHUFFLE: usize = 4;
    pub const SHUFFLE_ITEMS: usize = 5;
    pub const WOOD_SWORD_STATE: usize = 6;
    pub const TRIFORCES_REQUIRED: usize = 7;
    pub const ENEMY_HP: usize = 8;
    pub const BOSS_HP: usize = 9;
    pub const ARMOS_ITEM: usize = 10;
    pub const COAST_ITEM: usize = 11;
    pub const WHITE_SWORD_ITEM: usize = 12;
    pub const DUNGEON_ROOM_SHUFFLE: usize = 13;
    pub const ITEM_SPRITES: usize = 14;
    pub const START_HEARTS: usize = 15;
    pub const MAX_START_ITEMS: usize = 16;
    pub const START_TRIFORCES: usize = 17;
    pub const RED_BUBBLE: usize = 18;
    pub const WS_MIN: usize = 19;
    pub const WS_MAX: usize = 20;
    pub const MS_MIN: usize = 21;
    pub const MS_MAX: usize = 22;
    pub const TRIFORCE_RANGE_LOW: usize = 23;
    pub const TRIFORCE_RANGE_HIGH: usize = 24;
    pub const ITEM_IN_9_1: usize = 25;
    pub const ITEM_IN_9_2: usize = 26;

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
    pub const UPDATE_WHISTLE: usize = 0;
    pub const SHUFFLE_WOOD_SWORD: usize = 1;
    pub const WOOD_BLOCKS: usize = 2;
    pub const SHUFFLE_BRACELET_CAVES: usize = 3;
    pub const CHANGE_SECRET_SPOTS: usize = 4;
    pub const MIRROR_OW: usize = 5;
    pub const REVERSE_WHISTLE: usize = 6;
    pub const SHUFFLE_SHOP_ITEMS: usize = 7;
    pub const ADD_EXTRA_CANDLES: usize = 8;
    pub const CHANGE_SWORD_HEARTS: usize = 9;
    pub const SHUFFLE_MMG: usize = 10;
    pub const BOMB_UPGRADES: usize = 11;
    pub const SHUFFLE_ARMOS: usize = 12;
    pub const SHUFFLE_DUNGEON_ITEMS: usize = 13;
    pub const SHUFFLE_DUNGEON_DROPS: usize = 14;
    pub const DUNGEON_HEARTS: usize = 15;
    pub const MINOR_DUNGEON_DROPS: usize = 16;
    pub const FORCE_GANNON: usize = 17;
    pub const SHUFFLE_DUNGEON_TEXT: usize = 18;
    pub const ADD_2ND_QUEST_ROOMS: usize = 19;
    pub const ADD_2ND_QUEST_DOORS: usize = 20;
    pub const ADD_MONEY_OR_LIFE: usize = 21;
    pub const CHANGE_LIFE_OR_MONEY: usize = 22;
    pub const HIDE_DUNGEON_NUMBERS: usize = 23;
    pub const ANY_ITEM_IN_9: usize = 24;
    pub const START_ROOM_SWAP: usize = 25;
    pub const DUNGEON_PALETTE: usize = 26;
    pub const SHUFFLE_GORIYA: usize = 27;
    pub const SHUFFLE_BOMB_MEN: usize = 28;
    pub const REMOVE_OPEN_STAIRS: usize = 29;
    pub const SHUFFLE_OVERWORLD: usize = 30;
    pub const SHUFFLE_DUNGEON_MONSTERS: usize = 31;
    pub const SHUFFLE_GANON_ZELDA: usize = 32;
    pub const SHUFFLE_BOSSES: usize = 33;
    pub const INCLUDE_LEVEL_9: usize = 34;
    pub const SHUFFLE_MONSTERS_LEVELS: usize = 35;
    pub const ADD_2ND_QUEST_MONSTERS: usize = 36;
    pub const ENEMY_DROP_GROUPS: usize = 37;
    pub const ITEM_MADNESS: usize = 38;
    pub const SHUFFLE_BOSS_GROUPS: usize = 39;
    pub const GANNON_HP_TO_0: usize = 40;
    pub const SHUFFLE_ENEMY_GROUPS: usize = 41;
    pub const SHUFFLE_OVERWORLD_ENEMIES: usize = 42;
    pub const ITEM_DROP_RATE: usize = 43;
    pub const MAX_ENEMY_HEALTH: usize = 44;
    pub const MAX_BOSS_HEALTH: usize = 45;
    pub const BOOK_BOMB: usize = 46;
    pub const TRANSLATE_OLD_MEN: usize = 47;
    pub const BOOK_ATLAS: usize = 48;
    pub const BLACKOUT: usize = 49;
    pub const PRINT_DEBUG: usize = 50;
    pub const PERMA_BEAM: usize = 51;
    pub const OHKO: usize = 52;
    pub const SPEED_UP_TEXT: usize = 53;
    pub const HIDE_HEALTH: usize = 54;
    pub const KILL_ITEMS: usize = 55;
    pub const START_RAFT: usize = 56;
    pub const START_BOW: usize = 57;
    pub const START_BOMBS: usize = 58;
    pub const START_RECORDER: usize = 59;
    pub const START_BAIT: usize = 60;
    pub const START_ANY_KEY: usize = 61;
    pub const START_LETTER: usize = 62;
    pub const START_PB: usize = 63;
    pub const START_LADDER: usize = 64;
    pub const START_WAND: usize = 65;
    pub const START_BOOK: usize = 66;
    pub const START_WOOD_SWORD: usize = 67;
    pub const START_WHITE_SWORD: usize = 68;
    pub const START_MAGICAL_SWORD: usize = 69;
    pub const START_WOOD_ARROW: usize = 70;
    pub const START_SILVER_ARROW: usize = 71;
    pub const START_BLUE_CANDLE: usize = 72;
    pub const START_RED_CANDLE: usize = 73;
    pub const START_BLUE_RING: usize = 74;
    pub const START_RED_RING: usize = 75;
    pub const START_WOOD_BOOMERANG: usize = 76;
    pub const START_MAGIC_BOOMERANG: usize = 77;
    pub const ADD_EXTRA_BOSSES: usize = 78;
    pub const RUPEE_SPEED: usize = 79;
    pub const OVERWORLD_ITEM_BLOCK: usize = 80;
    pub const TOURNEY_MODE: usize = 81;
    pub const HIDE_NUMBERS: usize = 82;
    pub const PREVENT_CLIPS: usize = 83;
    pub const START_MAGIC_SHIELD: usize = 84;
    pub const NO_SORT_SHAPES: usize = 85;
    pub const ADD_BRIDGES: usize = 86;
    pub const LOST_WOODS_SHORTCUT: usize = 87;
    pub const OW_VISIBLE_TILES: usize = 88;
    pub const BOMB_CAPACITY_UP: usize = 89;
    pub const ALT_DROPS: usize = 90;

    pub const COUNT: usize = 91;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flags {
    pub combos: [u32; combo::COUNT],
    pub checks: [Check; check::COUNT],
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
    /// Decodes a flag string the way the original's text box does: the
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

    pub fn combo(&self, index: usize) -> u32 {
        self.combos[index]
    }

    pub fn check(&self, index: usize) -> Check {
        self.checks[index]
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
        flags.combos[combo::QUEST_NUMBER] = 3;
        assert_eq!(flags.to_string(), "3");
        flags.checks[check::UPDATE_WHISTLE] = Check::On;
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
