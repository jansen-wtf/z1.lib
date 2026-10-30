//! The z1r.rs string: a whole game of z1r.rs, the Zelda Randomizer's port,
//! in one Bech32m string, `z1r1` and the format version first. z1r.rs
//! names its files with it, so a program can read a game's flags from the
//! file name.
//!
//! After the version, the data is a list of fields, each a permanent number,
//! a length and a value, both numbers as LEB128. Only fields that are set
//! are written. A number is never reused for another meaning; a field this
//! version does not know means a newer z1r.rs made the string, and it is
//! refused rather than read as a different game.

use std::fmt;
use std::str::FromStr;

pub mod bech32m;

use bech32m::Bech32Error;

/// How many color sets a level can take.
pub const LEVEL_COLORS: usize = 24;

/// What z1r.rs adds to the Zelda Randomizer's game. The default adds
/// nothing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extensions {
    /// Hearts the white sword cave asks for, 1-16.
    pub white_sword_hearts: Option<u8>,
    /// Hearts the magical sword cave asks for, 1-16.
    pub magical_sword_hearts: Option<u8>,
    /// Levels 1-9: the color set each one takes, below `LEVEL_COLORS`.
    pub level_colors: [Option<u8>; 9],
}

impl Extensions {
    /// Whether anything is set, which makes the game one the Zelda
    /// Randomizer cannot make.
    pub fn any(&self) -> bool {
        *self != Extensions::default()
    }
}

/// What a z1r.rs string starts with, before the separator.
pub const KIND: &str = "z1r";
/// The format version this z1r.rs writes and reads.
pub const VERSION: u8 = 1;

/// The flag string, packed six bits a character.
const FLAGS: u64 = 0;
/// The seed number.
const SEED: u64 = 1;
const WHITE_SWORD_HEARTS: u64 = 2;
const MAGICAL_SWORD_HEARTS: u64 = 3;
/// Nine bytes, levels 1-9: a color set, or `NO_COLOR`.
const LEVEL_COLOR_SETS: u64 = 4;
const NO_COLOR: u8 = 0xff;

/// The flag string's 63 characters, as the Zelda Randomizer numbers them.
const FLAG_ALPHABET: &[u8; 63] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!";

/// A game as one string: the Zelda Randomizer's flags, optionally the seed,
/// and what z1r.rs adds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Code {
    /// The Zelda Randomizer's flag string.
    pub flags: String,
    /// The seed number, when the string names one.
    pub seed: Option<u64>,
    /// What z1r.rs adds.
    pub extensions: Extensions,
}

/// Why a string is no z1r.rs string this version can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeError {
    /// Not a Bech32m string, or one with a wrong character.
    Bech32(Bech32Error),
    /// Another kind of Bech32m string, such as an address.
    Kind(String),
    /// A newer format than this z1r.rs reads.
    Version(u8),
    /// A field of a newer z1r.rs.
    Field(u64),
    /// A field whose value cannot be.
    Value(&'static str),
    /// The data ends inside a field.
    Truncated,
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeError::Bech32(e) => write!(f, "not a z1r.rs string: {e}"),
            CodeError::Kind(k) => write!(f, "a \"{k}\" string, not a z1r.rs one"),
            CodeError::Version(v) => write!(
                f,
                "format {v}, made by a newer z1r.rs; this one reads format {VERSION}"
            ),
            CodeError::Field(n) => write!(f, "setting {n} is from a newer z1r.rs"),
            CodeError::Value(what) => write!(f, "the {what} cannot be"),
            CodeError::Truncated => write!(f, "the string ends inside a setting"),
        }
    }
}

impl std::error::Error for CodeError {}

fn put_number(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let byte = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn take_number(data: &mut &[u8]) -> Result<u64, CodeError> {
    let mut n = 0u64;
    for shift in (0..64).step_by(7) {
        let (&byte, rest) = data.split_first().ok_or(CodeError::Truncated)?;
        *data = rest;
        n |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(n);
        }
    }
    Err(CodeError::Value("number"))
}

fn put_field(out: &mut Vec<u8>, number: u64, value: &[u8]) {
    put_number(out, number);
    put_number(out, value.len() as u64);
    out.extend_from_slice(value);
}

