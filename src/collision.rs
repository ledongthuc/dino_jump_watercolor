// ─── Collision Detection ─────────────────────────────────────────────────────
//!
//! Two-phase collision detection:
//!
//! 1. **AABB (Axis-Aligned Bounding Box)** – fast rectangle overlap test.
//!    This quickly rejects pairs that are far apart.
//! 2. **Pixel-perfect** – if the AABB check passes, we scan every *overlapping*
//!    pixel position and check whether BOTH sprites have an opaque (non-transparent)
//!    pixel there. If so, it's a real collision.
//!
//! This gives us precise, fair collisions without the *annoying* "empty space
//! hitbox" problem.

use bevy::prelude::*;

use crate::components::{Dino, TreeBounds, Tree};
use crate::constants::DINO_SIZE;
use crate::pixel_mask::PixelMask;

/// Phase 1: Check whether two rectangles overlap (AABB test).
///
/// Each rectangle is described by its **center** and **full size**.
/// Returns `true` if they intersect.
///
/// This is extremely fast (just 4 comparisons) and weeds out most non-colliding
/// pairs before we do the expensive pixel scan.
pub fn aabb_overlap(
    dino_center: Vec2,
    dino_size: Vec2,
    tree_center: Vec2,
    tree_size: Vec2,
) -> bool {
    let dino_half = dino_size / 2.0;
    let tree_half = tree_size / 2.0;
    let dino_min = dino_center - dino_half;
    let dino_max = dino_center + dino_half;
    let tree_min = tree_center - tree_half;
    let tree_max = tree_center + tree_half;

    // Standard AABB intersection test.
    dino_min.x < tree_max.x
        && dino_max.x > tree_min.x
        && dino_min.y < tree_max.y
        && dino_max.y > tree_min.y
}

/// Phase 2: Pixel-perfect collision using alpha masks.
///
/// Once we know the AABBs overlap, we compute the **overlap region** in world
/// space and convert those world coordinates to **image pixel coordinates**
/// for both sprites. If any shared pixel is opaque in BOTH masks → collision!
///
/// # Coordinate systems
///
/// - Bevy's world: `y` increases **upward**; sprite center = `(cx, cy)`.
/// - Image pixels: `y` increases **downward**; pixel `(0, 0)` = top-left.
///
/// We flip Y when converting between the two systems.
pub fn pixel_perfect_collision(
    dino_tf: &Transform,
    dino_mask: &PixelMask,
    tree_tf: &Transform,
    tree_size: Vec2,
    tree_mask: &PixelMask,
) -> bool {
    let dino_half = DINO_SIZE / 2.0;
    let tree_half = tree_size / 2.0;

    let dino_center = dino_tf.translation.truncate();
    let tree_center = tree_tf.translation.truncate();

    let dino_min = dino_center - dino_half;
    let dino_max = dino_center + dino_half;
    let tree_min = tree_center - tree_half;
    let tree_max = tree_center + tree_half;

    // Overlap region in world space (integer pixel coords).
    let overlap_min_x = dino_min.x.max(tree_min.x) as i32;
    let overlap_max_x = dino_max.x.min(tree_max.x) as i32;
    let overlap_min_y = dino_min.y.max(tree_min.y) as i32;
    let overlap_max_y = dino_max.y.min(tree_max.y) as i32;

    // World → image pixel mapping.
    // In Bevy, the sprite's top edge is at (center.y + half_height) because
    // Y goes up. The image's top row (y=0) corresponds to that world Y.
    let dino_left = (dino_center.x - dino_half.x) as i32;
    let dino_top = (dino_center.y + dino_half.y) as i32;
    let tree_left = (tree_center.x - tree_half.x) as i32;
    let tree_top = (tree_center.y + tree_half.y) as i32;

    // Scan every pixel in the overlap region.
    for px in overlap_min_x..=overlap_max_x {
        for py in overlap_min_y..=overlap_max_y {
            let dx = px - dino_left; // image x for dino
            let dy = dino_top - py;  // image y for dino (Y flipped)
            let tx = px - tree_left; // image x for tree
            let ty = tree_top - py;  // image y for tree (Y flipped)

            if dino_mask.is_opaque(dx, dy) && tree_mask.is_opaque(tx, ty) {
                return true; // Both opaque → real collision!
            }
        }
    }

    false
}

/// Full collision check function: AABB pre-check → pixel-perfect → fallback.
///
/// Returns `true` if the dino has collided with ANY tree.
pub fn check_dino_tree_collision(
    dino_tf: &Transform,
    dino_mask_opt: Option<&PixelMask>,
    tree_query: &Query<(&Transform, &TreeBounds, Option<&PixelMask>), (With<Tree>, Without<Dino>)>,
) -> bool {
    let dino_center = dino_tf.translation.truncate();

    for (tree_tf, bounds, tree_mask_opt) in tree_query.iter() {
        let tree_center = tree_tf.translation.truncate();
        let tree_size = bounds.size;

        // Phase 1: AABB pre-check (fast rejection).
        if !aabb_overlap(dino_center, DINO_SIZE, tree_center, tree_size) {
            continue;
        }

        // Phase 2: Pixel-perfect check.
        if let (Some(dino_mask), Some(tree_mask)) = (dino_mask_opt, tree_mask_opt) {
            if pixel_perfect_collision(dino_tf, dino_mask, tree_tf, tree_size, tree_mask) {
                return true;
            }
        } else {
            // Fallback: masks haven't loaded yet → use tighter AABB (80% size).
            let dino_half = (DINO_SIZE * 0.8) / 2.0;
            let tree_half = (tree_size * 0.8) / 2.0;
            let dino_min = dino_center - dino_half;
            let dino_max = dino_center + dino_half;
            let tree_min = tree_center - tree_half;
            let tree_max = tree_center + tree_half;

            if dino_min.x < tree_max.x
                && dino_max.x > tree_min.x
                && dino_min.y < tree_max.y
                && dino_max.y > tree_min.y
            {
                return true;
            }
        }
    }

    false
}
