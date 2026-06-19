// ─── Constants ───────────────────────────────────────────────────────────────
//! All magic numbers in one place = easy tweaking.
//!
//! Constants are in `SCREAMING_SNAKE_CASE` by convention.
//! They are `pub` so any module can import them with `use crate::constants::*`.

use bevy::prelude::Vec2;

/// Width of the sky image in pixels (used for tiling & wrapping).
pub const SKY_IMAGE_WIDTH: f32 = 1536.0;

/// Default window size used at startup.
pub const INITIAL_WINDOW_WIDTH: f32 = 1536.0;
pub const INITIAL_WINDOW_HEIGHT: f32 = 1024.0;

/// Base horizontal speed for trees and road (px/s).
pub const TREE_SPEED: f32 = 700.0;

/// Parallax scroll speed for the sky (slower → depth illusion).
pub const SKY_SCROLL_SPEED: f32 = 350.0;

/// Average time (seconds) between tree spawns.
pub const TREE_SPAWN_INTERVAL: f32 = 2.2;

/// The full sprite size of the dino image (used for AABB pre-check).
pub const DINO_SIZE: Vec2 = Vec2::new(349.0, 200.0);

// ─── Road ────────────────────────────────────────────────────────────────────

pub const ROAD_IMAGE_WIDTH: f32 = 1536.0;
pub const ROAD_IMAGE_HEIGHT: f32 = 87.0;
/// How many copies of the road image we tile side-by-side.
pub const ROAD_SEGMENTS: u32 = 3;

// ─── Sky ─────────────────────────────────────────────────────────────────────

/// How many copies of the sky image we tile side-by-side.
pub const SKY_SEGMENTS: u32 = 3;

// ─── Dino offset ─────────────────────────────────────────────────────────────

/// Distance from the bottom of the window to the dino's feet.
pub const DINO_FLOOR_OFFSET: f32 = 30.0;
