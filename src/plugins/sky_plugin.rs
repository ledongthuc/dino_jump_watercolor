// ─── Sky Plugin ──────────────────────────────────────────────────────────────
//!
//! Manages the parallax sky background.
//!
//! **Parallax** means the sky scrolls **slower** than the road / trees,
//! which creates a depth illusion: the sky feels far away in the background.
//!
//! Like the road, we tile 3 sky segments and wrap them infinitely.

use bevy::prelude::*;

use crate::components::Sky;
use crate::constants::{SKY_IMAGE_WIDTH, SKY_SCROLL_SPEED, SKY_SEGMENTS};
use crate::resources::{GameRunning, Speed};

/// Bevy plugin that manages the parallax sky background.
pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_sky)
            .add_systems(Update, (move_sky, update_sky_y));
    }
}

/// Create 3 sky segments at startup, tiled horizontally.
fn spawn_sky(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let window = windows.single().expect("expected a window");
    let win_h = window.height();

    let sky_texture: Handle<Image> = asset_server.load("sky.png");
    let half_sky_width = (SKY_SEGMENTS as f32 * SKY_IMAGE_WIDTH) / 2.0;

    for i in 0..SKY_SEGMENTS {
        let x = -half_sky_width + SKY_IMAGE_WIDTH / 2.0 + i as f32 * SKY_IMAGE_WIDTH;
        commands.spawn((
            Sprite {
                image: sky_texture.clone(),
                custom_size: Some(Vec2::new(SKY_IMAGE_WIDTH, win_h)),
                ..Default::default()
            },
            Transform::from_xyz(x, 0.0, -1.0),
            Sky,
        ));
    }
}

/// Scroll all sky segments from right to left (slower than road).
///
/// Same two-pass wrapping logic as `move_road`.
fn move_sky(
    time: Res<Time>,
    game: Res<GameRunning>,
    speed: Res<Speed>,
    windows: Query<&Window>,
    mut query: Query<&mut Transform, With<Sky>>,
) {
    if !game.0 {
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let left_edge = -window.width() / 2.0 - SKY_IMAGE_WIDTH;

    // Pass 1: move all segments
    for mut transform in query.iter_mut() {
        transform.translation.x -= SKY_SCROLL_SPEED * speed.multiplier * time.delta_secs();
    }

    // Find the rightmost segment AFTER movement
    let rightmost_x = query
        .iter()
        .map(|tf| tf.translation.x)
        .fold(f32::NEG_INFINITY, f32::max);

    // Pass 2: wrap segments that scrolled off the left side
    for mut transform in query.iter_mut() {
        if transform.translation.x < left_edge {
            transform.translation.x = rightmost_x + SKY_IMAGE_WIDTH;
        }
    }
}

/// Scale the sky to match the window height when resized.
fn update_sky_y(windows: Query<&Window>, mut query: Query<&mut Sprite, With<Sky>>) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let win_h = window.height();
    for mut sprite in query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(SKY_IMAGE_WIDTH, win_h));
    }
}
