# ROM layout

What `z1::tables`, `z1::level`, `z1::overworld`, `z1::z1r` and `z1::flags`
read, besides the graphics in [rom-graphics.md](rom-graphics.md). Offsets
are file offsets including the 16-byte iNES header.

## Room tables

Six tables of `0x80` bytes, A to F, one byte per screen or room in each:
the overworld's at `0x18410`, the levels' from `0x18710`, `0x300` apart
(levels 1-6, levels 7-9, then the same of the second quest). The game
copies the set of the place Link is in to battery RAM at `$687E`, the
current level's block after it at `$6B81`. Tourney seeds fill the ROM's
copy (`0x18410`-`0x1930F`) with noise, so for those only the copy in RAM
holds.

| Table | Levels | Overworld |
|---|---|---|
| A | north door in bits 5-7, south door in bits 2-4; under a room: the room of one staircase | |
| B | west door in bits 5-7, east door in bits 2-4; under a room: the room of the other staircase | the screen's cave in bits 2-7 |
| D | the room's layout in bits 0-5: `3E` a cellar with an item, `3F` a passage | |
| E | the room's item in bits 0-4, `03` for none | from `0x200`: per cave from cave 16 on, three items (`3F` for none), then from `0x23C` their prices |
| F | | bit 7 set: the screen's cave does not count in this quest |

Door types: `1` a wall, `4` a wall to bomb. Under a room, both staircases
in the same room make a cellar with an item; two rooms make a passage.

## Levels

| Offset | Contents |
|---|---|
| `0x19313` + `0xFC` * place | the block of the overworld (place 0) or of level 1-9 |
| block + `0x2A` | the column the pause screen map starts at |
| block + `0x2C` | the start room |
| block + `0x3C` | the pause screen map: 16 columns, row 0 in bit 7 |

The map names a level's rooms, but the original leaves a room out here and
there (level 7's staircase to the red candle, three rooms of level 9).
`level::rooms` takes the map's rooms and adds what can be walked to through
doors and passages from them, without entering a room another level's map
claims. On the original ROM this gives each level its known items; the
tests check that when `Z1R_ROM` names one.

## Overworld

| Offset | Contents |
|---|---|
| `0x178AA` | the coast screen, the operand of `CPX #$5F`; `50` on a mirrored overworld |
| `0x1789A` | the coast item, the operand of `LDA #$1A` |
| `0x10CC2` | the screens of the armos statues that hide stairs, the item's first |
| `0x10CC9` | their x positions |
| `0x10D05` | the armos item |
| `0x490D` | the hearts the white sword cave asks for, as `(hearts - 1) * 16`: `40` (5) in the original |
| `0x4916` | the same for the magical sword cave: `B0` (12) |

- On the second quest's overworld, screens 11 and 66 have no cave, though
  the tables list one. Mirrored, those are screens 4 and 77.
- Mirroring is the only change that writes `0x178AA` (`xor 15`), so `50`
  there tells a mirrored overworld (`overworld::is_mirrored`); checked on
  mirrored and plain z1r seeds of both quests.

Cave ids (table B, bits 2-7; `overworld::cave_kind`):

| Id | Cave | Id | Cave |
|---|---|---|---|
| 1-9 | level entrance | 23 | door repair |
| 16 | wood sword | 24 | letter |
| 17 | take any one | 26 | potion shop |
| 18 | white sword | 27, 28 | pay for a hint (waterfall, lost woods) |
| 19 | magical sword | 29-32 | shops 1-4 |
| 20 | take any road (warp) | 33-35 | secret rupees |
| 21, 25 | hint | 22 | money making game |

## Text

| Offset | Contents |
|---|---|
| `0x4010` | the text table: 64 pointers, two bytes each, little-endian; a pointer minus `0x3FF0` is the text's file offset |
| `0x45B2` | per cave from 16 on, twice the number of the text its person says, in bits 0-5 |
| `0x494B` | the pay-to-talk caves' texts per price, twice the number each: cave 27 the first three, cave 28 the next three |
| `0x4A17` | per level, two bytes; the first is the low byte of the code picking the old men's texts: `23` uses the table at `0x4A2B`, `69` at `0x4A71`, `84` at `0x4A90`, eight entries for room kinds 11-18 |

- A text is font tiles, one byte each: the tile in bits 0-5, bit 7 ending
  the first line, bit 6 the second, both the text (`text::decode`).
- Tiles: 0-9 digits, 10-35 letters, 36 and 37 blanks, 40-47 `,!'&."?-`.
- Checked on both releases: the wood sword cave says "IT'S DANGEROUS TO
  GO / ALONE! TAKE THIS.", an old man of level 1 tells of the eastmost
  peninsula.

## What the flags fix: item places

How many heart containers and collected items (`z1r::PLACED_ITEMS`) each
level holds on its floor and down its staircases follows from the flags
alone (`z1r::item_places`), measured with z1r.rs over 12 seeds of every
combination of dungeon quest, room shuffle, item shuffle and minor drops:

| Level | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| First quest and shapes: floor / stairs | 2 / 1 | 2 / 0 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 2 | 0 / 2 |
| Second quest | 2 / 0 | 2 / 0 | 1 / 1 | 1 / 1 | 1 / 2 | 1 / 1 | 1 / 2 | 1 / 1 | 0 / 2 |

Mixed quests, and minor drops joining an item shuffle, leave the numbers
to the seed.

## The Zelda Randomizer's changes

| Offset | Contents |
|---|---|
| `0x4AB1` | level 9's gate, `LDA $0671; CMP #$FF` in the original |
| `0x5F10` | z1r's gate check, when the gate jumps to `$9F00` |
| `0x9F4E` | `JMP $A830` when z1r sets a starting inventory |
| `0xA850` | the starting inventory, 40 bytes copied to `$0657` |
| `0x17614` | `JMP $B8A0` when the book is an atlas (`39 BE E6` in the original) |

| Bytes at `0x4AB1` | Gate |
|---|---|
| `AD 71 06 C9 FF` | all eight triforce pieces |
| `AD xx 06` | the item at `$06xx` |
| `20 00 9F` | z1r's check at `0x5F10`: `A5 ...` counts pieces (count at `0x5F27`), `AD ...` wants the pieces in the mask at `0x5F14` |
| `.. .. .. EA EA EA EA` | open |

## Flags

z1r names its files `<name>_<seed>_<flags>.nes`. The flag string is one
number in base 63 (`0-9`, `A-Z`, `a-z`, `!`) that packs, from the lowest
digit up, 27 drop-down lists, each in its own radix, then 94 three-state
check boxes (off, on, random) in base 3. To get at a check box, every list
and check box before it is divided out.

Hide dungeon numbers, check box 23, writes `?` (tile 46) over each level's
digit in its level block. Hidden numbers, check box 82, is another option
and leaves the level digits alone.
