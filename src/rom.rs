//! iNES image of The Legend of Zelda (USA), first release (PRG0) or Rev 1.
//!
//! All offsets in this crate are file offsets including the 16-byte iNES
//! header, which is how the ROM layout is commonly documented.

use std::fmt;
use std::path::Path;

pub const HEADER_LEN: usize = 16;
pub const BODY_LEN: usize = 0x20000;
pub const FILE_LEN: usize = HEADER_LEN + BODY_LEN;

/// SHA-1 of the 128 KiB PRG body of Zelda (USA), header excluded; the
/// No-Intro DAT lists the same values for its headerless images.
pub const PRG0_BODY_SHA1: &str = "a12d74c73a0481599a5d832361d168f4737bbcf6";
pub const REV1_BODY_SHA1: &str = "be2f5dc8c5ba8ec1a344a71f9fb204750af24fe7";

/// The two US releases the Zelda Randomizer accepts. Its fixed offsets were
/// made for PRG0: on Rev 1 the seed and version lines of its title screen
/// land inside the logo's frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Revision {
    Prg0,
    Rev1,
}

impl Revision {
    pub const ALL: [Revision; 2] = [Revision::Prg0, Revision::Rev1];

    pub fn id(self) -> &'static str {
        match self {
            Revision::Prg0 => "prg0",
            Revision::Rev1 => "rev1",
        }
    }

    /// As No-Intro lists the headerless image.
    pub fn body_sha1(self) -> &'static str {
        match self {
            Revision::Prg0 => PRG0_BODY_SHA1,
            Revision::Rev1 => REV1_BODY_SHA1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Revision::Prg0 => "Legend of Zelda, The (USA)",
            Revision::Rev1 => "Legend of Zelda, The (USA) (Rev 1)",
        }
    }
}

#[derive(Debug)]
pub enum RomError {
    Io(std::io::Error),
    Size(usize),
    Header,
    Checksum(String),
}

impl fmt::Display for RomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomError::Io(e) => write!(f, "cannot read ROM: {e}"),
            RomError::Size(n) => write!(f, "ROM has {n} bytes, expected {FILE_LEN}"),
            RomError::Header => write!(f, "ROM has no iNES header"),
            RomError::Checksum(sum) => write!(
                f,
                "ROM body SHA-1 is {sum}, expected {PRG0_BODY_SHA1} ({}) or {REV1_BODY_SHA1} ({})",
                Revision::Prg0.name(),
                Revision::Rev1.name()
            ),
        }
    }
}

impl std::error::Error for RomError {}

impl From<std::io::Error> for RomError {
    fn from(e: std::io::Error) -> Self {
        RomError::Io(e)
    }
}

#[derive(Clone)]
pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn load(path: &Path) -> Result<Rom, RomError> {
        Rom::from_bytes(std::fs::read(path)?)
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Rom, RomError> {
        if data.len() != FILE_LEN {
            return Err(RomError::Size(data.len()));
        }
        if &data[..4] != b"NES\x1a" {
            return Err(RomError::Header);
        }
        let sum = sha1_smol::Sha1::from(&data[HEADER_LEN..])
            .digest()
            .to_string();
        if sum != PRG0_BODY_SHA1 && sum != REV1_BODY_SHA1 {
            return Err(RomError::Checksum(sum));
        }
        Ok(Rom { data })
    }

    pub fn revision(&self) -> Revision {
        let sum = sha1_smol::Sha1::from(&self.data[HEADER_LEN..])
            .digest()
            .to_string();
        if sum == PRG0_BODY_SHA1 {
            Revision::Prg0
        } else {
            Revision::Rev1
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn get(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    pub fn set(&mut self, offset: usize, value: u8) {
        self.data[offset] = value;
    }

    pub fn write(&mut self, offset: usize, values: &[u8]) {
        self.data[offset..offset + values.len()].copy_from_slice(values);
    }
}

/// Where the game keeps the six characters it shows before a level's
/// digit: "LEVEL-" unless the level-string setting picked another word.
pub const LEVEL_NAME: usize = 0x19d17;

/// The word the game shows before a level's digit, as it shows it:
/// "LEVEL-", "BOARD-", "SECTOR", "LAIR -", ...
pub fn level_name(rom: &[u8]) -> String {
    rom[LEVEL_NAME..LEVEL_NAME + 6]
        .iter()
        .map(|&b| match b {
            0..=9 => char::from(b'0' + b),
            10..=35 => char::from(b'A' + b - 10),
            41 => '!',
            42 => '\'',
            44 => '.',
            46 => '?',
            // The game writes its dash as tile 0x62, the Zelda Randomizer as 47.
            47 | 0x62 => '-',
            _ => ' ',
        })
        .collect()
}

/// The level's name for text: "Level 3", "Board 3", "Sector 3".
pub fn level_title(rom: &[u8], level: i32) -> String {
    let name = level_name(rom);
    let word = name.trim_end_matches([' ', '-']);
    let mut title = word[..1.min(word.len())].to_string();
    title.push_str(&word[1.min(word.len())..].to_lowercase());
    format!("{title} {level}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_size() {
        assert!(matches!(
            Rom::from_bytes(vec![0; 100]),
            Err(RomError::Size(100))
        ));
    }

    #[test]
    fn rejects_missing_header() {
        assert!(matches!(
            Rom::from_bytes(vec![0; FILE_LEN]),
            Err(RomError::Header)
        ));
    }

    #[test]
    fn rejects_other_dumps() {
        let mut data = vec![0; FILE_LEN];
        data[..4].copy_from_slice(b"NES\x1a");
        assert!(matches!(Rom::from_bytes(data), Err(RomError::Checksum(_))));
    }

    #[test]
    fn level_names_as_the_game_shows_them() {
        let mut data = vec![0; FILE_LEN];
        // The game's own "LEVEL-", with its own dash.
        data[LEVEL_NAME..LEVEL_NAME + 6].copy_from_slice(&[21, 14, 31, 14, 21, 0x62]);
        assert_eq!(level_name(&data), "LEVEL-");
        assert_eq!(level_title(&data, 3), "Level 3");
        // "LAIR -" and "SECTOR" as the Zelda Randomizer writes them.
        data[LEVEL_NAME..LEVEL_NAME + 6].copy_from_slice(&[21, 10, 18, 27, 36, 47]);
        assert_eq!(level_name(&data), "LAIR -");
        assert_eq!(level_title(&data, 7), "Lair 7");
        data[LEVEL_NAME..LEVEL_NAME + 6].copy_from_slice(&[28, 14, 12, 29, 24, 27]);
        assert_eq!(level_title(&data, 1), "Sector 1");
    }
}
