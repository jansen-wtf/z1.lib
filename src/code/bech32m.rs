//! Bech32m (BIP-350): a kind, the separator `1`, data in a 32-character
//! alphabet and a six-character BCH checksum that finds up to four wrong
//! characters for sure.

use std::fmt;

const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const BECH32M: u32 = 0x2bc8_30a3;
const CHECKSUM_LEN: usize = 6;

/// Why a string is no Bech32m string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bech32Error {
    /// Upper and lower case mixed.
    MixedCase,
    /// No `1` with a kind before it and a checksum after it.
    NoSeparator,
    /// A character outside the alphabet, at this position.
    Character(usize),
    /// The checksum does not match: a character is wrong or missing.
    Checksum,
}

impl fmt::Display for Bech32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bech32Error::MixedCase => write!(f, "upper and lower case mixed"),
            Bech32Error::NoSeparator => write!(f, "no '1' between kind and data"),
            Bech32Error::Character(at) => write!(f, "character {} is not allowed", at + 1),
            Bech32Error::Checksum => write!(f, "a character is wrong or missing"),
        }
    }
}

fn polymod(values: impl Iterator<Item = u8>) -> u32 {
    const GENERATOR: [u32; 5] = [
        0x3b6a_57b2,
        0x2650_8e6d,
        0x1ea1_19fa,
        0x3d42_33dd,
        0x2a14_62b3,
    ];
    let mut check = 1u32;
    for v in values {
        let top = check >> 25;
        check = ((check & 0x1ff_ffff) << 5) ^ u32::from(v);
        for (i, g) in GENERATOR.iter().enumerate() {
            if (top >> i) & 1 == 1 {
                check ^= g;
            }
        }
    }
    check
}

fn kind_values(kind: &str) -> impl Iterator<Item = u8> + '_ {
    kind.bytes()
        .map(|b| b >> 5)
        .chain(std::iter::once(0))
        .chain(kind.bytes().map(|b| b & 31))
}

/// The string for `kind` and 5-bit `data`, in lower case.
pub fn encode(kind: &str, data: &[u8]) -> String {
    let check = polymod(
        kind_values(kind)
            .chain(data.iter().copied())
            .chain([0; CHECKSUM_LEN]),
    ) ^ BECH32M;
    let mut out = String::with_capacity(kind.len() + 1 + data.len() + CHECKSUM_LEN);
    out.push_str(kind);
    out.push('1');
    for &d in data {
        out.push(char::from(ALPHABET[usize::from(d)]));
    }
    for i in 0..CHECKSUM_LEN {
        out.push(char::from(
            ALPHABET[((check >> (5 * (5 - i))) & 31) as usize],
        ));
    }
    out
}

/// The kind, in lower case, and the 5-bit data of `text`.
pub fn decode(text: &str) -> Result<(String, Vec<u8>), Bech32Error> {
    if text.bytes().any(|b| b.is_ascii_lowercase()) && text.bytes().any(|b| b.is_ascii_uppercase())
    {
        return Err(Bech32Error::MixedCase);
    }
    let text = text.to_ascii_lowercase();
    let at = text.rfind('1').ok_or(Bech32Error::NoSeparator)?;
    if at == 0 || at + 1 + CHECKSUM_LEN > text.len() {
        return Err(Bech32Error::NoSeparator);
    }
    let (kind, rest) = (&text[..at], &text[at + 1..]);
    if let Some(bad) = kind.bytes().position(|b| !(33..=126).contains(&b)) {
        return Err(Bech32Error::Character(bad));
    }
    let mut data = Vec::with_capacity(rest.len());
    for (i, b) in rest.bytes().enumerate() {
        let v = ALPHABET
            .iter()
            .position(|&a| a == b)
            .ok_or(Bech32Error::Character(at + 1 + i))?;
        data.push(v as u8);
    }
    if polymod(kind_values(kind).chain(data.iter().copied())) != BECH32M {
        return Err(Bech32Error::Checksum);
    }
    data.truncate(data.len() - CHECKSUM_LEN);
    Ok((kind.to_string(), data))
}

/// Bytes as 5-bit groups, the last one padded with zeros.
pub fn to_five_bits(bytes: &[u8]) -> Vec<u8> {
    let (mut acc, mut bits, mut out) = (0u32, 0, Vec::new());
    for &b in bytes {
        acc = (acc << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(((acc >> bits) & 31) as u8);
        }
    }
    if bits > 0 {
        out.push(((acc << (5 - bits)) & 31) as u8);
    }
    out
}

/// 5-bit groups as bytes; `None` when the padding is not zeros or too long.
pub fn from_five_bits(groups: &[u8]) -> Option<Vec<u8>> {
    let (mut acc, mut bits, mut out) = (0u32, 0, Vec::new());
    for &g in groups {
        acc = (acc << 5) | u32::from(g);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xff) as u8);
        }
    }
    (bits < 5 && acc & ((1 << bits) - 1) == 0).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_valid_strings_of_bip_350() {
        for s in [
            "A1LQFN3A",
            "a1lqfn3a",
            "an83characterlonghumanreadablepartthatcontainsthetheexcludedcharactersbioandnumber11sg7hg6",
            "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryx",
            "11llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllludsr8",
            "split1checkupstagehandshakeupstreamerranterredcaperredlc445v",
            "?1v759aa",
        ] {
            let (kind, data) = decode(s).unwrap_or_else(|e| panic!("{s}: {e}"));
            assert_eq!(encode(&kind, &data), s.to_ascii_lowercase());
        }
    }

    #[test]
    fn every_single_wrong_character_is_found() {
        let s = encode("z1r", &to_five_bits(b"the sword caves"));
        for at in 4..s.len() {
            let mut wrong = s.clone().into_bytes();
            wrong[at] = if wrong[at] == b'q' { b'p' } else { b'q' };
            assert_eq!(
                decode(std::str::from_utf8(&wrong).unwrap()),
                Err(Bech32Error::Checksum)
            );
        }
        assert_eq!(decode("Z1r1qqqqqq"), Err(Bech32Error::MixedCase));
        assert_eq!(decode("zxrqqqqqq"), Err(Bech32Error::NoSeparator));
    }

    #[test]
    fn five_bits_round_trip() {
        for bytes in [&b""[..], b"a", b"z1r", &[0xff; 17]] {
            assert_eq!(from_five_bits(&to_five_bits(bytes)).unwrap(), bytes);
        }
    }
}
