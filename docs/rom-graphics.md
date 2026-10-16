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
