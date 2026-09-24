use image::{imageops::FilterType, DynamicImage, GenericImage};

use crate::digimon::Digimon;

/// Combines several digimon sprites into one by stitching them horizontally.
pub fn combine(digimons: &[Digimon]) -> DynamicImage {
    let mut width: u32 = 0;
    let mut height: u32 = 0;

    for digimon in digimons {
        width += digimon.sprite.width() + 1;
        if digimon.sprite.height() > height {
            height = digimon.sprite.height();
        }
    }

    let mut combined = DynamicImage::new_rgba8(width - 1, height);
    let mut shift = 0;

    for digimon in digimons {
        combined
            .copy_from(&digimon.sprite, shift, height - digimon.sprite.height())
            .unwrap();
        shift += digimon.sprite.width() + 1;
    }

    combined
}

/// Lines kept free below the sprite for its name and the shell prompt.
const RESERVED_ROWS: u32 = 2;

/// Resizes pixel art with nearest-neighbour sampling, so edges stay crisp and
/// no half-transparent pixels appear (the renderer treats any alpha as opaque).
fn resize(sprite: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    sprite.resize_exact(width.max(1), height.max(1), FilterType::Nearest)
}

/// Scales a sprite by a whole-number factor.
pub fn scale(sprite: &DynamicImage, factor: u32) -> DynamicImage {
    resize(sprite, sprite.width() * factor, sprite.height() * factor)
}

/// Fits a sprite into a terminal of `cols` x `rows` cells.
///
/// Each cell shows one pixel across and two pixels down, so the space
/// available is `cols` x `2 * rows` pixels. Sprites that fit are enlarged by
/// the largest whole-number factor that still fits; sprites that don't fit
/// are shrunk just enough to fit.
pub fn fit(sprite: &DynamicImage, cols: u32, rows: u32) -> DynamicImage {
    let max_w = cols;
    let max_h = rows.saturating_sub(RESERVED_ROWS) * 2;
    let (w, h) = (sprite.width(), sprite.height());

    if max_w == 0 || max_h == 0 {
        return sprite.clone();
    }

    if w <= max_w && h <= max_h {
        let factor = (max_w / w).min(max_h / h);
        return scale(sprite, factor);
    }

    let ratio = (f64::from(max_w) / f64::from(w)).min(f64::from(max_h) / f64::from(h));
    resize(
        sprite,
        (f64::from(w) * ratio).floor() as u32,
        (f64::from(h) * ratio).floor() as u32,
    )
}

/// Returns the terminal size as `(cols, rows)`.
///
/// Checks stdout first, then stderr and stdin so it still works when stdout
/// is piped, and lets `COLUMNS` / `LINES` override the detected values.
pub fn terminal_size() -> Option<(u32, u32)> {
    let detected = terminal_size::terminal_size()
        .or_else(|| terminal_size::terminal_size_of(std::io::stderr()))
        .or_else(|| terminal_size::terminal_size_of(std::io::stdin()))
        .map(|(w, h)| (u32::from(w.0), u32::from(h.0)));

    let env = |key| std::env::var(key).ok().and_then(|v| v.parse::<u32>().ok());
    match (env("COLUMNS"), env("LINES"), detected) {
        (Some(cols), Some(rows), _) => Some((cols, rows)),
        (cols, rows, Some((dc, dr))) => Some((cols.unwrap_or(dc), rows.unwrap_or(dr))),
        _ => None,
    }
}
