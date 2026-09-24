use std::process::exit;

use image::DynamicImage;
use showie::Trim;

use crate::{fetch, list::List};

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
    /// The sprite's filename in the cache, e.g. `agumon.png`.
    pub path: String,

    /// The formatted name of the digimon, usually gotten from a [List].
    pub name: String,

    /// The sprite of the Digimon, as a [DynamicImage].
    pub sprite: DynamicImage,
}

impl Digimon {
    /// Creates a new digimon.
    /// This also fetches the sprite (from the cache, or by downloading it) & formats the name.
    ///
    /// If a random digimon was asked for and its sprite can't be downloaded,
    /// a random digimon that is already cached is used instead.
    pub fn new(arg: String, list: &List) -> Self {
        let selection = Selection::parse(arg);
        let random = selection == Selection::Random;
        let mut name = selection.eval(list);

        if !list.contains(&name) {
            eprintln!("digimon not found, try `digiget --list`");
            exit(1)
        }

        let sprite = match fetch::load(list, &name) {
            Ok(sprite) => sprite,
            Err(err) => match random.then(|| fetch::random_cached(list)).flatten() {
                Some((cached, sprite)) => {
                    name = cached;
                    sprite
                }
                None => {
                    eprintln!(
                        "could not download {}: {err}. Check your internet connection or run `digiget --download-all` while online.",
                        list.format_name(&name)
                    );
                    exit(1)
                }
            },
        };

        Self {
            path: format!("{name}.png"),
            name: list.format_name(&name),
            sprite: sprite.trim(),
        }
    }
}
