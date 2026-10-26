//! Pictures drawn from a ROM's own graphics: overworld screens, the game's
//! font and item sprites. Nothing here holds game graphics; everything is
//! read from the ROM passed in. The addresses are documented, with how they
//! were measured, in `docs/rom-graphics.md`.

mod overworld;
mod png;
pub mod underworld;

use crate::rom::Bytes;

pub use overworld::{Screen, decode_screen};

/// NES colors, as most emulators show them.
pub const NES: [u32; 64] = [
    0x7c7c7c, 0x0000fc, 0x0000bc, 0x4428bc, 0x940084, 0xa80020, 0xa81000, 0x881400, 0x503000,
    0x007800, 0x006800, 0x005800, 0x004058, 0x000000, 0x000000, 0x000000, 0xbcbcbc, 0x0078f8,
    0x0058f8, 0x6844fc, 0xd800cc, 0xe40058, 0xf83800, 0xe45c10, 0xac7c00, 0x00b800, 0x00a800,
    0x00a844, 0x008888, 0x000000, 0x000000, 0x000000, 0xf8f8f8, 0x3cbcfc, 0x6888fc, 0x9878f8,
    0xf878f8, 0xf85898, 0xf87858, 0xfca044, 0xf8b800, 0xb8f818, 0x58d854, 0x58f898, 0x00e8d8,
    0x787878, 0x000000, 0x000000, 0xfcfcfc, 0xa4e4fc, 0xb8b8f8, 0xd8b8f8, 0xf8b8f8, 0xf8a4c0,
    0xf0d0b0, 0xfce0a8, 0xf8d878, 0xd8f878, 0xb8f8b8, 0xb8f8d8, 0x00fcfc, 0xf8d8f8, 0x000000,
    0x000000,
];

/// Overworld background palettes, four of four NES colors: the start of
/// the overworld's block.
const BACKGROUND_PALETTES: usize = crate::level::BLOCKS;
/// Overworld sprite palettes; 0 is Link's, 1 blue and white, 2 red and white.
const SPRITE_PALETTES: usize = 0x19323;

/// An RGB picture, one `0xRRGGBB` per pixel.
pub struct Image {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Row after row, `width` pixels each.
    pub pixels: Vec<u32>,
}

impl Image {
    /// A picture all in `color`.
    pub fn new(width: usize, height: usize, color: u32) -> Self {
        Image {
            width,
            height,
            pixels: vec![color; width * height],
        }
    }

    /// Sets a pixel; pixels outside the picture are dropped.
    pub fn put(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    /// Fills a rectangle; what lies outside the picture is dropped.
    pub fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        for j in 0..h {
            for i in 0..w {
                self.put(x + i, y + j, color);
            }
        }
    }

    /// The picture as a PNG file.
    pub fn to_png(&self) -> Vec<u8> {
        png::encode(self)
    }

    /// The picture as RGBA bytes, for a canvas.
    pub fn to_rgba(&self) -> Vec<u8> {
        self.pixels
            .iter()
            .flat_map(|&c| [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255])
            .collect()
    }
}

/// The color of palette entry `index` (0-3) in a palette table.
fn palette_color(rom: Bytes<'_>, table: usize, palette: usize, index: usize) -> u32 {
    let entry = if index == 0 {
        table
    } else {
        table + 4 * palette + index
    };
    NES[usize::from(rom[entry] & 0x3f)]
}

/// Background tile `t` comes from one of three places.
fn background_tile(tile: u8) -> usize {
    let t = usize::from(tile);
    match t {
        0..0x70 => 0x878f + 16 * t,
        0x70..0xf2 => 0xc94b + 16 * (t - 0x70),
        _ => 0x8e8f + 16 * (t - 0xf2),
    }
}

/// Sprite tile `t` (the top of an 8 x 16 pair) on the overworld.
fn sprite_tile(tile: u8) -> usize {
    let t = usize::from(tile);
    match t {
        0..0x70 => 0x808f + 16 * t,
        0x70..0x8e => 0x4dc4 + 16 * (t - 0x70),
        _ => 0xd16b + 16 * (t - 0x8e),
    }
}