/// The flag string as six-bit characters, its length first.
fn pack_flags(flags: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    put_number(&mut out, flags.len() as u64);
    let (mut acc, mut bits) = (0u32, 0);
    for b in flags.bytes() {
        acc = (acc << 6) | FLAG_ALPHABET.iter().position(|&a| a == b)? as u32;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    if bits > 0 {
        out.push((acc << (8 - bits)) as u8);
    }
    Some(out)
}

fn unpack_flags(mut value: &[u8]) -> Result<String, CodeError> {
    let len = take_number(&mut value)? as usize;
    let (mut acc, mut bits, mut out) = (0u32, 0, String::new());
    for &b in value {
        acc = (acc << 8) | u32::from(b);
        bits += 8;
        while bits >= 6 && out.len() < len {
            bits -= 6;
            let i = ((acc >> bits) & 63) as usize;
            let c = FLAG_ALPHABET
                .get(i)
                .ok_or(CodeError::Value("flag string"))?;
            out.push(char::from(*c));
        }
    }
    if out.len() != len {
        return Err(CodeError::Value("flag string"));
    }
    Ok(out)
}

fn hearts(value: &[u8], what: &'static str) -> Result<u8, CodeError> {
    match value {
        [h @ 1..=16] => Ok(*h),
        _ => Err(CodeError::Value(what)),
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut data = Vec::new();
        put_field(
            &mut data,
            FLAGS,
            &pack_flags(&self.flags).ok_or(fmt::Error)?,
        );
        if let Some(seed) = self.seed {
            let mut v = Vec::new();
            put_number(&mut v, seed);
            put_field(&mut data, SEED, &v);
        }
        let e = &self.extensions;
        if let Some(h) = e.white_sword_hearts {
            put_field(&mut data, WHITE_SWORD_HEARTS, &[h]);
        }
        if let Some(h) = e.magical_sword_hearts {
            put_field(&mut data, MAGICAL_SWORD_HEARTS, &[h]);
        }
        if e.level_colors.iter().any(Option::is_some) {
            let sets: Vec<u8> = e
                .level_colors
                .iter()
                .map(|c| c.unwrap_or(NO_COLOR))
                .collect();
            put_field(&mut data, LEVEL_COLOR_SETS, &sets);
        }
        let mut groups = vec![VERSION];
        groups.extend(bech32m::to_five_bits(&data));
        f.write_str(&bech32m::encode(KIND, &groups))
    }
}

impl FromStr for Code {
    type Err = CodeError;

    fn from_str(text: &str) -> Result<Code, CodeError> {
        let (kind, groups) = bech32m::decode(text.trim()).map_err(CodeError::Bech32)?;
        if kind != KIND {
            return Err(CodeError::Kind(kind));
        }
        let (&version, groups) = groups.split_first().ok_or(CodeError::Truncated)?;
        if version != VERSION {
            return Err(CodeError::Version(version));
        }
        let bytes = bech32m::from_five_bits(groups).ok_or(CodeError::Truncated)?;
        let mut data = &bytes[..];
        let mut code = Code::default();
        let mut has_flags = false;
        while !data.is_empty() {
            let number = take_number(&mut data)?;
            let len = take_number(&mut data)? as usize;
            if len > data.len() {
                return Err(CodeError::Truncated);
            }
            let (value, rest) = data.split_at(len);
            data = rest;
            match number {
                FLAGS => {
                    code.flags = unpack_flags(value)?;
                    has_flags = true;
                }
                SEED => code.seed = Some(take_number(&mut &value[..])?),
                WHITE_SWORD_HEARTS => {
                    code.extensions.white_sword_hearts = Some(hearts(value, "white sword hearts")?)
                }
                MAGICAL_SWORD_HEARTS => {
                    code.extensions.magical_sword_hearts =
                        Some(hearts(value, "magical sword hearts")?)
                }
                LEVEL_COLOR_SETS => {
                    let sets: [u8; 9] = value
                        .try_into()
                        .map_err(|_| CodeError::Value("level colors"))?;
                    for (slot, &set) in code.extensions.level_colors.iter_mut().zip(&sets) {
                        *slot = match set {
                            NO_COLOR => None,
                            s if usize::from(s) < LEVEL_COLORS => Some(s),
                            _ => return Err(CodeError::Value("level colors")),
                        };
                    }
                }
                n => return Err(CodeError::Field(n)),
            }
        }
        if !has_flags {
            return Err(CodeError::Value("flag string"));
        }
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Code {
        Code {
            flags: "1zuh3gHaClLJcHHNapqQB1pT23b50Y1NhXkgO863".into(),
            seed: Some(2447833244173793317),
            extensions: Extensions {
                white_sword_hearts: Some(7),
                magical_sword_hearts: Some(16),
                level_colors: [
                    Some(9),
                    None,
                    None,
                    Some(22),
                    None,
                    None,
                    None,
                    None,
                    Some(0),
                ],
            },
        }
    }

    #[test]
    fn a_game_round_trips_through_its_string() {
        let text = game().to_string();
        assert!(text.starts_with("z1r1p"), "{text}");
        assert_eq!(text.parse::<Code>(), Ok(game()));
        assert_eq!(text.to_uppercase().parse::<Code>(), Ok(game()));
        let plain = Code {
            flags: "O".into(),
            ..Code::default()
        };
        assert_eq!(plain.to_string().parse::<Code>(), Ok(plain));
    }

    #[test]
    fn strings_that_cannot_be_are_refused() {
        let text = game().to_string();
        let mut typo = text.clone().into_bytes();
        typo[12] = if typo[12] == b'q' { b'p' } else { b'q' };
        assert!(matches!(
            String::from_utf8(typo).unwrap().parse::<Code>(),
            Err(CodeError::Bech32(Bech32Error::Checksum))
        ));
        let address = bech32m::encode("bc", &[1, 2, 3]);
        assert_eq!(address.parse::<Code>(), Err(CodeError::Kind("bc".into())));
        let newer = bech32m::encode(KIND, &[VERSION + 1]);
        assert_eq!(newer.parse::<Code>(), Err(CodeError::Version(VERSION + 1)));
        let mut data = Vec::new();
        put_field(&mut data, FLAGS, &pack_flags("O").unwrap());
        put_field(&mut data, 99, &[1]);
        let mut groups = vec![VERSION];
        groups.extend(bech32m::to_five_bits(&data));
        let future = bech32m::encode(KIND, &groups);
        assert_eq!(future.parse::<Code>(), Err(CodeError::Field(99)));
    }
}
