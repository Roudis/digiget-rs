//! Downloading sprites from Wikimon and caching them on disk.
//!
//! Sprites are not bundled with digiget. The first time a digimon is shown its
//! sprite is downloaded, shrunk back to its native pixel size and saved as
//! `<cache dir>/<filename>.png`, so later runs work offline.

use std::{
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use rand::seq::SliceRandom;
use ureq::Agent;

use crate::list::{List, Source};

/// Environment variable that overrides the cache directory.
pub const CACHE_DIR_ENV: &str = "DIGIGET_CACHE_DIR";

/// How long a single download may take in total before giving up.
const TIMEOUT: Duration = Duration::from_secs(5);

/// How long connecting to the server may take before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Pause between requests in [`download_all`], to be polite to the wiki.
const POLITENESS_DELAY: Duration = Duration::from_millis(100);

/// The `User-Agent` sent with every request.
const USER_AGENT: &str = concat!(
    "digiget/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/Roudis/digiget-rs)"
);

/// Returns the directory sprites are cached in.
///
/// This is `$DIGIGET_CACHE_DIR` if set, otherwise
/// `$XDG_CACHE_HOME/digiget/sprites`, falling back to `~/.cache/digiget/sprites`.
pub fn cache_dir() -> Option<PathBuf> {
    let non_empty = |key| env::var_os(key).filter(|v| !v.is_empty()).map(PathBuf::from);

    if let Some(dir) = non_empty(CACHE_DIR_ENV) {
        return Some(dir);
    }
    let base = non_empty("XDG_CACHE_HOME").or_else(|| non_empty("HOME").map(|h| h.join(".cache")))?;
    Some(base.join("digiget").join("sprites"))
}

/// Returns the path a digimon's sprite is cached at.
fn cache_path(dir: &Path, filename: &str) -> PathBuf {
    dir.join(format!("{filename}.png"))
}

/// Builds the HTTP agent used for downloads, with short timeouts so that
/// being offline fails fast.
pub fn agent() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

/// Loads a digimon's sprite from the cache, if it is there and readable.
pub fn load_cached(filename: &str) -> Option<DynamicImage> {
    let path = cache_path(&cache_dir()?, filename);
    image::open(path).ok()
}

/// Downloads a sprite and shrinks it back to its native size.
pub fn download(agent: &Agent, source: &Source) -> Result<DynamicImage, String> {
    let bytes = agent
        .get(&source.url)
        .call()
        .and_then(|mut response| response.body_mut().read_to_vec())
        .map_err(|err| err.to_string())?;

    let image = image::load_from_memory(&bytes).map_err(|err| format!("invalid image: {err}"))?;
    let scale = source.scale.max(1);
    let (width, height) = ((image.width() / scale).max(1), (image.height() / scale).max(1));

    Ok(image.resize_exact(width, height, FilterType::Nearest))
}

/// Saves a sprite to the cache.
///
/// The sprite is written to a temporary file first and then renamed into place,
/// so an interrupted write never leaves a corrupt sprite behind.
pub fn save(filename: &str, sprite: &DynamicImage) -> Result<(), String> {
    let dir = cache_dir().ok_or("could not find a cache directory (set $DIGIGET_CACHE_DIR)")?;
    fs::create_dir_all(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;

    let mut bytes = Vec::new();
    sprite
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|err| err.to_string())?;

    let path = cache_path(&dir, filename);
    let tmp = dir.join(format!(".{filename}.png.{}.tmp", std::process::id()));
    fs::write(&tmp, &bytes)
        .and_then(|()| fs::rename(&tmp, &path))
        .map_err(|err| {
            let _ = fs::remove_file(&tmp);
            format!("{}: {err}", path.display())
        })
}

/// Loads a digimon's sprite, downloading and caching it if it isn't cached yet.
pub fn load(list: &List, filename: &str) -> Result<DynamicImage, String> {
    if let Some(sprite) = load_cached(filename) {
        return Ok(sprite);
    }

    let source = list
        .source(filename)
        .ok_or_else(|| format!("{filename} is not a known digimon"))?;
    let sprite = download(&agent(), source)?;

    if let Err(err) = save(filename, &sprite) {
        eprintln!("warning: could not cache sprite: {err}");
    }
    Ok(sprite)
}

/// Lists the filenames of every known digimon whose sprite is cached.
pub fn cached(list: &List) -> Vec<String> {
    let Some(entries) = cache_dir().and_then(|dir| fs::read_dir(dir).ok()) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let filename = name.strip_suffix(".png")?;
            list.contains(filename).then(|| filename.to_owned())
        })
        .collect()
}

/// Picks a random digimon whose sprite is cached, returning its filename and sprite.
pub fn random_cached(list: &List) -> Option<(String, DynamicImage)> {
    let mut filenames = cached(list);
    filenames.shuffle(&mut rand::rng());

    filenames
        .into_iter()
        .find_map(|filename| load_cached(&filename).map(|sprite| (filename, sprite)))
}

/// Totals reported by [`download_all`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    /// Sprites that were downloaded and cached.
    pub downloaded: usize,

    /// Sprites that were already in the cache.
    pub cached: usize,

    /// Sprites that could not be downloaded or saved.
    pub failed: usize,
}

/// Downloads every sprite that isn't cached yet, printing progress to stderr.
///
/// Individual failures are reported and skipped.
pub fn download_all(list: &List) -> Result<Summary, String> {
    let dir = cache_dir().ok_or("could not find a cache directory (set $DIGIGET_CACHE_DIR)")?;
    let agent = agent();
    let total = list.entries().count();
    let mut summary = Summary::default();

    for (id, name, filename) in list.entries() {
        if cache_path(&dir, filename).is_file() {
            summary.cached += 1;
            continue;
        }
        if summary.downloaded + summary.failed > 0 {
            thread::sleep(POLITENESS_DELAY);
        }

        eprintln!("[{id}/{total}] {name}");
        let result = list
            .source(filename)
            .ok_or_else(|| "no download source".to_owned())
            .and_then(|source| download(&agent, source))
            .and_then(|sprite| save(filename, &sprite));

        match result {
            Ok(()) => summary.downloaded += 1,
            Err(err) => {
                eprintln!("  failed: {err}");
                summary.failed += 1;
            }
        }
    }

    eprintln!(
        "downloaded {}, already cached {}, failed {} (cache: {})",
        summary.downloaded,
        summary.cached,
        summary.failed,
        dir.display()
    );
    Ok(summary)
}
