//! iNES image of The Legend of Zelda (USA), first release (PRG0) or Rev 1.
//!
//! All offsets in this crate are file offsets including the 16-byte iNES
//! header, which is how the ROM layout is commonly documented.

use std::fmt;
use std::path::Path;

/// The iNES header in front of the game.
pub const HEADER_LEN: usize = 16;
/// The game itself: eight banks of 16 KiB.
pub const BODY_LEN: usize = 0x20000;
/// The whole file, header and game.
pub const FILE_LEN: usize = HEADER_LEN + BODY_LEN;

/// SHA-1 of the 128 KiB PRG body of Zelda (USA), header excluded; the
/// No-Intro DAT lists the same values for its headerless images.
pub const PRG0_BODY_SHA1: &str = "a12d74c73a0481599a5d832361d168f4737bbcf6";
/// The same for Rev 1.
pub const REV1_BODY_SHA1: &str = "be2f5dc8c5ba8ec1a344a71f9fb204750af24fe7";

/// The two US releases the Zelda Randomizer accepts. Its fixed offsets were
/// made for PRG0: on Rev 1 the seed and version lines of its title screen
/// land inside the logo's frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Revision {
    /// The first release, "PRG0".
    Prg0,
    /// The second, "PRG1" or "Rev 1".
    Rev1,
}

impl Revision {
    /// Both, the first release first.
    pub const ALL: [Revision; 2] = [Revision::Prg0, Revision::Rev1];

    /// A short name for files and settings: `prg0`, `rev1`.
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

    /// The name No-Intro gives the image.
    pub fn name(self) -> &'static str {
        match self {
            Revision::Prg0 => "Legend of Zelda, The (USA)",
            Revision::Rev1 => "Legend of Zelda, The (USA) (Rev 1)",
        }
    }
}

/// Why a file is no ROM this crate can read.
#[derive(Debug)]
#[non_exhaustive]
pub enum RomError {
    /// The file could not be read.
    Io(std::io::Error),
    /// The file has this many bytes, not `FILE_LEN`.
    Size(usize),
    /// The file does not start with an iNES header.
    Header,
    /// The game is neither release, by this SHA-1; a randomized ROM is not
    /// either, `Bytes` takes it.
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

/// The bytes of a Zelda ROM, the original or a randomized one, checked to
/// be an iNES image of the right size, so that every offset this crate
/// reads lies inside it.
#[derive(Clone, Copy)]
pub struct Bytes<'a>(&'a [u8; FILE_LEN]);

impl<'a> Bytes<'a> {
    /// Checks `data` for the size and the iNES header of a Zelda ROM.
    pub fn new(data: &'a [u8]) -> Result<Bytes<'a>, RomError> {
        let data: &[u8; FILE_LEN] = data.try_into().map_err(|_| RomError::Size(data.len()))?;
        if &data[..4] != b"NES\x1a" {
            return Err(RomError::Header);
        }
        Ok(Bytes(data))
    }

    /// The bytes, `FILE_LEN` of them.
    pub fn as_slice(&self) -> &'a [u8] {
        self.0
    }
}

impl std::ops::Deref for Bytes<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0
    }
}

impl fmt::Debug for Bytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bytes({} bytes)", self.0.len())
    }
}

impl<'a> From<&'a Rom> for Bytes<'a> {
    fn from(rom: &'a Rom) -> Self {
        Bytes::new(&rom.data).expect("a Rom has been checked")
    }
}

/// An unmodified ROM of either release, owned, to make a randomized one
/// from.
#[derive(Clone)]
pub struct Rom {
    data: Vec<u8>,
    revision: Revision,
}

impl Rom {
    /// Reads and checks the file at `path`.
    pub fn load(path: &Path) -> Result<Rom, RomError> {
        Rom::from_bytes(std::fs::read(path)?)
    }

    /// Checks `data` for size, header and the SHA-1 of either release.
    pub fn from_bytes(data: Vec<u8>) -> Result<Rom, RomError> {
        Bytes::new(&data)?;
        let sum = sha1_smol::Sha1::from(&data[HEADER_LEN..])
            .digest()
            .to_string();
        let revision = match sum.as_str() {
            PRG0_BODY_SHA1 => Revision::Prg0,
            REV1_BODY_SHA1 => Revision::Rev1,
            _ => return Err(RomError::Checksum(sum)),
        };
        Ok(Rom { data, revision })
    }

    /// Which release this was loaded as; bytes written since do not change
    /// it.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// The whole file.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// The byte at `offset`.
    ///
    /// # Panics
    /// When `offset` is past the end of the file.
    pub fn get(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    /// Writes one byte at `offset`.
    ///
    /// # Panics
    /// When `offset` is past the end of the file.
    pub fn set(&mut self, offset: usize, value: u8) {
        self.data[offset] = value;
    }

    /// Writes `values` from `offset` on.
    ///
    /// # Panics
    /// When they would run past the end of the file.
    pub fn write(&mut self, offset: usize, values: &[u8]) {
        self.data[offset..offset + values.len()].copy_from_slice(values);
    }
}

/// Where the game keeps the six characters it shows before a level's
/// digit: "LEVEL-" unless the level-string setting picked another word.
pub const LEVEL_NAME: usize = 0x19d17;

/// The word the game shows before a level's digit, as it shows it:
/// "LEVEL-", "BOARD-", "SECTOR", "LAIR -", ...
pub fn level_name(rom: Bytes<'_>) -> String {
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
pub fn level_title(rom: Bytes<'_>, level: i32) -> String {
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
    fn revision_stays_what_was_loaded() {
        let mut rom = Rom {
            data: blank(),
            revision: Revision::Prg0,
        };
        rom.set(HEADER_LEN, 0xff);
        assert_eq!(rom.revision(), Revision::Prg0);
    }

    fn blank() -> Vec<u8> {
        let mut data = vec![0; FILE_LEN];
        data[..4].copy_from_slice(b"NES\x1a");
        data
    }

    #[test]
    fn bytes_take_any_rom_of_the_right_size_with_a_header() {
        let data = blank();
        assert!(Bytes::new(&data).is_ok());
        assert!(matches!(Bytes::new(&data[1..]), Err(RomError::Size(n)) if n == FILE_LEN - 1));
        assert!(matches!(
            Bytes::new(&vec![0; FILE_LEN]),
            Err(RomError::Header)
        ));
    }

    #[test]
    fn level_names_as_the_game_shows_them() {
        let mut data = blank();
        let name = |data: &[u8]| level_name(Bytes::new(data).unwrap());
        let title = |data: &[u8], level| level_title(Bytes::new(data).unwrap(), level);
        // The game's own "LEVEL-", with its own dash.
        data[LEVEL_NAME..LEVEL_NAME + 6].copy_from_slice(&[21, 14, 31, 14, 21, 0x62]);
        assert_eq!(name(&data), "LEVEL-");
        assert_eq!(title(&data, 3), "Level 3");
        // "LAIR -" and "SECTOR" as the Zelda Randomizer writes them.
        data[LEVEL_NAME..LEVEL_NAME + 6].copy_from_slice(&[21, 10, 18, 27, 36, 47]);
        assert_eq!(name(&data), "LAIR -");
        assert_eq!(title(&data, 7), "Lair 7");
        data[LEVEL_NAME..LEVEL_NAME + 6].copy_from_slice(&[28, 14, 12, 29, 24, 27]);
        assert_eq!(title(&data, 1), "Sector 1");
    }
}
