<h1 align="center">z1.lib</h1>

<h3 align="center">What The Legend of Zelda keeps in its ROM, as a Rust library.</h3>

<p align="center">
  Draws overworld screens, dungeon rooms, items and text straight from a ROM,
  knows the ROM's layout and reads the Zelda Randomizer's flag string.<br>
  <b>No game data inside</b>: everything comes from the ROM you provide.
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/pure-Rust-f8b800?logo=rust&logoColor=white&labelColor=2b2b2b" alt="pure Rust"></a>
  <picture><img src="https://img.shields.io/badge/ROM-PRG0_%C2%B7_PRG1%2FRev1-0058f8?labelColor=2b2b2b" alt="ROM: PRG0 or PRG1/Rev1"></picture>
  <a href="docs/rom-graphics.md"><img src="https://img.shields.io/badge/graphics-measured,_not_guessed-00a800?labelColor=2b2b2b" alt="graphics measured, not guessed"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0-d82800?labelColor=2b2b2b" alt="license: AGPL-3.0"></a>
</p>

## What's in it

| Module | What |
|---|---|
| `z1::rom` | the ROM: `Bytes` for any Zelda ROM, the two US releases (PRG0, Rev 1) by SHA-1, the word before a level's digit (`LEVEL-`, `BOARD-`, …) |
| `z1::tables` | the six room tables of the overworld and of the levels, in the ROM or copied to RAM: a room's item, layout, doors and staircases, a screen's cave and what it offers |
| `z1::level` | level blocks, start rooms, the shape on a level's map, the rooms that belong to a level, and what lies in them |
| `z1::overworld` | caves per screen, the coast and the armos statues, the screens that hide something, the heart containers |
| `z1::item` | item ids |
| `z1::gfx` | pictures from the ROM: overworld screens, dungeon rooms with walls, doors, items and monsters, Link, hearts, text in the game's font, PNG output |
| `z1::flags` | the Zelda Randomizer's flag string: all 27 drop-down lists and 94 check boxes, read and written |
| `z1::z1r` | what the Zelda Randomizer's changes to the code mean: what opens level 9, the starting inventory, the book as an atlas |

What the game keeps is told apart from what a program makes of it: the
library says where things are, not what a player may see.

All offsets are file offsets including the 16-byte iNES header.
[docs/rom-layout.md](docs/rom-layout.md) says where the game keeps its
tables, levels and overworld, and what z1r's changes and flags mean;
[docs/rom-graphics.md](docs/rom-graphics.md) where it keeps what it draws,
and how that was measured.

## Using it

Add it to `Cargo.toml`, pinned to a tag:

```toml
[dependencies]
z1 = { git = "https://github.com/jansen-wtf/z1.lib", tag = "0.11.0" }
```

Draw an overworld screen from a ROM, original or randomized:

```rust
let data = std::fs::read("zelda.nes")?;
let rom = z1::rom::Bytes::new(&data)?;
let mut screen = z1::gfx::Image::new(256, 176, 0);
z1::gfx::paint_screen(rom, 119, &mut screen, 0, 0);
std::fs::write("screen.png", screen.to_png())?;
```

List what lies in level 1 of the first quest:

```rust
use z1::level;

let tables = level::tables(rom, 1, false);
let rooms = level::rooms(tables, level::block(rom, 1), &Default::default());
for placed in level::items(tables, &rooms) {
    println!("item {} in room {:#04x}{}", placed.item, placed.room,
        if placed.stairs { ", down the stairs" } else { "" });
}
```

Read a randomizer seed's flags:

```rust
use z1::flags::{Flags, check, combo};

let flags = Flags::parse("11fvaXsSNVMgaizTfEEM6gt!QwVfNfjfELUj7RA");
let second_quest = flags.combo(combo::QUEST_NUMBER) == 1;
let race_rom = flags.check(check::TOURNEY_MODE);
```

Every function that reads the ROM takes `z1::rom::Bytes`: any Zelda ROM,
original or randomized, checked once for its size and iNES header, so no
read can run past its end. `z1::rom::Rom::load` accepts only an unmodified
ROM.

## Ground rules

- **No game data in the repository.** Tables of offsets, tile numbers and
  sprite layouts are facts about the ROM; the graphics themselves are read
  from the user's ROM at runtime.
- **Measured, not guessed.** Every address and layout was measured in a
  headless emulator and checked against the running game.
- **Both US releases.** What is here holds for PRG0 and Rev 1 alike.

## Development

```sh
cargo test
cargo clippy --all-targets
```

## Used by

- [z1r-tracker](https://github.com/jansen-wtf/z1r-tracker), an autotracker
  for the Zelda Randomizer

## License

[AGPL-3.0-or-later](LICENSE). Whoever builds on this library and passes
the result on, as a program or as a service on the web, passes on its source
under the same license.