/// Draws one 8 x 16 overworld sprite with its top left at `x`, `y`, in
/// sprite palette `palette` (0 Link's, 1 blue and white, 2 red and white),
/// mirrored when `flip`.
pub fn draw_sprite(
    rom: Bytes<'_>,
    image: &mut Image,
    tile: u8,
    palette: usize,
    flip: bool,
    x: i32,
    y: i32,
) {
    let colors = [1, 2, 3].map(|v| palette_color(rom, SPRITE_PALETTES, palette, v));
    draw_sprite_in(rom, image, tile, colors, flip, x, y);
}

/// Draws one 8 x 16 sprite in the given colors 1-3.
fn draw_sprite_in(
    rom: Bytes<'_>,
    image: &mut Image,
    tile: u8,
    colors: [u32; 3],
    flip: bool,
    x: i32,
    y: i32,
) {
    let at = sprite_tile(tile);
    for row in 0..16 {
        let tile_at = at + (row / 8) * 16;
        for col in 0..8 {
            let sx = if flip { 7 - col } else { col };
            let v = tile_pixel(rom, tile_at, sx, row % 8);
            if v != 0 {
                image.put(x + col as i32, y + row as i32, colors[v - 1]);
            }
        }
    }
}

/// The two-bit color of pixel `x`, `y` of the tile data at `at`.
fn tile_pixel(rom: Bytes<'_>, at: usize, x: usize, y: usize) -> usize {
    let lo = rom[at + y];
    let hi = rom[at + y + 8];
    usize::from((lo >> (7 - x)) & 1) | (usize::from((hi >> (7 - x)) & 1) << 1)
}

/// Paints overworld screen `screen`, 256 x 176 pixels, at `x0`, `y0`.
///
/// # Panics
/// When `screen` is so far past 0x7F that its table entry lies past the
/// end of the ROM.
pub fn paint_screen(rom: Bytes<'_>, screen: usize, image: &mut Image, x0: i32, y0: i32) {
    let s = decode_screen(rom, screen);
    for r in 0..22 {
        for c in 0..32 {
            let at = background_tile(s.tiles[r][c]);
            let palette = usize::from(s.palettes[r / 2][c / 2]);
            for y in 0..8 {
                for x in 0..8 {
                    let v = tile_pixel(rom, at, x, y);
                    let color = palette_color(rom, BACKGROUND_PALETTES, palette, v);
                    image.put(x0 + (8 * c + x) as i32, y0 + (8 * r + y) as i32, color);
                }
            }
        }
    }
}

/// Item id -> sprites as the game draws the item: x offset, tile, palette,
/// mirrored. Measured in a headless emulator.
const ITEM_SPRITES: [&[(i32, u8, usize, bool)]; 36] = [
    &[(0, 0x34, 1, false)],
    &[(0, 0x20, 0, false)],
    &[(0, 0x20, 1, false)],
    &[(0, 0x48, 2, false)],
    &[(0, 0x22, 2, false)],
    &[(0, 0x24, 2, false)],
    &[(0, 0x26, 1, false)],
    &[(0, 0x26, 2, false)],
    &[(0, 0x28, 0, false)],
    &[(0, 0x28, 1, false)],
    &[(0, 0x2a, 0, false)],
    &[(0, 0x2c, 2, false)],
    &[(-4, 0x6c, 0, false), (4, 0x6c, 0, true)],
    &[(-4, 0x76, 0, false), (4, 0x76, 0, true)],
    &[],
    &[(0, 0x32, 1, false)],
    &[(0, 0x4a, 1, false)],
    &[(0, 0x42, 2, false)],
    &[(0, 0x46, 1, false)],
    &[(0, 0x46, 2, false)],
    &[(0, 0x4e, 2, false)],
    &[(0, 0x4c, 1, false)],
    &[(-4, 0x6a, 2, false), (3, 0x6a, 2, true)],
    &[(0, 0x4c, 2, false)],
    &[(0, 0x32, 1, false)],
    &[(0, 0x2e, 2, false)],
    &[(-4, 0x68, 2, false), (3, 0x68, 2, true)],
    &[(-4, 0x6e, 1, false), (4, 0x6e, 1, true)],
    &[(0, 0x56, 0, false)],
    &[(0, 0x36, 0, false)],
    &[(0, 0x36, 1, false)],
    &[(0, 0x40, 1, false)],
    &[(0, 0x40, 2, false)],
    &[(-4, 0x66, 2, false), (3, 0x66, 2, true)],
    &[],
    &[(0, 0x50, 2, false)],
];

