//! Overworld screens as the game writes them to the name table: 16 columns
//! of 11 squares, each square 2 x 2 tiles.

use crate::rom::Bytes;

/// 16 pointers into the column heap, in bank 5.
const COLUMN_DIRECTORY: usize = 0x19d1f;
const BANK5: usize = 0x14010 - 0x8000;
/// 16 column ids per screen layout.
const LAYOUTS: usize = 0x15428;
/// Low two bits: palette of the outer two squares.
const TABLE0: usize = 0x18410;
/// Low two bits: palette inside.
const TABLE1: usize = 0x18490;
/// Low seven bits: screen layout.
const TABLE3: usize = 0x18590;
/// First tile of each square.
const PRIMARY: usize = 0x1698c;
/// First tile for primary values E5-EA.
const SECONDARY: usize = 0x16986;
/// Squares 0-15: four tiles each.
const FOURS: usize = 0x169c4;

pub struct Screen {
    /// 22 rows of 32 tiles
    pub tiles: [[u8; 32]; 22],
    /// 11 rows of 16 squares, background palette per square
    pub palettes: [[u8; 16]; 11],
    /// 11 rows of 16 squares as the column data names them
    pub squares: [[u8; 16]; 11],
}

/// A column id's high nibble picks a directory entry; from there the low
/// nibble counts bytes with bit 7 set. Each byte is a square in bits 0-5,
/// which bit 6 repeats once.
fn column(rom: Bytes<'_>, id: u8) -> [u8; 11] {
    let d = COLUMN_DIRECTORY + 2 * usize::from(id >> 4);
    let mut p = usize::from(rom[d]) + (usize::from(rom[d + 1]) << 8) + BANK5;
    let mut starts = 0;
    loop {
        if rom[p] & 0x80 != 0 {
            if starts == id & 15 {
                break;
            }
            starts += 1;
        }
        p += 1;
    }
    let mut squares = [0; 11];
    let mut n = 0;
    while n < 11 {
        let b = rom[p];
        p += 1;
        squares[n] = b & 0x3f;
        n += 1;
        if b & 0x40 != 0 && n < 11 {
            squares[n] = b & 0x3f;
            n += 1;
        }
    }
    squares
}

/// Top left, bottom left, top right, bottom right.
fn square_tiles(rom: Bytes<'_>, square: u8) -> [u8; 4] {
    let s = usize::from(square);
    if s < 16 {
        return [
            rom[FOURS + 4 * s],
            rom[FOURS + 4 * s + 1],
            rom[FOURS + 4 * s + 2],
            rom[FOURS + 4 * s + 3],
        ];
    }
    let mut first = rom[PRIMARY + s];
    if first >= 0xe5 {
        first = rom[SECONDARY + usize::from(first - 0xe5)];
    }
    [first, first + 1, first + 2, first + 3]
}

pub fn decode_screen(rom: Bytes<'_>, screen: usize) -> Screen {
    let layout = usize::from(rom[TABLE3 + screen] & 0x7f);
    let mut s = Screen {
        tiles: [[0; 32]; 22],
        palettes: [[0; 16]; 11],
        squares: [[0; 16]; 11],
    };
    for c in 0..16 {
        let squares = column(rom, rom[LAYOUTS + 16 * layout + c]);
        for (r, &square) in squares.iter().enumerate() {
            let [tl, bl, tr, br] = square_tiles(rom, square);
            s.tiles[2 * r][2 * c] = tl;
            s.tiles[2 * r + 1][2 * c] = bl;
            s.tiles[2 * r][2 * c + 1] = tr;
            s.tiles[2 * r + 1][2 * c + 1] = br;
            s.squares[r][c] = square;
            let inside = (2..=8).contains(&r) && (2..=13).contains(&c);
            s.palettes[r][c] = rom[if inside { TABLE1 } else { TABLE0 } + screen] & 3;
        }
    }
    s
}
