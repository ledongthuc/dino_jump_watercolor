// ─── Components ──────────────────────────────────────────────────────────────
//!
//! In Bevy, **Components** are plain data structs that you attach to entities.
//! Think of them as tags or data bags – they describe *what* an entity *is*
//! or *has* (position, velocity, sprite, health, …).
//!
//! Every component must derive `Component` so Bevy can store them in its
//! internal ECS (Entity Component System) tables.
//!
//! We use marker components (empty structs) just to *label* entities so that
//! systems can query them. For example, `Dino` marks the player character.

use bevy::prelude::*;

// ─── Player ──────────────────────────────────────────────────────────────────

/// Marks the player-controlled dinosaur entity.
///
/// **Why a marker?** Systems query for `With<Dino>` to find exactly one entity
/// – the player – without having to store a handle somewhere.
#[derive(Component)]
pub struct Dino;

/// Jump physics data attached to the dino.
///
/// Bevy's ECS encourages *data-oriented design*: instead of a `Player` struct
/// with nested jump fields, we store jump data as a separate component so
/// systems stay small and focused.
#[derive(Component)]
pub struct Jump {
    /// Vertical velocity (px/s). Positive = upward.
    pub velocity: f32,
    /// Gravity acceleration (px/s²). Negative = pulls down.
    pub gravity: f32,
    /// The Y position of the ground that the dino lands on.
    pub base_y: f32,
    /// Whether the dino is currently in the air.
    pub is_jumping: bool,
}

// ─── Trees / Obstacles ──────────────────────────────────────────────────────

/// Marks an entity as a tree obstacle.
#[derive(Component)]
pub struct Tree;

/// Marks a tree that the dino has *already* jumped over (scored).
///
/// We add this component *dynamically* at runtime (via `commands.entity(e).insert(Scored)`)
/// to avoid scoring the same tree twice.
#[derive(Component)]
pub struct Scored;

/// Stores the *native pixel size* of a tree sprite (varies per variant).
///
/// This is needed because we `custom_size` on trees – the actual rendering
/// size matches the image dimensions exactly (no scaling).
#[derive(Component)]
pub struct TreeBounds {
    pub size: Vec2,
}

// ─── Background ──────────────────────────────────────────────────────────────

/// Marks a road segment entity (tiled ground).
#[derive(Component)]
pub struct Road;

/// Marks a sky segment entity (parallax background).
#[derive(Component)]
pub struct Sky;

// ─── HUD (Heads-Up Display) ──────────────────────────────────────────────────

/// Marks the score text entity.
#[derive(Component)]
pub struct ScoreText;

/// Marks the speed text entity.
#[derive(Component)]
pub struct SpeedText;

// ─── Game-Over UI ────────────────────────────────────────────────────────────

/// Marks the "Game Over" text entity.
#[derive(Component)]
pub struct GameOverText;

/// Marks the "Press R to Restart" text entity.
#[derive(Component)]
pub struct RestartText;
