// ─── Road Plugin ─────────────────────────────────────────────────────────────
//!
//! Manages the scrolling ground (road) at the bottom of the screen.
//!
//! The road is made of 3 identical image segments tiled side-by-side
//! that continuously scroll from right to left. When a segment scrolls
//! completely off the left edge, it wraps around to the right.
//!
//! This creates an **infinite scrolling** effect with just 3 entities
//! (no memory leak, no entity churn).

use bevy::prelude::*;

use crate::components::Road;
use crate::constants::{ROAD_IMAGE_HEIGHT, ROAD_IMAGE_WIDTH, ROAD_SEGMENTS, TREE_SPEED};
use crate::resources::{GameRunning, Speed};

/// Bevy plugin that manages the scrolling road.
pub struct RoadPlugin;

impl Plugin for RoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_road)
            .add_systems(Update, (move_road, update_road_y));
    }
}

/// Create 3 road segments at startup, spaced evenly across the window.
fn spawn_road(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let window = windows.single().expect("expected a window");
    let road_y = -window.height() / 2.0 + ROAD_IMAGE_HEIGHT / 2.0;

    let road_texture: Handle<Image> = asset_server.load("road.png");
    let half_total_width = (ROAD_SEGMENTS as f32 * ROAD_IMAGE_WIDTH) / 2.0;

    for i in 0..ROAD_SEGMENTS {
        let x = -half_total_width + ROAD_IMAGE_WIDTH / 2.0 + i as f32 * ROAD_IMAGE_WIDTH;
        commands.spawn((
            Sprite {
                image: road_texture.clone(),
                custom_size: Some(Vec2::new(ROAD_IMAGE_WIDTH, ROAD_IMAGE_HEIGHT)),
                ..Default::default()
            },
            Transform::from_xyz(x, road_y, -0.5),
            Road,
        ));
    }
}

/// Scroll all road segments from right to left, wrapping when off-screen.
///
/// We do **two passes**:
/// 1. Move every segment by `-speed * dt`.
/// 2. Find the rightmost segment, then wrap any segment beyond the left edge
///    to `rightmost + width` (chaining them seamlessly).
fn move_road(
    time: Res<Time>,
    game: Res<GameRunning>,
    speed: Res<Speed>,
    windows: Query<&Window>,
    mut query: Query<&mut Transform, With<Road>>,
) {
    if !game.0 {
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let left_edge = -window.width() / 2.0 - ROAD_IMAGE_WIDTH;

    // Pass 1: move all segments
    for mut transform in query.iter_mut() {
        transform.translation.x -= TREE_SPEED * speed.multiplier * time.delta_secs();
    }

    // Find the rightmost segment AFTER movement
    let rightmost_x = query
        .iter()
        .map(|tf| tf.translation.x)
        .fold(f32::NEG_INFINITY, f32::max);

    // Pass 2: wrap segments that scrolled off the left side
    for mut transform in query.iter_mut() {
        if transform.translation.x < left_edge {
            transform.translation.x = rightmost_x + ROAD_IMAGE_WIDTH;
        }
    }
}

/// Keep the road at the right Y position when the window is resized.
fn update_road_y(windows: Query<&Window>, mut query: Query<&mut Transform, With<Road>>) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let road_y = -window.height() / 2.0 + ROAD_IMAGE_HEIGHT / 2.0;
    for mut transform in query.iter_mut() {
        transform.translation.y = road_y;
    }
}
