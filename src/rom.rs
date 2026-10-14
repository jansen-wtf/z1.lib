//! iNES image of The Legend of Zelda (USA, Rev 1).
//!
//! All offsets in this crate are file offsets including the 16-byte iNES
//! header, which is how the ROM layout is commonly documented.

use std::fmt;
use std::path::Path;

pub const HEADER_LEN: usize = 16;
pub const BODY_LEN: usize = 0x20000;
pub const FILE_LEN: usize = HEADER_LEN + BODY_LEN;

/// SHA-1 of the 128 KiB PRG body of Zelda (USA) Rev 1, header excluded.
pub const REV1_BODY_SHA1: &str = "be2f5dc8c5ba8ec1a344a71f9fb204750af24fe7";

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
                "ROM body SHA-1 is {sum}, expected {REV1_BODY_SHA1} (Legend of Zelda, The (USA) (Rev 1))"
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
        let sum = sha1_smol::Sha1::from(&data[HEADER_LEN..]).digest().to_string();
        if sum != REV1_BODY_SHA1 {
            return Err(RomError::Checksum(sum));
        }
        Ok(Rom { data })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_size() {
        assert!(matches!(Rom::from_bytes(vec![0; 100]), Err(RomError::Size(100))));
    }

    #[test]
    fn rejects_missing_header() {
        assert!(matches!(Rom::from_bytes(vec![0; FILE_LEN]), Err(RomError::Header)));
    }

    #[test]
    fn rejects_other_dumps() {
        let mut data = vec![0; FILE_LEN];
        data[..4].copy_from_slice(b"NES\x1a");
        assert!(matches!(Rom::from_bytes(data), Err(RomError::Checksum(_))));
    }
}
