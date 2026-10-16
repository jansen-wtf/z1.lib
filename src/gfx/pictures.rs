//! Small pictures for explaining options: an overworld screen with a spot
//! marked, a cave with the old man and what he offers, or a row of items.
//! Each is named by a short spec, so a user interface can ask for them:
//!
//! - `screen:<n>`: overworld screen `n`
//! - `armos`: the screen of the armos item, its statue framed
//! - `coast`: the coast screen with the item that waits there
//! - `cave` or `cave:<item>,...`: the old man between his fires, offering
//!   items
//! - `items:<item>,...`: items in a row

use super::{Image, NES, decode_screen, draw_item, draw_sprite, paint_screen};

/// The screens of the armos that hide stairs, the item's one first, and
/// their x positions.
pub const ARMOS_SCREENS: usize = 0x10cc2;
pub const ARMOS_X: usize = 0x10cc9;
/// First tile of an armos statue.
pub const ARMOS_TILE: u8 = 0xc0;
const ARMOS_ITEM: usize = 0x10d05;
/// The heart on the coast, reached with the ladder; measured in the game.
pub const COAST_SCREEN: usize = 0x5f;
pub const COAST_X: i32 = 200;
pub const COAST_Y: i32 = 80;
const COAST_ITEM: usize = 0x1789a;

const BLACK: u32 = 0x000000;
const GOLD: u32 = NES[0x28];

/// The picture a spec names, or `None` for a spec it does not know.
pub fn picture(rom: &[u8], spec: &str) -> Option<Image> {
    let (kind, arg) = spec.split_once(':').unwrap_or((spec, ""));
    let numbers = || -> Option<Vec<i32>> {
        arg.split(',')
            .map(|n| n.trim().parse().ok())
            .collect::<Option<Vec<i32>>>()
            .filter(|v| !v.is_empty())
    };
    match kind {
        "screen" => {
            let screen: usize = arg.parse().ok().filter(|&s| s < 128)?;
            Some(screen_picture(rom, screen))
        }
        "armos" => {
            let screen = usize::from(rom[ARMOS_SCREENS]) & 0x7f;
            let x = i32::from(rom[ARMOS_X]);
            let mut image = screen_picture(rom, screen);
            let tiles = decode_screen(rom, screen).tiles;
            let row = (0..11)
                .find(|&r| tiles[2 * r][2 * (x / 16) as usize] == ARMOS_TILE)
                .unwrap_or(5) as i32;
            frame(&mut image, 16 * (x / 16), 16 * row);
            let item = i32::from(rom[ARMOS_ITEM] & 0x3f);
            draw_item(rom, &mut image, item, 16 * (x / 16) + 8, 16 * row - 20);
            Some(image)
        }
        "coast" => {
            let mut image = screen_picture(rom, COAST_SCREEN);
            let item = i32::from(rom[COAST_ITEM] & 0x3f);
            draw_item(rom, &mut image, item, COAST_X, COAST_Y);
            Some(image)
        }
        "cave" if arg.is_empty() => Some(cave_picture(rom, &[])),
        "cave" => Some(cave_picture(rom, &numbers()?)),
        "items" => Some(items_picture(rom, &numbers()?)),
        _ => None,
    }
}

fn screen_picture(rom: &[u8], screen: usize) -> Image {
    let mut image = Image::new(256, 176, BLACK);
    paint_screen(rom, screen, &mut image, 0, 0);
    image
}

/// A gold frame around the square at `x`, `y`.
fn frame(image: &mut Image, x: i32, y: i32) {
    for (dx, dy, w, h) in [
        (-2, -2, 20, 2),
        (-2, 16, 20, 2),
        (-2, 0, 2, 16),
        (16, 0, 2, 16),
    ] {
        image.fill(x + dx, y + dy, w, h, GOLD);
    }
}

/// The old man (tile 98, mirrored) between two fires (tiles 5E and 5C,
/// mirrored), all in palette 2, as the game draws a cave; items below.
fn cave_picture(rom: &[u8], items: &[i32]) -> Image {
    let width = 128.max(24 * items.len() as i32 + 16);
    let mut image = Image::new(width as usize, 48, BLACK);
    let mid = width / 2;
    for fire in [mid - 56, mid + 40] {
        draw_sprite(rom, &mut image, 0x5e, 2, true, fire, 4);
        draw_sprite(rom, &mut image, 0x5c, 2, true, fire + 8, 4);
    }
    draw_sprite(rom, &mut image, 0x98, 2, false, mid - 8, 4);
    draw_sprite(rom, &mut image, 0x98, 2, true, mid, 4);
    row_of_items(rom, &mut image, items, 28);
    image
}

fn items_picture(rom: &[u8], items: &[i32]) -> Image {
    let mut image = Image::new(24 * items.len(), 20, BLACK);
    row_of_items(rom, &mut image, items, 2);
    image
}

fn row_of_items(rom: &[u8], image: &mut Image, items: &[i32], y: i32) {
    let first = (image.width as i32 - 24 * items.len() as i32) / 2 + 12;
    for (k, &item) in items.iter().enumerate() {
        draw_item(rom, image, item, first + 24 * k as i32, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_specs_give_nothing() {
        let rom = vec![0u8; 0x20010];
        assert!(picture(&rom, "screen:200").is_none());
        assert!(picture(&rom, "items:").is_none());
        assert!(picture(&rom, "nonsense").is_none());
        assert_eq!(picture(&rom, "items:1,2").map(|i| i.width), Some(48));
    }
}
