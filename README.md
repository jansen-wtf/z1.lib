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

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#whats-in-it">What's in it</a> ·
  <a href="#examples">Examples</a> ·
  <a href="#ground-rules">Ground rules</a> ·
  <a href="#development">Development</a>
</p>

> [!TIP]
> **The ROM is the source.** Everything this library knows is a fact about
> the ROM, and everything it draws is read from the ROM you pass in:
> - Pictures in the game's own graphics, original or randomized ROM alike.
> - Every address measured in a headless emulator, none guessed.
> - Both US releases, PRG0 and Rev 1.

<table>
  <tr>
    <td valign="top" width="33%">
      <p align="center"><b>Read the ROM</b></p>
      <ul><li>Room tables of overworld and levels</li><li>Caves, coast, armos, secrets</li><li>What lies in each level</li><li>Level 9 gate, start items</li></ul>
    </td>
    <td valign="top" width="33%">
      <p align="center"><b>Draw from it</b></p>
      <ul><li>Overworld screens</li><li>Dungeon rooms with doors, items, monsters</li><li>Link, hearts, the game's font</li><li>PNG output</li></ul>
    </td>
    <td valign="top" width="33%">
      <p align="center"><b>Speak Z1R</b></p>
      <ul><li>The flag string, read and written</li><li>27 lists, 94 check boxes</li><li>The randomizer's code changes</li><li>Captions as the form shows them</li></ul>
    </td>
  </tr>
</table>

> [!NOTE]
> **No game data inside.** No ROM, no graphics, no sprites are part of
> this crate; bring your own *Legend of Zelda* (USA) ROM.

## Contents

- [Quick start](#quick-start)
- [What's in it](#whats-in-it)
- [Examples](#examples)
- [Ground rules](#ground-rules)
- [Development](#development)
- [Used by](#used-by)
- [License](#license)

## Quick start

1. **Add it** to `Cargo.toml`, pinned to a tag:

   ```toml
   [dependencies]
   z1 = { git = "https://github.com/jansen-wtf/z1.lib", tag = "1.1.0" }
   ```

2. **Check the ROM once:** `z1::rom::Bytes::new(&data)` takes any Zelda
   ROM, original or randomized, of the right size with an iNES header.
3. **Read or draw:** every function takes those `Bytes`; see the
   [examples](#examples) and `cargo doc --open`.

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

- **Where things are, not what to show:** the library tells apart what the
  game keeps from what a program makes of it.
- **File offsets** throughout, including the 16-byte iNES header.
- **[docs/rom-layout.md](docs/rom-layout.md):** where the game keeps its
  tables, levels and overworld, what z1r's changes and flags mean.
- **[docs/rom-graphics.md](docs/rom-graphics.md):** where it keeps what it
  draws, and how that was measured.

## Examples

Draw an overworld screen from a ROM, original or randomized:

```rust no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read("zelda.nes")?;
    let rom = z1::rom::Bytes::new(&data)?;
    let mut screen = z1::gfx::Image::new(256, 176, 0);
    z1::gfx::paint_screen(rom, 119, &mut screen, 0, 0);
    std::fs::write("screen.png", screen.to_png())?;
    Ok(())
}
```

List what lies in level 1 of the first quest:

```rust no_run
use z1::level;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read("zelda.nes")?;
    let rom = z1::rom::Bytes::new(&data)?;
    let tables = level::tables(rom, 1, false);
    let rooms = level::rooms(tables, level::block(rom, 1), &Default::default());
    for placed in level::items(tables, &rooms) {
        let stairs = if placed.stairs { ", down the stairs" } else { "" };
        println!("item {} in room {:#04x}{stairs}", placed.item, placed.room);
    }
    Ok(())
}
```

Read a randomizer seed's flags:

```rust
use z1::flags::{Check, Flags, check, combo};

let flags = Flags::parse("11fvaXsSNVMgaizTfEEM6gt!QwVfNfjfELUj7RA");
let second_quest = flags.combo(combo::QUEST_NUMBER) == 1;
let race_rom = flags.check(check::TOURNEY_MODE) == Check::On;
assert!(second_quest && !race_rom);
```

- **`z1::rom::Bytes`** is what every reader takes: any Zelda ROM, checked
  once for size and iNES header, so no read runs past its end.
- **`z1::rom::Rom::load`** accepts only an unmodified ROM, and knows its
  release.

## Ground rules

- **No game data in the repository.** Offsets, tile numbers and sprite
  layouts are facts about the ROM; the graphics are read from the user's
  ROM at runtime.
- **Measured, not guessed.** Every address and layout was measured in a
  headless emulator and checked against the running game.
- **Both US releases.** What is here holds for PRG0 and Rev 1 alike.

## Development

```sh
cargo test                     # with the README's examples
Z1R_ROM=zelda.nes cargo test -- --include-ignored   # and the checks against the original ROM
cargo clippy --all-targets
```

- **Documentation:** every public item has its own (`cargo doc --open`); a
  missing one fails the CI.
- **ROM tests** show as ignored without `Z1R_ROM`.
- **[AGENTS.md](AGENTS.md)** maps the code and its rules for anyone changing
  it, coding agents included.

## Used by

- [z1r-tracker](https://github.com/jansen-wtf/z1r-tracker), an autotracker
  for the Zelda Randomizer

## License

- [AGPL-3.0-or-later](LICENSE).
- Whoever builds on this library and passes the result on, as a program or
  as a service on the web, passes on its source under the same license.
