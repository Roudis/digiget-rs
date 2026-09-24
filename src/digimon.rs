use std::process::exit;

use image::DynamicImage;
use showie::Trim;

use crate::{list::List, Data};

/// Enum used to assist parsing user input.
///
/// It can sort all types of inputs, and then evaluate them to a filename.
#[derive(PartialEq, Eq)]
pub enum Selection {
    /// When a random digimon is selected (`0` or `random`).
    Random,

    /// When a number is selected (number larger than 0).
    Id(usize),

    /// When a digimon name is selected.
    Name(String),
}

impl Selection {
    /// Parses a raw argument into a [`Selection`].
    pub fn parse(arg: String) -> Self {
        match arg.parse::<usize>() {
            Ok(0) => Selection::Random,
            Ok(id) => Selection::Id(id - 1),
            Err(_) if arg.eq_ignore_ascii_case("random") => Selection::Random,
            Err(_) => Selection::Name(arg),
        }
    }

    /// Evaluates the selection and returns a digimon filename.
    pub fn eval(self, list: &List) -> String {
        match self {
            Selection::Random => list.random(),
            Selection::Id(id) => list
                .get_by_id(id)
                .unwrap_or_else(|| {
                    // add 1 to id so that error message matches user input
                    eprintln!("{} is not a valid digimon number", id + 1);
                    exit(1)
                })
                .clone(),
            Selection::Name(name) => filename(&name),
        }
    }
}

/// Normalizes a user-supplied name into a sprite filename,
/// e.g. `"Omegamon Zwart"` -> `"omegamon-zwart"`.
pub fn filename(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.trim().chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_owned()
}

/// The struct used to represent a Digimon's data.
pub struct Digimon {
    /// The path of the sprite inside the embedded data, e.g. `agumon.png`.
    pub path: String,

    /// The formatted name of the digimon, usually gotten from a [List].
    pub name: String,

    /// The sprite of the Digimon, as a [DynamicImage].
    pub sprite: DynamicImage,
}

impl Digimon {
    /// Creates a new digimon.
    /// This also fetches the sprite & formats the name.
    pub fn new(arg: String, list: &List) -> Self {
        let name = Selection::parse(arg).eval(list);
        let path = format!("{name}.png");

        let bytes = Data::get(&path)
            .unwrap_or_else(|| {
                eprintln!("digimon not found, try `digiget --list`");
                exit(1)
            })
            .data
            .into_owned();

        let sprite = image::load_from_memory(&bytes).unwrap().trim();
        Self {
            path,
            name: list.format_name(&name),
            sprite,
        }
    }
}
