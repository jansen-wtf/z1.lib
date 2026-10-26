//! Dungeon rooms as the game draws them: walls and doors, the room's
//! layout of 12 columns of 7 squares, its items and its monsters. The
//! addresses are documented, with how they were measured, in
//! `docs/rom-graphics.md`.

use super::{Image, NES, tile_pixel};

/// 10 pointers into the column heap, in bank 5.
const COLUMN_DIRECTORY: usize = 0x16714;
const BANK5: usize = 0x14010 - 0x8000;
const BANK3: usize = 0xc010 - 0x8000;
/// 12 column ids per room layout.
const LAYOUTS: usize = 0x160ee;
/// First tile of each square.
const SQUARES: usize = 0x16728;
/// The left wall, 3 columns of 10 tiles, and the top wall, 12 columns of
/// 3 tiles (4 bytes apart); the other walls mirror them.
const WALL_SIDE: usize = 0x15fb0;
const WALL_TOP: usize = 0x15fce;
/// Five looks of 12 tiles per side: open, locked, shutter, wall, bombed.
const DOORS_EAST: usize = 0x15ffe;
const DOORS_WEST: usize = 0x1603a;
const DOORS_SOUTH: usize = 0x16076;
const DOORS_NORTH: usize = 0x160b2;
/// The room tables A-F, 0x80 bytes each, per level group and quest.
pub const GRID: usize = 0x18710;
/// Level blocks: palettes (background, then sprites), monster counts at
/// 0x21, item positions at 0x26.
const LEVEL_BLOCKS: usize = 0x19313;
const LEVEL_BLOCK_SIZE: usize = 0xfc;
/// Per level, bank 3 pointers to the sprites loaded at tile 0x9E
/// (monsters) and 0xC0 (bosses).
const MONSTER_SPRITES: usize = 0xc010;
const BOSS_SPRITES: usize = 0xc024;
/// Mixed monster groups 0x62-0x7F: pointers to their monster codes.
const MIXED_GROUPS: usize = 0x1474f;

/// How a door type looks: the game's view, where a bomb wall is still a
/// wall, or the map's, which shows it blown open.
const DOOR_LOOK: [usize; 8] = [0, 3, 3, 3, 3, 1, 1, 2];
const BOMBED: usize = 4;

pub struct Room {
    /// 22 rows of 32 tiles
    pub tiles: [[u8; 32]; 22],
    /// 11 rows of 16 squares, background palette per square
    pub palettes: [[u8; 16]; 11],
}

/// Which room tables a level uses: levels 7-9 have their own, and the
/// second quest has a set of both.
pub fn tables(level: usize, second_quest: bool) -> usize {
    (if second_quest { 0x600 } else { 0 }) + if level >= 7 { 0x300 } else { 0 }
}

/// Byte `room` of room table `t` (0 = A ... 5 = F).
pub fn grid(rom: &[u8], tables: usize, t: usize, room: usize) -> u8 {
    rom[GRID + tables + 0x80 * t + room]
}

fn level_block(level: usize) -> usize {
    LEVEL_BLOCKS + LEVEL_BLOCK_SIZE * level
}

/// Like the overworld's, but a column is 7 squares, each byte a square in
/// bits 0-3 repeated as often as bits 4-6 say, plus once.
fn column(rom: &[u8], id: u8) -> [u8; 7] {
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
    let mut squares = [0; 7];
    let mut n = 0;
    while n < 7 {
        let b = rom[p];
        p += 1;
        for _ in 0..=((b >> 4) & 7) {
            if n < 7 {
                squares[n] = b & 15;
                n += 1;
            }
        }
    }
    squares
}

/// The tile mirrored horizontally (`h`) and vertically (`v`): the game keeps
/// the mirrored wall tiles next to each other.
fn mirror(t: u8, h: bool, v: bool) -> u8 {
    match t {
        0xb8..=0xdb if (t - 0xb8).is_multiple_of(4) => {
            t + [0, 3, 1, 2][usize::from(h) | usize::from(v) << 1]
        }
        0xdc => t + u8::from(v),
        0xde => t + u8::from(h),
        0xe0 => t + u8::from(h != v),
        _ => t,
    }
}

