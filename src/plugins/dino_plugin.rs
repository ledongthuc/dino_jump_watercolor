// ─── Dino Plugin ─────────────────────────────────────────────────────────────
//!
//! Everything related to the player-controlled dinosaur:
//! - Spawning the dino entity at startup.
//! - Reading keyboard input to trigger a jump.
//! - Applying gravity when the dino is airborne.
//! - Keeping the dino at the correct height when the window is resized.

use bevy::prelude::*;

use crate::components::{Dino, Jump};
use crate::constants::{DINO_FLOOR_OFFSET, DINO_SIZE};
use crate::pixel_mask::{try_load_mask, PixelMask};
use crate::resources::GameRunning;

/// Bevy plugin that manages the dinosaur player character.
pub struct DinoPlugin;

impl Plugin for DinoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_dino)
            .add_systems(
                Update,
                (
                    jump_input,
                    apply_gravity,
                    load_dino_mask,
                    update_dino_y,
                ),
            );
    }
}

/// Spawn the dino entity at startup (before any Update systems run).
///
/// The dino is placed at a fixed X position (`-450`) and at the bottom of
/// the window. Its Y base is recalculated every frame in `update_dino_y`
/// to support dynamic window resizing.
fn spawn_dino(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    let window = windows.single().expect("expected a window");
    let dino_y = -window.height() / 2.0 + DINO_FLOOR_OFFSET + DINO_SIZE.y / 2.0;

    let dino_handle: Handle<Image> = asset_server.load("dino.png");

    // Try to load the pixel mask now; if the image isn't ready yet,
    // `load_dino_mask` will pick it up in a later frame.
    let mask = try_load_mask(&dino_handle, &images);

    let mut entity = commands.spawn((
        Sprite {
            image: dino_handle,
            custom_size: Some(DINO_SIZE),
            ..Default::default()
        },
        Transform::from_xyz(-450.0, dino_y, 0.0),
        Dino,
        Jump {
            velocity: 0.0,
            gravity: -3600.0,
            base_y: dino_y,
            is_jumping: false,
        },
    ));

    if let Some(mask) = mask {
        entity.insert(mask);
    }
}

/// Press **Space** to make the dino jump (only when the game is running).
///
/// If the dino is already in the air, we ignore the input (no double-jump).
fn jump_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    game: Res<GameRunning>,
    mut query: Query<&mut Jump, With<Dino>>,
) {
    // Early return if game is paused / over.
    if !game.0 {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok(mut jump) = query.single_mut() {
            if !jump.is_jumping {
                jump.velocity = 2068.0; // Impulse velocity (upward)
                jump.is_jumping = true;
            }
        }
    }
}

/// Apply gravity to the dino every frame while it's in the air.
///
/// On each frame:
/// 1. `velocity += gravity * dt`  (gravity pulls downward, so velocity decreases)
/// 2. `position.y += velocity * dt`
/// 3. If the dino is back on the ground, snap it down and reset.
fn apply_gravity(
    time: Res<Time>,
    game: Res<GameRunning>,
    mut query: Query<(&mut Transform, &mut Jump), With<Dino>>,
) {
    if !game.0 {
        return;
    }

    if let Ok((mut transform, mut jump)) = query.single_mut() {
        if jump.is_jumping {
            // v = v₀ + a·Δt   (Euler integration)
            jump.velocity += jump.gravity * time.delta_secs();
            transform.translation.y += jump.velocity * time.delta_secs();

            // Landed?
            if transform.translation.y <= jump.base_y {
                transform.translation.y = jump.base_y;
                jump.velocity = 0.0;
                jump.is_jumping = false;
            }
        }
    }
}

/// Keep the dino at the correct Y position when the window is resized.
///
/// If the dino is jumping we only update `base_y` so it lands at the
/// correct height. If it's on the ground, we snap it immediately.
fn update_dino_y(
    windows: Query<&Window>,
    mut query: Query<(&mut Transform, &mut Jump), With<Dino>>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let new_base_y = -window.height() / 2.0 + DINO_FLOOR_OFFSET + DINO_SIZE.y / 2.0;

    if let Ok((mut transform, mut jump)) = query.single_mut() {
        jump.base_y = new_base_y;
        if !jump.is_jumping {
            transform.translation.y = new_base_y;
        }
    }
}

/// Retry loading the dino's `PixelMask` if it wasn't ready at startup.
///
/// Assets are loaded asynchronously – the image data might not be available
/// when `spawn_dino` runs. This system runs every frame and inserts the mask
/// as soon as the image is ready.
fn load_dino_mask(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &Sprite), (With<Dino>, Without<PixelMask>)>,
) {
    for (entity, sprite) in query.iter() {
        if let Some(mask) = try_load_mask(&sprite.image, &images) {
            commands.entity(entity).insert(mask);
        }
    }
}
