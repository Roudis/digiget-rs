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

        for (i, entry) in reader.deserialize().enumerate() {
            let record: (String, String) = entry.unwrap();

            ids.insert(i, record.1);
            names.push(record.0);
        }

        Self { ids, names }
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
