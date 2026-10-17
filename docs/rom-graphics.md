# ROM graphics

Where the game keeps what it draws, for rendering maps, tooltips and the
like straight from a ROM. All addresses are file offsets in
"Legend of Zelda, The (USA) (Rev 1)" including the 16-byte iNES header.
Everything below was measured with the emulator scripts in
[tools/emu](../tools/emu), not taken from guesses; `node verify.js` checks
the overworld part against the running game for all 128 screens.

## Colors

| What | Offset | Contents |
|---|---|---|
| Overworld background palettes | `0x19313` | 4 x 4 NES colors: status bar, red, green, brown |
| Overworld sprite palettes | `0x19323` | 4 x 4 NES colors; 0 is Link's, 1 blue/white, 2 red/white, 3 changes with the enemies |
| File select sprite palettes | `0x19cfb` | used by the hash on the file select screen |

The status bar's hearts use background palette 1 (`0F 16 27 36`): full
hearts in color `16`, lost hearts in `36`.

## Background tiles

Background tile `t` (pattern table `$1000`) comes from three places:

| Tiles | Offset of tile `t` |
|---|---|
| `00`-`6F` font and common | `0x878F + 16 * t` |
| `70`-`F1` overworld | `0xC94B + 16 * (t - 0x70)` |
| `F2`-`FF` status bar | `0x8E8F + 16 * (t - 0xF2)` |

The font follows the game's text encoding: `0`-`9` digits, `10`-`35`
letters, `36` space, `40` `,`, `41` `!`, `42` `'`, `43` `&`, `44` `.`,
`45` `"`, `46` `?`, `47` `-`. Tile `F2` is the status bar's heart.

## Sprite tiles

Sprites are 8 x 16; an even tile `t` names the pair `t`, `t + 1` from
pattern table `$0000`:

| Tiles | Offset of tile `t` |
|---|---|
| `00`-`6F` Link, items, common | `0x808F + 16 * t` |
| `70`-`8D` overworld extras (ladder, ...) | `0x4DC4 + 16 * (t - 0x70)` |
| `8E`-`FE` overworld enemies and people | `0xD16B + 16 * (t - 0x8E)`, except `9C`, `9E`, `D8`, `DA` |

People in caves: the old man is tile `98` mirrored into a 16 x 16 sprite,
palette 2; the cave fires are `5C` and `5E` mirrored, palette 2.

### Items

How the game draws each item id in a cave (x offset from the item's
position, tile, palette, mirrored), measured with `node items.js`:

| Id | Item | Sprites |
|---|---|---|
| 0 | Bombs | `34` p1 |
| 1 | Wood sword | `20` p0 |
| 2 | White sword | `20` p1 |
| 3 | Magical sword | `48` p2 |
| 4 | Bait | `22` p2 |
| 5 | Recorder | `24` p2 |
| 6 | Blue candle | `26` p1 |
| 7 | Red candle | `26` p2 |
| 8 | Wood arrow | `28` p0 |
| 9 | Silver arrow | `28` p1 |
| 10 | Bow | `2A` p0 |
| 11 | Magical key | `2C` p2 |
| 12 | Raft | -4: `6C` p0, +4: `6C` p0 mirrored |
| 13 | Ladder | -4: `76` p0, +4: `76` p0 mirrored |
| 15 | 5 rupees | `32` p1 |
| 16 | Wand | `4A` p1 |
| 17 | Book | `42` p2 |
| 18 | Blue ring | `46` p1 |
| 19 | Red ring | `46` p2 |
| 20 | Power bracelet | `4E` p2 |
| 21 | Letter | `4C` p1 |
| 22 | Compass | -4: `6A` p2, +3: `6A` p2 mirrored |
| 23 | Map | `4C` p2 |
| 24 | Rupee | `32` p1 |
| 25 | Key | `2E` p2 |
| 26 | Heart container | -4: `68` p2, +3: `68` p2 mirrored |
| 27 | Triforce | -4: `6E` p1, +4: `6E` p1 mirrored |
| 28 | Magical shield | `56` p0 |
| 29 | Boomerang | `36` p0 |
| 30 | Magical boomerang | `36` p1 |
| 31 | Blue potion | `40` p1 |
| 32 | Red potion | `40` p2 |
| 33 | Clock | -4: `66` p2, +3: `66` p2 mirrored |
| 34 | Heart | `F3` (status bar heart) p1 |
| 35 | Fairy | `50` p2 |

## Overworld screens

A screen is 16 columns x 11 squares of 16 x 16 pixels; a square is 2 x 2
tiles.

| Table | Offset | Use |
|---|---|---|
| Table 0 | `0x18410` + screen | bits 0-1: palette of the outer two squares; bits 4-7: column of the cave entrance |
| Table 1 | `0x18490` + screen | bits 0-1: palette inside; bits 2-7: cave id |
| Table 3 | `0x18590` + screen | bits 0-6: screen layout |
| Layouts | `0x15428` + 16 * layout | 16 column ids |
| Column directory | `0x19D1F` | 16 pointers (bank 5, `$8000` = `0x14010`) into the column heap |
| Primary squares | `0x1698C` + square | first tile of a square |
| Secondary squares | `0x16986` | first tile for primary values `E5`-`EA` |
| Four-tile squares | `0x169C4` + 4 * square | squares `00`-`0F`, four tiles each |

**Columns.** The high nibble of a column id picks a directory entry; from
there the low nibble counts bytes with bit 7 set to find the column's
start. A column holds 11 squares: each byte gives one square in bits 0-5,
and bit 6 repeats it once.