/// Draws item `item` as the game does, its 8-pixel column centered on `cx`
/// and its top at `y`. Unknown items draw nothing.
pub fn draw_item(rom: Bytes<'_>, image: &mut Image, item: i32, cx: i32, y: i32) {
    let Some(sprites) = usize::try_from(item).ok().and_then(|i| ITEM_SPRITES.get(i)) else {
        return;
    };
    for &(dx, tile, palette, flip) in sprites.iter() {
        draw_sprite(rom, image, tile, palette, flip, cx - 4 + dx, y);
    }
}

/// The starting tunic color, an NES color.
const START_TUNIC: usize = 41623;

/// Link facing down (tile 5A, and 08 mirrored), as he stands on the start
/// screen, in his tunic color or the one given.
pub fn link_picture(rom: Bytes<'_>, tunic: Option<u8>) -> Image {
    let tunic = tunic.unwrap_or(rom[START_TUNIC]);
    let colors = [
        NES[usize::from(tunic & 0x3f)],
        palette_color(rom, SPRITE_PALETTES, 0, 2),
        palette_color(rom, SPRITE_PALETTES, 0, 3),
    ];
    let mut image = Image::new(24, 20, 0x000000);
    draw_sprite_in(rom, &mut image, 0x5a, colors, false, 4, 2);
    draw_sprite_in(rom, &mut image, 0x08, colors, true, 12, 2);
    image
}

/// The status bar's heart (background tile F2) in an NES color.
pub fn heart_picture(rom: Bytes<'_>, color: u8) -> Image {
    let mut image = Image::new(16, 12, 0x000000);
    let at = background_tile(0xf2);
    for y in 0..8 {
        for x in 0..8 {
            if tile_pixel(rom, at, x, y) != 0 {
                image.put(4 + x as i32, 2 + y as i32, NES[usize::from(color & 0x3f)]);
            }
        }
    }
    image
}

/// The game's text tile for a character; unknown characters are blanks.
fn font_tile(ch: char) -> u8 {
    match ch {
        '0'..='9' => ch as u8 - b'0',
        'A'..='Z' => ch as u8 - b'A' + 10,
        'a'..='z' => ch as u8 - b'a' + 10,
        ',' => 40,
        '!' => 41,
        '\'' => 42,
        '&' => 43,
        '.' => 44,
        '"' => 45,
        '?' => 46,
        '-' => 47,
        _ => 36,
    }
}

/// Writes `text` in the game's font, 8 pixels per character.
pub fn draw_text(rom: Bytes<'_>, image: &mut Image, text: &str, x: i32, y: i32, color: u32) {
    for (k, ch) in text.chars().enumerate() {
        let at = background_tile(font_tile(ch));
        for row in 0..8 {
            for col in 0..8 {
                if tile_pixel(rom, at, col, row) != 0 {
                    image.put(x + 8 * k as i32 + col as i32, y + row as i32, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_has_signature_and_size() {
        let image = Image::new(3, 2, 0x123456);
        let png = image.to_png();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[16..24], &[0, 0, 0, 3, 0, 0, 0, 2]);
    }

    #[test]
    fn every_item_but_two_has_sprites() {
        let missing: Vec<usize> = (0..36).filter(|&i| ITEM_SPRITES[i].is_empty()).collect();
        assert_eq!(missing, [14, 34]);
    }
}