/// Room `room` of `level` from the given room tables, as the game draws it
/// (`map` false) or with bomb walls blown open (`map` true).
pub fn decode_room(rom: &[u8], tables: usize, room: usize, map: bool) -> Room {
    let a = grid(rom, tables, 0, room);
    let b = grid(rom, tables, 1, room);
    let d = grid(rom, tables, 3, room);
    let mut tiles = [[0xf6u8; 32]; 22];
    let mut quarter = |r: usize, c: usize, t: u8| {
        tiles[r][c] = t;
        tiles[r][31 - c] = mirror(t, true, false);
        tiles[21 - r][c] = mirror(t, false, true);
        tiles[21 - r][31 - c] = mirror(t, true, true);
    };
    for c in 0..3 {
        for r in 0..10 {
            quarter(1 + r, 1 + c, rom[WALL_SIDE + 10 * c + r]);
        }
    }
    for c in 0..12 {
        for r in 0..3 {
            quarter(1 + r, 4 + c, rom[WALL_TOP + 4 * c + r]);
        }
    }
    let layout = usize::from(d & 0x3f);
    for c in 0..12 {
        let squares = column(rom, rom[LAYOUTS + 12 * layout + c]);
        for (r, &square) in squares.iter().enumerate() {
            let first = rom[SQUARES + usize::from(square)];
            // Below the dungeon tiles and among the last ones, a square is
            // one tile four times.
            let four = if !(0x70..0xf0).contains(&first) {
                [first; 4]
            } else {
                [first, first + 1, first + 2, first + 3]
            };
            let (y, x) = (4 + 2 * r, 4 + 2 * c);
            tiles[y][x] = four[0];
            tiles[y + 1][x] = four[1];
            tiles[y][x + 1] = four[2];
            tiles[y + 1][x + 1] = four[3];
        }
    }
    let look = |door: u8| match door {
        4 if map => BOMBED,
        _ => DOOR_LOOK[usize::from(door)],
    };
    let mut door = |table: usize, door: u8, place: &dyn Fn(usize) -> (usize, usize)| {
        let at = table + 12 * look(door);
        for i in 0..12 {
            let (r, c) = place(i);
            tiles[r][c] = rom[at + i];
        }
    };
    door(DOORS_NORTH, a >> 5, &|i| (1 + i % 3, 14 + i / 3));
    door(DOORS_SOUTH, (a >> 2) & 7, &|i| (18 + i % 3, 14 + i / 3));
    let side = |c0: usize| move |i: usize| (9 + 2 * (i / 6) + i % 2, c0 + (i % 6) / 2);
    door(DOORS_WEST, b >> 5, &side(1));
    door(DOORS_EAST, (b >> 2) & 7, &side(28));
    let mut palettes = [[a & 3; 16]; 11];
    for row in palettes.iter_mut().take(9).skip(2) {
        for p in row.iter_mut().take(14).skip(2) {
            *p = b & 3;
        }
    }
    Room { tiles, palettes }
}

/// Background tile `t` in a dungeon.
fn background_tile(tile: u8) -> usize {
    let t = usize::from(tile);
    match t {
        0..0x70 => 0x878f + 16 * t,
        0x70..0xf2 => 0xc12b + 16 * (t - 0x70),
        _ => 0x8e8f + 16 * (t - 0xf2),
    }
}

/// Sprite tile `t` (the top of an 8 x 16 pair) in `level`.
fn sprite_tile(rom: &[u8], level: usize, tile: u8) -> usize {
    let t = usize::from(tile);
    let pointer = |table: usize| {
        let at = table + 2 * level;
        usize::from(rom[at]) + (usize::from(rom[at + 1]) << 8) + BANK3
    };
    match t {
        0..0x70 => 0x808f + 16 * t,
        0x70..0x8e => 0x4dc4 + 16 * (t - 0x70),
        0x8e..0x9e => 0xdccb + 16 * (t - 0x8e),
        0x9e..0xc0 => pointer(MONSTER_SPRITES) + 16 * (t - 0x9e),
        _ => pointer(BOSS_SPRITES) + 16 * (t - 0xc0),
    }
}

fn color(rom: &[u8], level: usize, sprite: bool, palette: usize, index: usize) -> u32 {
    let base = level_block(level) + if sprite { 16 } else { 0 };
    let at = if index == 0 {
        base
    } else {
        base + 4 * palette + index
    };
    NES[usize::from(rom[at] & 0x3f)]
}

/// Paints room `room` of `level`, 256 x 176 pixels, at `x0`, `y0`.
pub fn paint_room(rom: &[u8], level: usize, room: &Room, image: &mut Image, x0: i32, y0: i32) {
    for r in 0..22 {
        for c in 0..32 {
            let at = background_tile(room.tiles[r][c]);
            let palette = usize::from(room.palettes[r / 2][c / 2]);
            for y in 0..8 {
                for x in 0..8 {
                    let v = tile_pixel(rom, at, x, y);
                    image.put(
                        x0 + (8 * c + x) as i32,
                        y0 + (8 * r + y) as i32,
                        color(rom, level, false, palette, v),
                    );
                }
            }
        }
    }
}

