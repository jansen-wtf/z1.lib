//! The z1r.rs string: a whole game of z1r.rs, the Zelda Randomizer's port,
//! in one Bech32m string, `z1r1` and the format version first. z1r.rs
//! writes it into the ROM of every game it makes beyond the Zelda
//! Randomizer (`IN_ROM`), so a program can read the game's flags from it.
//!
//! After the version, the data is a list of fields, each a permanent number,
//! a length and a value, both numbers as LEB128. Field 0 is the flag string,
//! field 1 the seed; what the fields from 2 on mean is z1r.rs's to say, and
//! this module hands them over as they are. A number is never reused for
//! another meaning, so a program that does not know a field knows the
//! string is from a newer z1r.rs.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

pub mod bech32m;

use bech32m::Bech32Error;

/// What a z1r.rs string starts with, before the separator.
pub const KIND: &str = "z1r";
/// The format version this z1r.rs writes and reads.
pub const VERSION: u8 = 1;

/// The flag string, packed six bits a character.
const FLAGS: u64 = 0;
/// The seed number.
const SEED: u64 = 1;
/// The first number of a field z1r.rs gives its meaning.
pub const FIRST_FIELD: u64 = 2;

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
    /// What z1r.rs adds: values by field number, from `FIRST_FIELD` on.
    /// Numbers below it are not written.
    pub fields: BTreeMap<u64, Vec<u8>>,
}

/// Why a string is no z1r.rs string this version can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeError {
    /// Not a Bech32m string, or one with a wrong character.
    Bech32(Bech32Error),
    /// Another kind of Bech32m string, such as an address.
    Kind(String),
    /// A newer format than this module reads.
    Version(u8),
    /// The flag string or the seed cannot be.
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
        for (&number, value) in self.fields.range(FIRST_FIELD..) {
            put_field(&mut data, number, value);
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
                SEED => {
                    let mut v = value;
                    let seed = take_number(&mut v)?;
                    if !v.is_empty() {
                        return Err(CodeError::Value("seed"));
                    }
                    code.seed = Some(seed);
                }
                n => {
                    code.fields.insert(n, value.to_vec());
                }
            }
        }
        if !has_flags {
            return Err(CodeError::Value("flag string"));
        }
        Ok(code)
    }
}

/// Where a z1r.rs enhanced ROM keeps its own string: free space in bank 5
/// (`$BBF0`) that the game never runs or reads, unused by the Zelda
/// Randomizer. A tracker reads the game from the ROM, whatever the file is
/// called.
pub const IN_ROM: usize = 0x17c00;
/// The room there, up to the code at `0x17F60`.
pub const IN_ROM_LEN: usize = 0x360;
/// What the string in the ROM starts with, then its length (two bytes,
/// low first), then the string in ASCII.
pub const IN_ROM_MARK: &[u8; 6] = b"Z1R.RS";

impl Code {
    /// What z1r.rs writes at `IN_ROM`; `None` when the string does not fit.
    pub fn rom_bytes(&self) -> Option<Vec<u8>> {
        let text = self.to_string();
        let mut out = IN_ROM_MARK.to_vec();
        out.extend((text.len() as u16).to_le_bytes());
        out.extend(text.bytes());
        (out.len() <= IN_ROM_LEN).then_some(out)
    }

    /// The game a z1r.rs enhanced ROM keeps at `IN_ROM`, if it keeps one.
    pub fn in_rom(rom: crate::rom::Bytes<'_>) -> Option<Code> {
        let room = &rom[IN_ROM..IN_ROM + IN_ROM_LEN];
        let rest = room.strip_prefix(IN_ROM_MARK)?;
        let len = usize::from(u16::from_le_bytes([rest[0], rest[1]]));
        let text = std::str::from_utf8(rest.get(2..2 + len)?).ok()?;
        text.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Code {
        Code {
            flags: "1zuh3gHaClLJcHHNapqQB1pT23b50Y1NhXkgO863".into(),
            seed: Some(2447833244173793317),
            fields: BTreeMap::from([(2, vec![7]), (3, vec![16]), (4, vec![9, 0xff, 22])]),
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
    fn a_string_of_z1r_rs_1_0_reads_the_same() {
        let code: Code = "z1r1pqqyqjg4av9lcpp9gqyqswqspqgzqj9lllllllllllllsf5j2hr"
            .parse()
            .unwrap();
        assert_eq!(code.flags, "8hrXVu24g");
        assert_eq!(code.seed, Some(7));
        let mut colors = vec![0xff; 9];
        colors[0] = 23;
        assert_eq!(code.fields, BTreeMap::from([(2, vec![2]), (4, colors)]));
        assert_eq!(
            code.to_string(),
            "z1r1pqqyqjg4av9lcpp9gqyqswqspqgzqj9lllllllllllllsf5j2hr"
        );
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
        let later = bech32m::encode(KIND, &groups);
        assert_eq!(later.parse::<Code>().unwrap().fields[&99], [1]);
    }

    #[test]
    fn a_game_round_trips_through_its_rom() {
        let mut rom = vec![0xff; crate::rom::FILE_LEN];
        rom[..4].copy_from_slice(b"NES\x1a");
        assert_eq!(Code::in_rom(crate::rom::Bytes::new(&rom).unwrap()), None);
        let bytes = game().rom_bytes().unwrap();
        rom[IN_ROM..IN_ROM + bytes.len()].copy_from_slice(&bytes);
        assert_eq!(
            Code::in_rom(crate::rom::Bytes::new(&rom).unwrap()),
            Some(game())
        );
        let mut huge = game();
        huge.fields.insert(9, vec![0; IN_ROM_LEN]);
        assert_eq!(huge.rom_bytes(), None);
    }
}
