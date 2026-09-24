use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The digimon to display, by name or number. Use "random" to get a random digimon
    pub digimon: Vec<String>,

    /// Whether to hide the digimon's name which appears above it
    #[arg(long, default_value_t = false)]
    pub hide_name: bool,

    /// Scale the sprite by this whole-number factor instead of fitting it to the terminal
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=16), conflicts_with = "no_fit")]
    pub scale: Option<u32>,

    /// Keep the sprite at its native size instead of fitting it to the terminal
    #[arg(long, default_value_t = false)]
    pub no_fit: bool,

    /// List every available digimon with its number, then exit
    #[arg(short, long, default_value_t = false)]
    pub list: bool,

    /// Download every sprite that isn't cached yet, so digiget works offline, then exit
    #[arg(long, default_value_t = false)]
    pub download_all: bool,
}
