// ─── Dino Jump Watercolor ─────────────────────────────────────────────────────
//!
//! A watercolor-themed Chrome dino-game clone built with the **Bevy** game engine (Rust).
//!
//! # Architecture
//!
//! This project is organised into **small, focused modules** – each with a
//! single responsibility – to make learning Rust + Bevy as easy as possible.
//!
//! ```
//! src/
//!   main.rs              Entry point – creates the Bevy App and adds plugins.
//!   constants.rs         All magic numbers in one place.
//!   components.rs        Bevy Component definitions (data attached to entities).
//!   resources.rs         Bevy Resource definitions (global game state).
//!   pixel_mask.rs        Alpha mask for pixel-perfect collision detection.
//!   collision.rs         Two-phase collision detection (AABB + pixel-perfect).
//!   plugins/
//!     mod.rs             Re-exports all plugins.
//!     dino_plugin.rs     Player character, jumping, gravity.
//!     tree_plugin.rs     Obstacle spawning, scrolling, scoring.
//!     road_plugin.rs     Scrolling ground segments.
//!     sky_plugin.rs      Parallax background.
//!     hud_plugin.rs      Score/speed HUD overlay.
//!     game_plugin.rs     Collision, game-over, restart.
//! ```
//!
//! # Learning Resources
//!
//! - **Bevy Book**: https://bevyengine.org/learn/book/introduction/
//! - **Bevy API docs**: https://docs.rs/bevy/latest/bevy/
//! - **Rust Book**: https://doc.rust-lang.org/book/

use bevy::prelude::*;

// Declare our top-level modules.
mod collision;
mod components;
mod constants;
mod pixel_mask;
mod plugins;
mod resources;

/// Entry point – the first function that runs when the game starts.
///
/// # What this does
///
/// 1. Creates a new Bevy `App`.
/// 2. Configures the window (title, size, resizable).
/// 3. Adds our custom plugins (each one registers its own systems & resources).
/// 4. Calls `.run()` – this enters Bevy's **game loop** (the Main Loop).
///
/// # Build commands
///
/// | Scenario                              | Command                             |
/// |---------------------------------------|-------------------------------------|
/// | **Fast dev iteration** (dynamic link) | `cargo run --features fast_dev`     |
/// | **Release build** (optimised runtime) | `cargo run --release`               |
/// | **Release + fast compile**            | `cargo run --profile release-fast`  |
/// | **Quick error check**                 | `cargo check`                       |
fn main() {
    App::new()
        // ── Bevy's built-in plugins ──
        // `DefaultPlugins` includes: renderer, audio, input, windowing, asset
        // system, and more. We override the Window settings here.
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Dino Jump Watercolor".to_string(),
                    resolution: (
                        constants::INITIAL_WINDOW_WIDTH as u32,
                        constants::INITIAL_WINDOW_HEIGHT as u32,
                    )
                        .into(),
                    resizable: false,
                    ..Default::default()
                }),
                ..Default::default()
            }), // Optional: disable the default Bevy logo splash screen.
                // .set(ImagePlugin::default_nearest())
        )
        // ── Our custom plugins ──
        // Each plugin registers its own systems, components, and resources.
        // The order here doesn't matter – Bevy's schedule runs systems based
        // on their dependencies, not the plugin registration order.
        .add_plugins((
            plugins::GamePlugin, // Resources + collision + game-over + restart
            plugins::DinoPlugin, // Dino entity + jumping + gravity
            plugins::TreePlugin, // Tree spawning + movement + scoring
            plugins::RoadPlugin, // Road scrolling
            plugins::SkyPlugin,  // Parallax sky
            plugins::HudPlugin,  // Score / speed HUD
        ))
        .run();
}
