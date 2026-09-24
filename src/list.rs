#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::io::Cursor;

use bimap::BiHashMap;

/// A parsed representation of `names.csv`.
///
/// Used to derive filenames from numeric IDs, and to
/// format image filenames back into proper digimon names.
pub struct List {
    /// The IDs and their corresponding filenames.
    ids: BiHashMap<usize, String>,

    /// All the proper, formatted names in order of ID.
    names: Vec<String>,

    /// Where each sprite can be downloaded from, in order of ID.
    sources: Vec<Source>,
}

/// Where a digimon's sprite can be downloaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The URL of the upscaled sprite image.
    pub url: String,

    /// How many times the image at [`Source::url`] is larger than the native sprite.
    pub scale: u32,
}

impl List {
    /// Reads a new [`List`] from `data/names.csv`.
    pub fn read() -> Self {
        const FILE: &str = include_str!("../data/names.csv");
        const CAPACITY: usize = 1000;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Cursor::new(FILE));

        let mut ids = BiHashMap::with_capacity(CAPACITY);
        let mut names = Vec::with_capacity(CAPACITY);
        let mut sources = Vec::with_capacity(CAPACITY);

        for (i, entry) in reader.deserialize().enumerate() {
            let (name, file, url, scale): (String, String, String, u32) = entry.unwrap();

            ids.insert(i, file);
            names.push(name);
            sources.push(Source { url, scale });
        }

        Self {
            ids,
            names,
            sources,
        }
    }

    /// Takes a filename and looks up the proper display name.
    ///
    /// # Examples
    ///
    /// ```
    /// use digiget::list::List;
    /// let list = List::read();
    /// assert_eq!(list.format_name("agumon"), "Agumon")
    /// ```
    pub fn format_name(&self, filename: &str) -> String {
        self.ids
            .get_by_right(filename)
            .and_then(|id| self.names.get(*id))
            .map_or_else(|| filename.to_owned(), Clone::clone)
    }

    /// Gets a digimon filename by its ID.
    pub fn get_by_id(&self, id: usize) -> Option<&String> {
        self.ids.get_by_left(&id)
    }

    /// Returns whether `filename` is a known digimon.
    pub fn contains(&self, filename: &str) -> bool {
        self.ids.contains_right(filename)
    }

    /// Looks up where a digimon's sprite can be downloaded from.
    ///
    /// # Examples
    ///
    /// ```
    /// use digiget::list::List;
    /// let list = List::read();
    /// let source = list.source("agumon").unwrap();
    /// assert!(source.url.starts_with("https://"));
    /// assert!(source.scale > 0);
    /// ```
    pub fn source(&self, filename: &str) -> Option<&Source> {
        self.ids
            .get_by_right(filename)
            .and_then(|id| self.sources.get(*id))
    }

    /// Gets a random digimon & returns its filename.
    pub fn random(&self) -> String {
        let idx = rand::random_range(0..self.ids.len());
        self.ids.get_by_left(&idx).unwrap().clone()
    }

    /// Iterates over `(number, display name, filename)` in order.
    pub fn entries(&self) -> impl Iterator<Item = (usize, &str, &str)> {
        self.names.iter().enumerate().map(|(i, name)| {
            let file = self.ids.get_by_left(&i).unwrap();
            (i + 1, name.as_str(), file.as_str())
        })
    }
}
