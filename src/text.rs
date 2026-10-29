//! The game's text: its font as tiles and characters, and the table of
//! texts the people in caves and the old men in levels say.
//!
//! A text is a run of font tiles, one byte each: the tile in bits 0-5, bit 7
//! ending the first line, bit 6 the second, both together the text.

use crate::rom::Bytes;

/// The text table: 64 pointers of two bytes, little-endian, into bank 1.
pub const POINTERS: usize = 0x4010;
/// The table holds this many texts.
pub const COUNT: u8 = 64;
/// A pointer minus this is the text's file offset.
pub const BANK_BASE: usize = 0x3ff0;
/// Per cave from 16 on, the text its person says: twice the text's number
/// in the low six bits.
pub const CAVE_TEXTS: usize = 0x45b2;
/// What the pay-to-talk caves say for each of their three prices, twice the
/// text's number each: cave 27 the first three, cave 28 the next three.
pub const PAY_TEXTS: usize = 0x494b;
/// Per level 0-9, two bytes; the first is the low byte of the code that
/// picks the old men's texts, telling which of three tables they use.
pub const LEVEL_TEXT_CODE: usize = 0x4a17;
/// The longest text the game's table holds, in bytes; no text is read
/// further than this.
pub const MAX_LEN: usize = 64;

/// The character the font's `tile` shows: digits, letters, a blank and the
/// signs `,!'&."?-`. `None` for any other tile.
pub fn char_of_tile(tile: u8) -> Option<char> {
    Some(match tile {
        0..=9 => char::from(b'0' + tile),
        10..=35 => char::from(b'A' + tile - 10),
        36 | 37 => ' ',
        40 => ',',
        41 => '!',
        42 => '\'',
        43 => '&',
        44 => '.',
        45 => '"',
        46 => '?',
        47 => '-',
        _ => return None,
    })
}

/// The font's tile for `c`, letters in either case; a blank is tile 36.
/// `None` for characters the font lacks.
pub fn tile_of_char(c: char) -> Option<u8> {
    let c = c.to_ascii_uppercase();
    Some(match c {
        '0'..='9' => c as u8 - b'0',
        'A'..='Z' => c as u8 - b'A' + 10,
        ' ' => 36,
        ',' => 40,
        '!' => 41,
        '\'' => 42,
        '&' => 43,
        '.' => 44,
        '"' => 45,
        '?' => 46,
        '-' => 47,
        _ => return None,
    })
}

