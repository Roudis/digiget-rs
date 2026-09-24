use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The digimon to display, by name or number. Use "random" to get a random digimon
    pub digimon: Vec<String>,

    /// Whether to hide the digimon's name which appears above it
    #[arg(long, default_value_t = false)]
    pub hide_name: bool,

    /// List every available digimon with its number, then exit
    #[arg(short, long, default_value_t = false)]
    pub list: bool,
}
