use image::{DynamicImage, GenericImage};

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