/// A text's bytes as lines: bit 7 or 6 ends a line, both end the text.
/// Tiles the font lacks show as blanks; each line is trimmed.
pub fn decode(bytes: &[u8]) -> Vec<String> {
    let mut lines = vec![String::new()];
    for &b in bytes {
        lines
            .last_mut()
            .expect("one line at least")
            .push(char_of_tile(b & 0x3f).unwrap_or(' '));
        if b & 0xc0 == 0xc0 {
            break;
        }
        if b & 0xc0 != 0 {
            lines.push(String::new());
        }
    }
    let mut lines: Vec<String> = lines.iter().map(|l| l.trim().to_string()).collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// The bytes of text `id` (0-63) of the game's table, up to and including
/// the one that ends it, at most `MAX_LEN`. `None` for an id past the table
/// or a pointer that leads outside the ROM.
pub fn text_bytes(rom: Bytes<'_>, id: u8) -> Option<Vec<u8>> {
    if id >= COUNT {
        return None;
    }
    let at = POINTERS + 2 * usize::from(id);
    let pointer = usize::from(rom[at]) | usize::from(rom[at + 1]) << 8;
    let start = pointer.checked_sub(BANK_BASE)?;
    let mut out = Vec::new();
    for &b in rom.get(start..)?.iter().take(MAX_LEN) {
        out.push(b);
        if b & 0xc0 == 0xc0 {
            break;
        }
    }
    Some(out)
}

/// Text `id` of the game's table as lines; see `text_bytes`.
pub fn text(rom: Bytes<'_>, id: u8) -> Option<Vec<String>> {
    text_bytes(rom, id).map(|b| decode(&b))
}

/// The number of the text the person in `cave` (16-35) says. `None` for
/// other caves. The pay-to-talk caves 27 and 28 say one of three texts
/// after paying instead (`pay_texts`).
pub fn cave_text(rom: Bytes<'_>, cave: u8) -> Option<u8> {
    (16..=35)
        .contains(&cave)
        .then(|| (rom[CAVE_TEXTS + usize::from(cave - 16)] & 0x3f) / 2)
}

/// The three texts a pay-to-talk cave (27 or 28) says, one per price.
pub fn pay_texts(rom: Bytes<'_>, cave: u8) -> Option<[u8; 3]> {
    let first = match cave {
        27 => 0,
        28 => 3,
        _ => return None,
    };
    Some(std::array::from_fn(|k| rom[PAY_TEXTS + first + k] / 2))
}

/// The number of the text an old man says in `level` (0-9), in a room of
/// kind `kind` (11-18, the room's table C value in bits 0-5). `None` for
/// other kinds or levels, or a level whose code the game does not use.
pub fn old_man_text(rom: Bytes<'_>, level: u8, kind: u8) -> Option<u8> {
    if level > 9 || !(11..=18).contains(&kind) {
        return None;
    }
    let table = match rom[LEVEL_TEXT_CODE + 2 * usize::from(level)] {
        0x23 => 0x4a2b,
        0x69 => 0x4a71,
        0x84 => 0x4a90,
        _ => return None,
    };
    Some(rom[table + usize::from(kind - 11)] / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> Vec<u8> {
        let mut data = vec![0; crate::rom::FILE_LEN];
        data[..4].copy_from_slice(b"NES\x1a");
        data
    }

    #[test]
    fn tiles_and_characters_go_both_ways() {
        for tile in (0..=47).filter(|&t| !(37..=39).contains(&t)) {
            if let Some(c) = char_of_tile(tile) {
                assert_eq!(tile_of_char(c), Some(tile), "{c}");
            }
        }
        assert_eq!(tile_of_char('a'), Some(10));
        assert_eq!(char_of_tile(37), Some(' '));
        assert_eq!(tile_of_char('#'), None);
    }

    #[test]
    fn lines_end_at_bits_7_and_6() {
        let mut t: Vec<u8> = "HI".chars().map(|c| tile_of_char(c).unwrap()).collect();
        t[1] |= 0x80;
        t.extend([tile_of_char('A').unwrap() | 0xc0, 0x0b]);
        assert_eq!(decode(&t), ["HI", "A"]);
    }

    #[test]
    fn any_checked_bytes_read_no_further_than_their_end() {
        let mut data = blank();
        // A pointer to the very last byte, which never ends the text.
        data[POINTERS..POINTERS + 2].copy_from_slice(&0xffffu16.to_le_bytes());
        let rom = Bytes::new(&data).unwrap();
        for id in 0..=255 {
            text_bytes(rom, id);
        }
        for cave in 0..=255 {
            cave_text(rom, cave);
            pay_texts(rom, cave);
        }
        for level in 0..=255 {
            for kind in 0..=255 {
                old_man_text(rom, level, kind);
            }
        }
    }

    #[test]
    #[ignore = "needs the original ROM in Z1R_ROM"]
    fn the_original_says_what_players_know() {
        let data = std::fs::read(std::env::var_os("Z1R_ROM").expect("Z1R_ROM"))
            .expect("the ROM in Z1R_ROM");
        let rom = Bytes::new(&data).unwrap();
        // The wood sword cave.
        let sword = text(rom, cave_text(rom, 16).unwrap()).unwrap();
        assert_eq!(sword, ["IT'S DANGEROUS TO GO", "ALONE! TAKE THIS."]);
        // Level 1's old man in the room with the eagle's hint.
        let all: Vec<String> = (11..=18)
            .filter_map(|k| text(rom, old_man_text(rom, 1, k)?))
            .map(|l| l.join(" "))
            .collect();
        assert!(
            all.iter().any(|t| t.contains("EASTMOST PENNINSULA")),
            "{all:?}"
        );
    }
}
