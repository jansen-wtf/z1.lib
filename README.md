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
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-d82800?labelColor=2b2b2b" alt="license: MIT"></a>
</p>

## What's in it

| Module | What |
|---|---|
| `z1::gfx` | pictures from the ROM: overworld screens, dungeon rooms with walls, doors, items and monsters, Link, hearts, text in the game's font, PNG output |
| `z1::rom` | the ROM's layout: size, header, the two US releases (PRG0, Rev 1) by SHA-1, the word before a level's digit (`LEVEL-`, `BOARD-`, …) |
| `z1::flags` | the Zelda Randomizer's flag string: all 27 drop-down lists and 94 check boxes, read and written |

All offsets are file offsets including the 16-byte iNES header.
[docs/rom-graphics.md](docs/rom-graphics.md) says where the game keeps what
it draws, and how that was measured.

## Using it

Add it to `Cargo.toml`, pinned to a tag:

```toml
[dependencies]
z1 = { git = "https://github.com/jansen-wtf/z1.lib", tag = "v0.1.0" }
```

Draw an overworld screen from a ROM, original or randomized:

```rust
let rom = std::fs::read("zelda.nes")?;
let screen = z1::gfx::picture(&rom, "screen:119").unwrap();
std::fs::write("screen.png", screen.to_png())?;
```

Read a randomizer seed's flags:

```rust
use z1::flags::{Flags, check, combo};

let flags = Flags::parse("11fvaXsSNVMgaizTfEEM6gt!QwVfNfjfELUj7RA");
let second_quest = flags.combo(combo::QUEST_NUMBER) == 1;
let race_rom = flags.check(check::TOURNEY_MODE);
```

`z1::rom::Rom::load` accepts only an unmodified ROM; the drawing and
reading functions take the bytes of any ROM.

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

MIT
