//! A minimal PNG writer: 8-bit, indexed when the picture has at most 256
//! colors (as maps drawn from NES colors do), RGB otherwise; no filtering,
//! zlib from `miniz_oxide`.

use std::collections::HashMap;

use super::Image;

const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut crc = n as u32;
        let mut k = 0;
        while k < 8 {
            crc = if crc & 1 != 0 {
                0xedb8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
            k += 1;
        }
        table[n] = crc;
        n += 1;
    }
    table
};

fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(!0u32, |crc, &b| {
        CRC_TABLE[((crc ^ u32::from(b)) & 0xff) as usize] ^ (crc >> 8)
    })
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// The picture's colors in order of appearance, with each pixel's index;
/// `None` past 256 colors.
fn indexed(image: &Image) -> Option<(Vec<u32>, Vec<u8>)> {
    let mut palette = Vec::new();
    let mut index = HashMap::new();
    let mut pixels = Vec::with_capacity(image.pixels.len());
    let mut last = None;
    for &c in &image.pixels {
        let i = match last {
            Some((color, i)) if color == c => i,
            _ => {
                let i = match index.get(&c) {
                    Some(&i) => i,
                    None if palette.len() == 256 => return None,
                    None => {
                        palette.push(c);
                        index.insert(c, (palette.len() - 1) as u8);
                        (palette.len() - 1) as u8
                    }
                };
                last = Some((c, i));
                i
            }
        };
        pixels.push(i);
    }
    Some((palette, pixels))
}

pub(super) fn encode(image: &Image) -> Vec<u8> {
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&(image.width as u32).to_be_bytes());
    header.extend_from_slice(&(image.height as u32).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let raw = if let Some((palette, pixels)) = indexed(image) {
        header.extend_from_slice(&[8, 3, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &header);
        let plte: Vec<u8> = palette
            .iter()
            .flat_map(|&c| [(c >> 16) as u8, (c >> 8) as u8, c as u8])
            .collect();
        chunk(&mut out, b"PLTE", &plte);
        let mut raw = Vec::with_capacity((image.width + 1) * image.height);
        for row in pixels.chunks(image.width) {
            raw.push(0);
            raw.extend_from_slice(row);
        }
        raw
    } else {
        header.extend_from_slice(&[8, 2, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &header);
        let mut raw = Vec::with_capacity((image.width * 3 + 1) * image.height);
        for row in image.pixels.chunks(image.width) {
            raw.push(0);
            for &c in row {
                raw.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
            }
        }
        raw
    };
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&raw, LEVEL),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

const LEVEL: u8 = 9;

#[cfg(test)]
mod tests {
    use super::crc32;

    #[test]
    fn crc_matches_the_png_check_value() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }
}