**Squares.** Squares `00`-`0F` take their four tiles from the four-tile
table. Every other square takes the first tile `p` from the primary table
(through the secondary table when `p` is `E5`-`EA`) and uses `p`, `p + 1`,
`p + 2`, `p + 3`. Tiles are listed top left, bottom left, top right,
bottom right.

**Palettes.** Squares in rows 2-8 and columns 2-13 use table 1's palette,
the outer ring table 0's.

**Cave doors.** The visible entrance is square `0C` (tiles `F3 24 F3 24`)
in the column table 0 names. Hidden entrances sit on squares of their own,
measured with `node secret.js`, which marks every secret as found (RAM
`$067F` + screen, bit 7) and compares the screen with its layout:

| Square | Hides | Opened by |
|---|---|---|
| `27` | wall, turns into a door | bomb |
| `28` | bush, turns into stairs | candle |
| `26` | rock, stairs under it | pushing |
| `29` | grave, stairs under it | pushing |
| `0F` | foot of the waterfall | walking in |
| `05` | the lake of level 7 | recorder |

Stairs under an armos are listed at `0x10CC2` (seven screens, the first
one holds the armos item) with their x positions at `0x10CC9`.

While the game runs, the overworld tables from `0x18410` on sit in
battery RAM from `$687E`; the cave item table `0x18610` is at `$6A7E`.

## RAM

| Address | Use |
|---|---|
| `$EB` | current overworld screen |
| `$70`, `$84` | Link's x and y |
| `$067F` + screen | overworld screen flags; bit 7: secret found |

## Dungeon rooms

A room is drawn like an overworld screen, 32 x 22 tiles, but from its own
tables; `node verify-dungeons.js` checks `z1r::gfx::underworld` against the
running game for every room of the nine levels. It enters a level with the
start room, which the game copies to RAM at `$6BAD`, changed to the room
wanted, so the game itself loads and draws it.

| Table | Offset | Use |
|---|---|---|
| Room tables A-F | `0x18710` + 0x80 * table + room | per level group: `+0x300` for levels 7-9, `+0x600` for the second quest |
| A | | bits 5-7: north door, bits 2-4: south door, bits 0-1: palette of the walls |
| B | | bits 5-7: west door, bits 2-4: east door, bits 0-1: palette inside |
| C | | bits 0-5: monster code (with D bit 7 as bit 6), bits 6-7: which monster count |
| D | | bits 0-5: room layout; `3E`/`3F` are the basements staircases lead to |
| E | | bits 0-4: item (`3` none) |
| F | | bits 4-5: which item position |
| Layouts | `0x160EE` + 12 * layout | 12 column ids |
| Column directory | `0x16714` | 10 pointers (bank 5) into the column heap |
| Squares | `0x16728` + square | first tile of each square |
| Walls | `0x15FB0` / `0x15FCE` | left wall, 3 columns of 10 tiles; top wall, 12 columns of 3 tiles 4 bytes apart |
| Doors | `0x15FFE` east, `0x1603A` west, `0x16076` south, `0x160B2` north | 5 looks x 12 tiles: open, locked, shutter, wall, bombed |
| Level blocks | `0x19313` + 0xFC * level | background then sprite palettes; `+0x21` monster counts; `+0x26` four item positions; `+0x2C` start room |
| Monster sprites | `0xC010` + 2 * level | bank 3 pointer to the sprites loaded at tile `9E` |
| Boss sprites | `0xC024` + 2 * level | bank 3 pointer to the sprites loaded at tile `C0` |
| Mixed monster groups | `0x1474F` + 2 * (code - 0x62) | pointer (bank 5) to the monster codes; a room takes as many as its count, reading on past the group's own list |
| Object tiles | `0x6E70`-`0x6F5F` | first tiles of the monsters' frames, copied to RAM at `$75F0`; the generator rewrites them when it moves monster graphics |

**Columns.** As on the overworld, the high nibble of a column id picks a
directory entry and the low nibble counts column starts (bit 7). A column
is 7 squares; a byte gives a square in bits 0-3 and repeats it as often as
bits 4-6 say, plus once.

**Squares.** A square's first tile `p` gives `p`, `p + 1`, `p + 2`, `p + 3`,
except for tiles below `70` and from `F0` on (sand, water, black), which
fill all four.

**Walls.** The stored quarter is the top left. The others use mirrored
tiles that the game keeps next to each other: for tiles `B8`-`D8` in steps
of 4, `+3` mirrors horizontally, `+1` vertically, `+2` both; `DC` has its
vertical mirror at `DD`, `DE` its horizontal one at `DF`, and `E0` flips
to `E1` when mirrored one way only.

**Doors.** Door types 0-7 look open, wall, wall, wall, wall (bomb; blown
open it is the fifth look), locked, locked and shutter. Each look is 12
tiles: 3 columns of 4 for the side doors, 4 columns of 3 for the others.

**Tiles.** Background tiles `70`-`F1` come from `0xC12B + 16 * (t - 0x70)`,
the rest as on the overworld. Sprite tiles `8E`-`9D` come from `0xDCCB`,
`9E`-`BF` and `C0`-`FF` through the level's two pointers.

**Items and monsters.** An item lies at its level's position `0xXY`
(16-pixel steps, including the status bar's 64 pixels). How each item and
monster is drawn was measured with the emulator, as for the overworld
items; a monster's tiles are kept as the object tile entries the game
read while drawing it, so they follow the generator's changes; old men's words are written into the room's rows 5 and 6 from
column 4.
