//! What The Legend of Zelda (NES) keeps in its ROM, read and drawn from a
//! ROM the user provides: the room tables, levels and overworld, the
//! graphics, and what the Zelda Randomizer's flag string and changes to the
//! code mean. No game data is part of this crate. It says where things are,
//! not what a player may see; that is for the program using it.

pub mod flags;
pub mod gfx;
pub mod item;
pub mod level;
pub mod overworld;
pub mod rom;
pub mod tables;
pub mod z1r;
