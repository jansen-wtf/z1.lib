# Working on z1.lib

The `z1` crate: what The Legend of Zelda (NES) keeps in its ROM, read and
drawn from a ROM the user provides, and what the Zelda Randomizer's flag
string and code changes mean. It holds no game data. Two programs build on
it: a port of the Zelda Randomizer that must stay byte-identical to the
original, and an autotracker. The README is for users of the crate; this
file is the map and the rules for changing it.

## The rules

- **No game data in the repository.** Offsets, tile numbers and sprite
  layouts are facts about the ROM; graphics are read from the ROM passed
  in at run time. No ROM, no picture made from one, is committed.
- **Measured, not guessed.** An address or layout goes in only after it
  was measured in a headless emulator and checked against the running
  game; [docs/](docs/) says how for each.
- **Both US releases.** What is here holds for PRG0 and Rev 1 alike;
  where they differ, `rom::Revision` tells them apart.
- **No panic on checked bytes.** Every reader takes `rom::Bytes`, checked
  once for size and iNES header. With any such bytes, a damaged ROM
  included, no function may read past the end or panic; loops that search
  the ROM stop at its end. Panics are allowed only for arguments out of
  their documented range, and every such function says so under
  `# Panics`.
- **Every public item is documented.** `#![warn(missing_docs)]` with
  `-D warnings` in CI makes a missing doc a failure.
- **The library says where things are, not what to show.** Decisions such
  as what a tracker may reveal belong in the program using it.

## Repository map

| Path | What |
|---|---|
| `src/rom.rs` | `Bytes` (checked view of any Zelda ROM), `Rom` (an unmodified one, by SHA-1, with its `Revision`), the level word |
| `src/tables.rs` | the six room tables A-F, in the ROM or copied to battery RAM |
| `src/level.rs` | level blocks, start rooms, map shape, the rooms of a level, what lies in them |
| `src/overworld.rs` | caves per screen, coast, armos, secrets, heart containers |
| `src/item.rs` | item ids |
| `src/z1r.rs` | the randomizer's code changes: level 9 gate, start inventory, book as atlas |
| `src/flags.rs` | the flag string: base-63 codec, `combo` (27 lists) and `check` (94 boxes) indexes with the form's captions |
| `src/gfx/` | pictures: `mod.rs` (palettes, font, items, Link, hearts), `overworld.rs` (screens), `underworld.rs` (rooms, doors, monsters), `png.rs` |
| `docs/rom-layout.md` | where the tables, levels and overworld lie, and what the randomizer's changes and flags mean |
| `docs/rom-graphics.md` | where the graphics lie, and how each was measured |

All offsets are file offsets including the 16-byte iNES header.

## Checking a change

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test                                            # the README's examples run as doctests
Z1R_ROM=zelda.nes cargo test -- --include-ignored     # checks against the original ROM
```

Tests that need a ROM are `#[ignore = "needs the original ROM in Z1R_ROM"]`
and read it with `expect`, so a run without one shows them as ignored and
a run that asks for them fails loudly without it. A test that must hold
without a ROM builds one: zeros of `rom::FILE_LEN` with the iNES header.

A change that the programs built on this crate see is checked in them too,
before a tag: point their `Cargo.toml` at this checkout with a
`[patch."https://github.com/jansen-wtf/z1.lib"]` entry, `path = "../z1.lib"`,
run their tests, and never commit that entry. The randomizer's golden test,
with both ROM releases, is what proves a change to `gfx`, `flags` or `rom`
left its games and pictures as they were.

CI (`.github/workflows/ci.yml`) runs fmt, clippy, Rust 1.88 and the tests on
every push.

## Releases

- Semantic versioning: the programs pin a tag, so a change to a public
  signature is a new major version; additions are minor, fixes patch.
- Tags without "v" (`1.1.0`); the commit is "Version x.y.z" and sets the
  version in `Cargo.toml` and the tag in the README's `Cargo.toml` example.
- After the tag is pushed, the programs move to it in their own commits.

## Conventions

- Everything in the repository is English. Comments only where a decision
  does not follow from the code; doc comments say what the game does and,
  for an address, where it was measured.
- Small commits with plain messages; author from the clone's git config;
  no `Co-Authored-By` or other agent trailers.
- The repository is public: no personal data, host names or paths of a
  machine in files or messages.
- Licensed AGPL-3.0-or-later.