/// One 8 x 16 sprite as the game puts it in OAM: `attr` holds the palette
/// in bits 0-1 and the mirroring in bits 6 and 7.
pub fn draw_sprite(
    rom: &[u8],
    level: usize,
    image: &mut Image,
    tile: u8,
    attr: u8,
    x: i32,
    y: i32,
) {
    let at = sprite_tile(rom, level, tile & 0xfe);
    let palette = usize::from(attr & 3);
    for row in 0..16 {
        let sy = if attr & 0x80 != 0 { 15 - row } else { row };
        let tile_at = at + (sy / 8) * 16;
        for col in 0..8 {
            let sx = if attr & 0x40 != 0 { 7 - col } else { col };
            let v = tile_pixel(rom, tile_at, sx, sy % 8);
            if v != 0 {
                image.put(
                    x + col as i32,
                    y + row as i32,
                    color(rom, level, true, palette, v),
                );
            }
        }
    }
}

/// The text old men say, as the game writes it: bytes in the game's font,
/// bit 7 ending the first line, bit 6 the second, both the text.
pub fn draw_text_bytes(
    rom: &[u8],
    level: usize,
    image: &mut Image,
    bytes: &[u8],
    x0: i32,
    y0: i32,
) {
    let (mut x, mut y) = (x0, y0);
    for &b in bytes {
        // 0x24 and 0x25 are spaces; the game leaves the tile as it is.
        let tile = b & 0x3f;
        let at = background_tile(tile);
        for row in 0..8 {
            if tile == 0x24 || tile == 0x25 {
                break;
            }
            for col in 0..8 {
                let v = tile_pixel(rom, at, col, row);
                if v != 0 {
                    image.put(
                        x + col as i32,
                        y + row as i32,
                        color(rom, level, false, 0, v),
                    );
                }
            }
        }
        x += 8;
        if b & 0xc0 != 0 {
            x = x0;
            y += 8;
        }
    }
}

/// Where the item of a room lies, on the 256 x 176 room: the level's four
/// positions as `0xXY` in 16-pixel steps, chosen by table F bits 4-5.
pub fn item_position(rom: &[u8], level: usize, f: u8) -> (i32, i32) {
    let p = rom[level_block(level) + 0x26 + usize::from((f >> 4) & 3)];
    (i32::from(p & 0xf0), i32::from(p & 0x0f) * 16 - 64)
}

/// How many monsters a room's table C bits 6-7 ask for in `level`.
pub fn monster_count(rom: &[u8], level: usize, c: u8) -> usize {
    usize::from(rom[level_block(level) + 0x21 + usize::from(c >> 6)])
}

/// The monsters of a room with `count` of them: one code `count` times, or
/// the first `count` of a mixed group, read on past its own list as the
/// game does.
pub fn monster_codes(rom: &[u8], code: u8, count: usize) -> Vec<u8> {
    if code < 0x62 {
        return vec![code; count];
    }
    let at = MIXED_GROUPS + 2 * usize::from(code - 0x62);
    let p = usize::from(rom[at]) + (usize::from(rom[at + 1]) << 8) + BANK5;
    rom[p..p + count].to_vec()
}

/// Item id -> sprites as the game draws it in a dungeon: x offset from the
/// item's position, tile, OAM attributes. Measured in a headless emulator.
const ITEM_SPRITES: [&[(i32, u8, u8)]; 32] = [
    &[(4, 0x34, 1)],
    &[(4, 0x20, 0)],
    &[(4, 0x20, 1)],
    &[],
    &[(4, 0x22, 2)],
    &[(4, 0x24, 2)],
    &[(4, 0x26, 1)],
    &[(4, 0x26, 2)],
    &[(4, 0x28, 0)],
    &[(4, 0x28, 1)],
    &[(4, 0x2a, 0)],
    &[(4, 0x2c, 2)],
    &[(0, 0x6c, 0), (8, 0x6c, 0x40)],
    &[(0, 0x76, 0), (8, 0x76, 0x40)],
    &[(0, 0xf2, 2), (8, 0xf4, 2)],
    &[(4, 0x32, 1)],
    &[(4, 0x4a, 1)],
    &[(4, 0x42, 2)],
    &[(4, 0x46, 1)],
    &[(4, 0x46, 2)],
    &[(4, 0x4e, 2)],
    &[(4, 0x4c, 1)],
    &[(0, 0x6a, 2), (7, 0x6a, 0x42)],
    &[(4, 0x4c, 2)],
    &[(4, 0x32, 2)],
    &[(4, 0x2e, 2)],
    &[(0, 0x68, 2), (7, 0x68, 0x42)],
    &[(-8, 0x6e, 2), (0, 0x6e, 0x42)],
    &[(4, 0x56, 0)],
    &[(4, 0x36, 0)],
    &[(4, 0x36, 1)],
    &[(4, 0x40, 1)],
];

