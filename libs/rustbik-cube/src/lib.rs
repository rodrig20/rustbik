//! Core logic for a Rubik's Cube simulator
//! This crate provides the `Cube` struct for maintaining state and `Scramble` for parsing and applying moves

mod cube;
mod moves;

pub use cube::Cube;
pub use moves::Scramble;
