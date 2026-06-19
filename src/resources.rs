// ─── Resources ───────────────────────────────────────────────────────────────
//!
//! **Resources** are Bevy's way to store *global* data that isn't tied to any
//! specific entity. Think of them as singletons – there is at most one instance
//! of each resource type in the `World`.
//!
//! Examples of good uses: configuration, random state, asset handles, timers,
//! score, game state flags, etc.

use bevy::prelude::*;

/// Whether the game is currently running (`true`) or in game-over state (`false`).
///
/// Many systems check this at the top and return early so they don't process
/// while the game is paused on "Game Over".
#[derive(Resource)]
pub struct GameRunning(pub bool);

/// Timer + interval for spawning trees.
///
/// Every frame we subtract `delta_secs` from `timer`. When it reaches zero,
/// we spawn a new tree and reset the timer.
#[derive(Resource)]
pub struct TreeSpawner {
    /// Countdown until the next tree spawn (seconds).
    pub timer: f32,
    /// How long to wait between spawns (seconds). Gets randomised slightly.
    pub interval: f32,
}

/// Current speed multiplier that increases as the dino jumps over obstacles.
///
/// Starts at `1.0` and goes up by `+0.05` per tree scored.
/// Affects tree/road/sky scroll speed.
#[derive(Resource)]
pub struct Speed {
    pub multiplier: f32,
}

/// Current score – number of trees the dino has successfully jumped over.
#[derive(Resource)]
pub struct Score(pub u32);
