// ─── PixelMask ───────────────────────────────────────────────────────────────
//!
//! An **alpha mask** is a per-pixel bitmap that stores whether each pixel is
//! transparent (0) or opaque (1). We build it from the sprite image data and
//! use it for **pixel-perfect collision detection**.
//!
//! Without this, we'd have to rely on bounding-box (AABB) collisions, which
//! would feel unfair because the tree & dino sprites have irregular shapes
//! and lots of transparent background.

use bevy::prelude::*;

/// A pixel-perfect alpha mask loaded from an image's RGBA data.
///
/// # How it works
///
/// 1. We read the raw RGBA bytes of the sprite image.
/// 2. For every pixel, if the **alpha channel** (byte at offset 3) is > 0,
///    we mark it as opaque (1). Otherwise transparent (0).
/// 3. At collision time, we check whether *any* overlapping pixel is opaque
///    in **both** the dino mask and the tree mask.
#[derive(Component)]
pub struct PixelMask {
    pub width: u32,
    pub height: u32,
    /// One byte per pixel: `1` = opaque, `0` = transparent.
    pub data: Vec<u8>,
}

impl PixelMask {
    /// Build a `PixelMask` from a Bevy `Image` asset.
    ///
    /// Expects the image to be in **RGBA8** format (4 bytes/pixel).
    /// The alpha byte is at offset `3` within each 4-byte pixel.
    pub fn from_image(image: &Image) -> Self {
        // Bevy stores the *logical* size separately from the GPU texture size.
        // We use `texture_descriptor.size` which matches the data buffer.
        let width = image.texture_descriptor.size.width;
        let height = image.texture_descriptor.size.height;
        let mut data = vec![0u8; (width * height) as usize];

        // Get raw RGBA bytes. `Image::data` returns `Option<&Vec<u8>>`.
        if let Some(raw_data) = &image.data {
            for y in 0..height {
                for x in 0..width {
                    // 4 bytes per pixel (RGBA); alpha is the 4th byte.
                    let pixel_idx = ((y * width + x) * 4) as usize;
                    if pixel_idx + 3 < raw_data.len() && raw_data[pixel_idx + 3] > 0 {
                        data[(y * width + x) as usize] = 1;
                    }
                }
            }
        }

        PixelMask {
            width,
            height,
            data,
        }
    }

    /// Check whether the pixel at local image coordinates `(x, y)` is opaque.
    ///
    /// - `(0, 0)` = top-left corner of the sprite.
    /// - Returns `false` if coordinates are out of bounds.
    pub fn is_opaque(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        self.data[(y as u32 * self.width + x as u32) as usize] != 0
    }
}

/// Helper: try to load a `PixelMask` from an asset handle.
///
/// Returns `Some(mask)` if the image is already loaded, `None` otherwise.
/// Assets may not be ready immediately (loaded asynchronously), so we
/// periodically retry in `load_masks` systems.
pub fn try_load_mask(handle: &Handle<Image>, images: &Assets<Image>) -> Option<PixelMask> {
    images.get(handle).map(PixelMask::from_image)
}