/// Draws dungeon item `item` with its position at `x`, `y` on the room.
pub fn draw_item(rom: &[u8], level: usize, image: &mut Image, item: u8, x: i32, y: i32) {
    if let Some(sprites) = ITEM_SPRITES.get(usize::from(item)) {
        for &(dx, tile, attr) in sprites.iter() {
            draw_sprite(rom, level, image, tile, attr, x + dx, y + 1);
        }
    }
}

/// A monster's sprite. Most monsters take their tiles from the object tile
/// table at 0x6E70-0x6F5F, which the generator rewrites when it moves
/// monster graphics between banks: a nonzero `at` names the entry and the
/// tile is that byte plus `tile`; with `at` 0, `tile` is the tile itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Sprite {
    pub x: i32,
    pub y: i32,
    pub at: usize,
    pub tile: u8,
    /// OAM attributes: palette in bits 0-1, mirrored in bit 6, upside down
    /// in bit 7.
    pub attr: u8,
}

const fn s(x: i32, y: i32, at: usize, tile: u8, attr: u8) -> Sprite {
    Sprite {
        x,
        y,
        at,
        tile,
        attr,
    }
}

#[non_exhaustive]
pub enum Monster {
    None,
    /// Drawn in the room at its own place.
    Fixed(&'static [Sprite]),
    /// One of a crowd: sprites relative to it.
    Icon(&'static [Sprite]),
}

/// Draws a monster's sprite with `x`, `y` as its origin.
pub fn draw_monster_sprite(
    rom: &[u8],
    level: usize,
    image: &mut Image,
    sprite: Sprite,
    x: i32,
    y: i32,
) {
    let tile = if sprite.at == 0 {
        sprite.tile
    } else {
        rom[sprite.at].wrapping_add(sprite.tile)
    };
    draw_sprite(
        rom,
        level,
        image,
        tile,
        sprite.attr,
        x + sprite.x,
        y + sprite.y,
    );
}

/// How monster code `code` (0x00-0x61) looks.
pub fn monster(code: u8) -> &'static Monster {
    MONSTERS.get(usize::from(code)).unwrap_or(&Monster::None)
}

/// Monster code 0x00-0x61 -> how the game draws it, measured in a headless
/// emulator: in the room at its own place (people, bosses and other
/// fixed sets, sprites x, y on the 256 x 176 room) or as an icon (sprites
/// relative to one monster). See `Sprite` for the tiles.
const MONSTERS: [Monster; 0x62] = [
    Monster::None,
    Monster::Icon(&[s(-1, 0, 0x6e9f, 0x00, 0x01), s(7, 0, 0x6e9f, 0x02, 0x01)]),
    Monster::Icon(&[s(-1, 0, 0x6ea3, 0x00, 0x02), s(7, 0, 0x6ea3, 0x02, 0x02)]),
    Monster::Icon(&[s(0, 0, 0x6ea6, 0x02, 0x43), s(8, 0, 0x6ea6, 0x00, 0x43)]),
    Monster::Icon(&[s(-1, 0, 0x6ea8, 0x02, 0x42), s(7, 0, 0x6ea8, 0x00, 0x42)]),
    Monster::Icon(&[s(0, 0, 0x6ef2, 0x02, 0x41), s(8, 0, 0x6ef2, 0x00, 0x41)]),
    Monster::Icon(&[s(-1, 0, 0x6ef4, 0x00, 0x02), s(7, 0, 0x6ef4, 0x02, 0x02)]),
    Monster::Icon(&[s(0, 0, 0x6eb1, 0x02, 0x42), s(8, 0, 0x6eb1, 0x00, 0x42)]),
    Monster::Icon(&[s(-1, 0, 0x6eb1, 0x02, 0x42), s(7, 0, 0x6eb1, 0x00, 0x42)]),
    Monster::Icon(&[s(0, 0, 0x6eb7, 0x02, 0x41), s(8, 0, 0x6eb7, 0x00, 0x41)]),
    Monster::Icon(&[s(0, 0, 0x6eb7, 0x02, 0x41), s(8, 0, 0x6eb7, 0x00, 0x41)]),
    Monster::Icon(&[s(-1, 0, 0x6efb, 0x00, 0x02), s(7, 0, 0x6efb, 0x02, 0x02)]),
    Monster::Icon(&[s(0, 0, 0x6f00, 0x02, 0x41), s(8, 0, 0x6f00, 0x00, 0x41)]),
    Monster::Icon(&[s(1, -1, 0x6ebb, 0x00, 0x01), s(9, -1, 0x6ebb, 0x00, 0x41)]),
    Monster::Icon(&[s(1, -2, 0x6ebd, 0x00, 0x02), s(9, -2, 0x6ebd, 0x00, 0x42)]),
    Monster::Icon(&[s(0, 0, 0x6ec0, 0x00, 0x01), s(8, 0, 0x6ec0, 0x00, 0x41)]),
    Monster::Icon(&[s(0, -1, 0x6ecc, 0x00, 0x02), s(8, -1, 0x6ecc, 0x00, 0x42)]),
    Monster::Icon(&[s(0, 0, 0x6ed4, 0x00, 0x03), s(8, 0, 0x6ed4, 0x00, 0x43)]),
    Monster::Icon(&[s(-1, 3, 0x6f04, 0x00, 0x01), s(7, 3, 0x6f04, 0x00, 0x41)]),
    Monster::Icon(&[s(0, 0, 0x6f07, 0x00, 0x03), s(8, 0, 0x6f07, 0x00, 0x43)]),
    Monster::Icon(&[s(4, 0, 0x6f0a, 0x00, 0x03)]),
    Monster::Icon(&[s(3, 0, 0x6f0a, 0x00, 0x03)]),
    Monster::Icon(&[s(0, 0, 0x6f0c, 0x00, 0x00), s(8, 0, 0x6f0c, 0x00, 0x40)]),
    Monster::Icon(&[s(-1, 0, 0x6f0e, 0x00, 0x02), s(7, 0, 0x6f0e, 0x00, 0x42)]),
    Monster::Icon(&[s(0, 0, 0x6f11, 0x00, 0x03), s(8, 0, 0x6f12, 0x00, 0x03)]),
    Monster::None,
    Monster::Icon(&[s(0, 0, 0x6edd, 0x00, 0x02), s(8, 0, 0x6edd, 0x00, 0x42)]),
    Monster::Icon(&[s(0, 0, 0x6f15, 0x00, 0x01), s(8, 0, 0x6f15, 0x00, 0x41)]),
    Monster::Icon(&[s(1, 0, 0x6f17, 0x00, 0x02), s(9, 0, 0x6f17, 0x00, 0x42)]),
    Monster::Icon(&[s(1, 0, 0x6f19, 0x00, 0x03), s(9, 0, 0x6f19, 0x00, 0x43)]),
    Monster::None,
    Monster::Icon(&[
        s(-48, -21, 0x6ee2, 0x00, 0x02),
        s(-40, -21, 0x6ee2, 0x02, 0x02),
        s(-37, -19, 0x6ee2, 0x00, 0x02),
        s(-29, -19, 0x6ee2, 0x02, 0x02),
        s(19, 1, 0x6ee2, 0x00, 0x02),
        s(27, 1, 0x6ee2, 0x02, 0x02),
    ]),
    Monster::Icon(&[s(-1, -1, 0x6ee3, 0x00, 0x02), s(7, -1, 0x6ee3, 0x02, 0x02)]),
    Monster::Icon(&[s(0, 0, 0x6ee5, 0x02, 0x41), s(8, 0, 0x6ee5, 0x00, 0x41)]),
    Monster::Icon(&[s(0, 0, 0x6ee5, 0x00, 0x01), s(8, 0, 0x6ee5, 0x02, 0x01)]),
    Monster::Icon(&[s(0, 0, 0x6f1c, 0x00, 0x01), s(8, 0, 0x6f1c, 0x02, 0x01)]),
    Monster::Icon(&[s(0, 0, 0x6f1f, 0x02, 0x42), s(8, 0, 0x6f1f, 0x00, 0x42)]),
    Monster::None,
    Monster::None,
    Monster::Icon(&[s(0, 1, 0x6f26, 0x02, 0x41), s(8, 1, 0x6f25, 0x00, 0x41)]),
    Monster::Icon(&[s(-1, 0, 0x6f28, 0x00, 0x02), s(7, 0, 0x6f28, 0x02, 0x02)]),
    Monster::None,
    Monster::Icon(&[s(-1, 0, 0x6f2b, 0x02, 0x42), s(7, 0, 0x6f2b, 0x00, 0x42)]),
    Monster::Icon(&[s(-1, 0, 0x6f2c, 0x02, 0x40), s(7, 0, 0x6f2c, 0x00, 0x40)]),
    Monster::Icon(&[s(-1, 0, 0x6f2c, 0x02, 0x41), s(7, 0, 0x6f2c, 0x00, 0x41)]),
    Monster::Icon(&[s(-1, 0, 0x6f2c, 0x02, 0x42), s(7, 0, 0x6f2c, 0x00, 0x42)]),
    Monster::Icon(&[s(-2, 0, 0x6ee6, 0x02, 0x40), s(6, 0, 0x6ee6, 0x00, 0x40)]),
    Monster::Icon(&[s(4, 0, 0, 0x50, 0x02)]),
    Monster::Icon(&[s(-1, 0, 0x6f2d, 0x02, 0x41), s(7, 0, 0x6f2d, 0x00, 0x41)]),
    Monster::Icon(&[s(0, 0, 0x6f30, 0x00, 0x03), s(8, 0, 0x6f30, 0x02, 0x03)]),
    Monster::Fixed(&[
        s(49, 29, 0x6f30, 0x00, 0x03),
        s(57, 29, 0x6f30, 0x02, 0x03),
        s(65, 29, 0x6f31, 0x00, 0x03),
        s(73, 29, 0x6f31, 0x02, 0x03),
    ]),
    Monster::Fixed(&[
        s(86, 48, 0x6f3d, 0x02, 0x41),
        s(94, 48, 0x6f3d, 0x00, 0x41),
        s(100, 48, 0, 0x44, 0x00),
        s(102, 48, 0x6f3b, 0x00, 0x01),
        s(110, 48, 0x6f3b, 0x00, 0x41),
        s(118, 48, 0x6f3c, 0x02, 0x41),
        s(126, 48, 0x6f3c, 0x00, 0x41),
    ]),
    Monster::Fixed(&[
        s(86, 48, 0x6f3d, 0x02, 0x42),
        s(94, 48, 0x6f3d, 0x00, 0x42),
        s(100, 48, 0, 0x44, 0x00),
        s(102, 48, 0x6f3b, 0x00, 0x02),
        s(110, 48, 0x6f3b, 0x00, 0x42),
        s(118, 48, 0x6f3c, 0x02, 0x42),
        s(126, 48, 0x6f3c, 0x00, 0x42),
    ]),
    Monster::Fixed(&[
        s(124, 48, 0, 0x32, 0x02),
        s(116, 64, 0, 0x32, 0x02),
        s(132, 64, 0, 0x32, 0x02),
        s(100, 80, 0, 0x32, 0x02),
        s(116, 80, 0, 0x32, 0x02),
        s(132, 80, 0, 0x32, 0x02),
        s(148, 80, 0, 0x32, 0x02),
        s(116, 96, 0, 0x32, 0x02),
        s(132, 96, 0, 0x32, 0x02),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f3e, 0x00, 0x02),
        s(128, 64, 0x6f3e, 0x02, 0x02),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::Fixed(&[
        s(120, 72, 0x6f3f, 0x00, 0x00),
        s(128, 72, 0x6f3f, 0x00, 0x40),
        s(112, 93, 0, 0x5e, 0x42),
        s(120, 93, 0, 0x5c, 0x42),
        s(128, 93, 0, 0x5e, 0x42),
        s(136, 93, 0, 0x5c, 0x42),
        s(96, 117, 0, 0x5e, 0x42),
        s(104, 117, 0, 0x5c, 0x42),
        s(144, 117, 0, 0x5c, 0x02),
        s(152, 117, 0, 0x5e, 0x02),
    ]),
    Monster::Fixed(&[
        s(84, 44, 0x6f41, 0x00, 0x03),
        s(92, 44, 0x6f41, 0x02, 0x03),
        s(100, 44, 0x6f41, 0x02, 0x43),
        s(108, 44, 0x6f41, 0x00, 0x43),
        s(92, 52, 0x6f11, 0x00, 0x03),
        s(100, 52, 0x6f11, 0x00, 0x43),
        s(84, 60, 0x6f41, 0x00, 0x83),
        s(92, 60, 0x6f41, 0x02, 0x83),
        s(100, 60, 0x6f41, 0x02, 0xc3),
        s(108, 60, 0x6f41, 0x00, 0xc3),
    ]),
    Monster::None,
    Monster::Fixed(&[
        s(68, 61, 0, 0xa0, 0x02),
        s(68, 61, 0, 0xa0, 0x02),
        s(76, 61, 0, 0xa0, 0x02),
        s(76, 61, 0, 0xa0, 0x02),
        s(83, 61, 0, 0x9e, 0x02),
        s(83, 61, 0, 0x9e, 0x02),
        s(68, 69, 0, 0xa0, 0x02),
        s(68, 69, 0, 0xa0, 0x02),
    ]),
    Monster::Fixed(&[
        s(68, 61, 0, 0xa0, 0x01),
        s(68, 61, 0, 0xa0, 0x01),
        s(76, 61, 0, 0xa0, 0x01),
        s(82, 61, 0, 0x9e, 0x01),
        s(68, 69, 0, 0xa0, 0x01),
        s(68, 69, 0, 0xa0, 0x01),
        s(68, 69, 0, 0xa0, 0x01),
        s(68, 75, 0, 0x9e, 0x01),
    ]),
    Monster::Icon(&[s(0, 0, 0x6f46, 0x00, 0x01), s(8, 0, 0x6f46, 0x00, 0x41)]),
    Monster::Fixed(&[
        s(168, 64, 0, 0xcc, 0x03),
        s(176, 64, 0, 0xc4, 0x03),
        s(184, 64, 0, 0xc8, 0x03),
        s(168, 80, 0, 0xc2, 0x03),
        s(176, 80, 0, 0xc6, 0x03),
        s(184, 80, 0, 0xca, 0x03),
        s(127, 87, 0, 0x44, 0x00),
    ]),
    Monster::Fixed(&[
        s(80, 29, 0x6f51, 0x00, 0x03),
        s(88, 29, 0x6f51, 0x02, 0x03),
        s(96, 29, 0x6f53, 0x02, 0x43),
        s(104, 29, 0x6f53, 0x00, 0x43),
        s(80, 45, 0x6f52, 0x00, 0x03),
        s(88, 45, 0x6f52, 0x02, 0x03),
        s(96, 45, 0x6f54, 0x02, 0x43),
        s(104, 45, 0x6f54, 0x00, 0x43),
    ]),
    Monster::Fixed(&[s(80, 29, 0, 0x5e, 0x42), s(88, 29, 0, 0x5c, 0x42)]),
    Monster::Fixed(&[s(80, 29, 0, 0x5e, 0x42), s(88, 29, 0, 0x5c, 0x42)]),
    Monster::Fixed(&[
        s(135, 48, 0, 0x44, 0x02),
        s(128, 55, 0, 0x44, 0x02),
        s(143, 55, 0, 0x44, 0x02),
        s(128, 63, 0, 0x44, 0x02),
        s(144, 63, 0, 0x44, 0x02),
        s(128, 71, 0, 0x44, 0x02),
        s(144, 71, 0, 0x44, 0x02),
        s(128, 79, 0, 0x44, 0x02),
        s(144, 79, 0, 0x44, 0x02),
        s(128, 87, 0, 0x44, 0x02),
    ]),
    Monster::Fixed(&[
        s(116, 23, 0, 0xc0, 0x03),
        s(124, 23, 0, 0xc4, 0x03),
        s(132, 23, 0, 0xc8, 0x03),
        s(116, 39, 0, 0xc2, 0x03),
        s(124, 39, 0, 0xc6, 0x03),
        s(132, 39, 0, 0xca, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(120, 58, 0, 0xda, 0x03),
        s(116, 65, 0, 0xda, 0x03),
        s(118, 70, 0, 0xda, 0x03),
        s(108, 71, 0, 0xdc, 0x03),
        s(112, 71, 0, 0x44, 0x03),
    ]),
    Monster::Fixed(&[
        s(116, 23, 0, 0xc0, 0x03),
        s(124, 23, 0, 0xc4, 0x03),
        s(132, 23, 0, 0xc8, 0x03),
        s(116, 39, 0, 0xc2, 0x03),
        s(124, 39, 0, 0xc6, 0x03),
        s(132, 39, 0, 0xca, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(126, 52, 0, 0xda, 0x03),
        s(126, 57, 0, 0xda, 0x03),
        s(120, 58, 0, 0xda, 0x03),
        s(120, 59, 0, 0xda, 0x03),
        s(130, 59, 0, 0xdc, 0x03),
        s(116, 65, 0, 0xda, 0x03),
        s(118, 70, 0, 0xda, 0x03),
        s(108, 71, 0, 0xdc, 0x03),
        s(112, 71, 0, 0x44, 0x03),
    ]),
    Monster::Fixed(&[
        s(116, 23, 0, 0xc0, 0x03),
        s(124, 23, 0, 0xc4, 0x03),
        s(132, 23, 0, 0xc8, 0x03),
        s(116, 39, 0, 0xc2, 0x03),
        s(124, 39, 0, 0xc6, 0x03),
        s(132, 39, 0, 0xca, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(125, 52, 0, 0xda, 0x03),
        s(125, 52, 0, 0xda, 0x03),
        s(125, 57, 0, 0xda, 0x03),
        s(127, 57, 0, 0xda, 0x03),
        s(120, 58, 0, 0xda, 0x03),
        s(121, 60, 0, 0xda, 0x03),
        s(131, 60, 0, 0xdc, 0x03),
        s(125, 62, 0, 0xda, 0x03),
        s(116, 65, 0, 0xda, 0x03),
        s(124, 67, 0, 0xdc, 0x03),
        s(118, 70, 0, 0xda, 0x03),
        s(108, 71, 0, 0xdc, 0x03),
        s(112, 75, 0, 0x44, 0x00),
    ]),
    Monster::Fixed(&[
        s(116, 23, 0, 0xc0, 0x03),
        s(124, 23, 0, 0xc4, 0x03),
        s(132, 23, 0, 0xc8, 0x03),
        s(116, 39, 0, 0xc2, 0x03),
        s(124, 39, 0, 0xc6, 0x03),
        s(132, 39, 0, 0xca, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 47, 0, 0xda, 0x03),
        s(124, 52, 0, 0xda, 0x03),
        s(125, 52, 0, 0xda, 0x03),
        s(125, 52, 0, 0xda, 0x03),
        s(124, 57, 0, 0xda, 0x03),
        s(125, 57, 0, 0xda, 0x03),
        s(127, 57, 0, 0xda, 0x03),
        s(120, 58, 0, 0xda, 0x03),
        s(121, 60, 0, 0xda, 0x03),
        s(131, 60, 0, 0xdc, 0x03),
        s(124, 62, 0, 0xda, 0x03),
        s(125, 62, 0, 0xda, 0x03),
        s(116, 65, 0, 0xda, 0x03),
        s(124, 67, 0, 0xdc, 0x03),
        s(124, 67, 0, 0xdc, 0x03),
        s(128, 67, 0, 0x44, 0x00),
        s(118, 70, 0, 0xda, 0x03),
        s(108, 71, 0, 0xdc, 0x03),
    ]),
    Monster::Fixed(&[
        s(80, 54, 0x6f5a, 0x00, 0x02),
        s(88, 54, 0x6f5a, 0x00, 0x42),
        s(88, 62, 0, 0x44, 0x00),
    ]),
    Monster::Fixed(&[
        s(125, 6, 0x6f23, 0x00, 0x02),
        s(99, 11, 0x6f24, 0x00, 0x02),
        s(156, 20, 0x6f23, 0x00, 0x02),
        s(101, 33, 0x6f24, 0x00, 0x02),
        s(136, 40, 0x6f5b, 0x02, 0x41),
        s(144, 40, 0x6f5b, 0x00, 0x41),
        s(184, 50, 0x6f24, 0x00, 0x02),
        s(126, 60, 0x6f23, 0x00, 0x02),
        s(184, 72, 0x6f24, 0x00, 0x02),
        s(161, 76, 0x6f23, 0x00, 0x02),
    ]),
    Monster::Fixed(&[
        s(147, 15, 0x6f23, 0x00, 0x02),
        s(128, 17, 0x6f24, 0x00, 0x02),
        s(163, 26, 0x6f24, 0x00, 0x02),
        s(111, 35, 0x6f24, 0x00, 0x02),
        s(136, 40, 0x6f5b, 0x02, 0x41),
        s(144, 40, 0x6f5b, 0x00, 0x41),
        s(167, 47, 0x6f23, 0x00, 0x02),
        s(115, 58, 0x6f23, 0x00, 0x02),
        s(154, 65, 0x6f24, 0x00, 0x02),
        s(132, 69, 0x6f24, 0x00, 0x02),
    ]),
    Monster::Fixed(&[
        s(32, 29, 0x6f5c, 0x00, 0x01),
        s(40, 29, 0x6f5c, 0x00, 0x41),
        s(208, 29, 0x6f5c, 0x00, 0x01),
        s(216, 29, 0x6f5c, 0x00, 0x41),
        s(64, 77, 0x6f5c, 0x00, 0x01),
        s(72, 77, 0x6f5c, 0x00, 0x41),
        s(176, 77, 0x6f5c, 0x00, 0x01),
        s(184, 77, 0x6f5c, 0x00, 0x41),
        s(110, 125, 0x6f5c, 0x00, 0x01),
        s(118, 125, 0x6f5c, 0x00, 0x41),
        s(130, 125, 0x6f5c, 0x00, 0x01),
        s(138, 125, 0x6f5c, 0x00, 0x41),
    ]),
    Monster::None,
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
        s(124, 88, 0, 0x32, 0x02),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
        s(88, 88, 0, 0x68, 0x02),
        s(95, 88, 0, 0x68, 0x42),
        s(156, 88, 0, 0x32, 0x02),
    ]),
    Monster::Fixed(&[
        s(72, 64, 0, 0x5e, 0x42),
        s(80, 64, 0, 0x5c, 0x42),
        s(120, 64, 0x6f5d, 0x00, 0x02),
        s(128, 64, 0x6f5d, 0x00, 0x42),
        s(168, 64, 0, 0x5e, 0x42),
        s(176, 64, 0, 0x5c, 0x42),
    ]),
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::None,
    Monster::Fixed(&[s(84, 29, 0, 0x34, 0x01)]),
    Monster::None,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrored_wall_tiles_follow_the_game() {
        assert_eq!(mirror(0xc0, true, false), 0xc3);
        assert_eq!(mirror(0xc0, false, true), 0xc1);
        assert_eq!(mirror(0xc0, true, true), 0xc2);
        assert_eq!(mirror(0xe0, true, true), 0xe0);
        assert_eq!(mirror(0xf5, true, false), 0xf5);
    }
}
