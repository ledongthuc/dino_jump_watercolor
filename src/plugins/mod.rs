// ─── Plugins ─────────────────────────────────────────────────────────────────
//!
//! Bevy **plugins** are the unit of modularity. Each plugin:
//!
//! 1. Registers its own systems, resources, components, and events.
//! 2. Spawns its own entities at startup.
//! 3. Is completely self-contained – it only depends on data types from
//!    sibling modules.
//!
//! This makes the code easy to **read**, **test**, and **reuse**.
//!
//! To add a new feature (e.g. sound effects, enemies, power-ups), you just
//! create a new plugin module and add it to the `main.rs` app builder.

pub use dino_plugin::DinoPlugin;
pub use tree_plugin::TreePlugin;
pub use road_plugin::RoadPlugin;
pub use sky_plugin::SkyPlugin;
pub use hud_plugin::HudPlugin;
pub use game_plugin::GamePlugin;

mod dino_plugin;
mod tree_plugin;
mod road_plugin;
mod sky_plugin;
mod hud_plugin;
mod game_plugin;
