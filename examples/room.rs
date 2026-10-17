//! Prints dungeon rooms as `z1r::gfx::underworld` decodes them, for
//! `tools/emu/verify-dungeons.js`: one line of JSON per room, tiles and
//! palettes.
//!
//!     room <rom> <tables> <room>...

use z1r::gfx::underworld::decode_room;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&args[1]).expect("rom");
    let tables: usize = args[2].parse().expect("tables");
    for room in &args[3..] {
        let r = decode_room(&rom, tables, room.parse().expect("room"), false);
        let rows = |v: Vec<String>| format!("[{}]", v.join(","));
        let tiles = rows(r.tiles.iter().map(|row| format!("{:?}", row)).collect());
        let palettes = rows(r.palettes.iter().map(|row| format!("{:?}", row)).collect());
        println!("{{\"tiles\":{tiles},\"palettes\":{palettes}}}");
    }
}
