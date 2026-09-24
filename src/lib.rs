use rust_embed::RustEmbed;

pub mod cli;
pub mod digimon;
pub mod list;
pub mod sprites;

#[derive(RustEmbed)]
#[folder = "data/sprites"]
pub struct Data;
