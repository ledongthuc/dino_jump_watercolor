// ─── Tree Plugin ─────────────────────────────────────────────────────────────
//!
//! Everything related to tree obstacles:
//! - Spawning new trees at random intervals.
//! - Moving trees from right to left (scrolling).
//! - Scoring when the dino successfully jumps over a tree.
//! - Loading pixel masks for trees (async).

use bevy::prelude::*;
use rand::Rng;

use crate::components::{Dino, Scored, Tree, TreeBounds};
use crate::constants::{DINO_FLOOR_OFFSET, DINO_SIZE, TREE_SPAWN_INTERVAL, TREE_SPEED};
use crate::pixel_mask::{try_load_mask, PixelMask};
use crate::resources::{GameRunning, Score, Speed, TreeSpawner};

/// Bevy plugin that manages tree obstacles.
pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TreeSpawner {
            timer: 1.5,
            interval: TREE_SPAWN_INTERVAL,
        })
        .add_systems(Update, (spawn_trees, move_trees, score_trees, load_tree_masks));
    }
}

/// Spawn a new tree at the right edge of the screen when the timer expires.
///
/// The spawn interval includes a small random variation (±0.4 s) to make
/// the game feel less robotic.
fn spawn_trees(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<GameRunning>,
    mut spawner: ResMut<TreeSpawner>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    if !game.0 {
        return;
    }

    // Countdown
    spawner.timer -= time.delta_secs();
    if spawner.timer > 0.0 {
        return;
    }

    // Reset timer with random variation
    spawner.timer = spawner.interval;
    spawner.interval = TREE_SPAWN_INTERVAL + (rand::random::<f32>() - 0.5) * 0.8;

    if let Ok(window) = windows.single() {
        spawn_one_tree(&mut commands, &asset_server, &images, window);
    }
}

/// Pick a random tree image from the 3 available variants.
///
/// Returns (image_handle, native_size). Each variant has a different
/// size so the game feels more varied.
fn random_tree(asset_server: &Res<AssetServer>) -> (Handle<Image>, Vec2) {
    let variant = rand::thread_rng().gen_range(1..=3);
    match variant {
        1 => (asset_server.load("tree1.png"), Vec2::new(153.0, 170.0)),
        2 => (asset_server.load("tree2.png"), Vec2::new(245.0, 170.0)),
        3 => (asset_server.load("tree3.png"), Vec2::new(153.0, 222.0)),
        _ => unreachable!(),
    }
}

/// Create a single tree entity at the right side of the screen.
fn spawn_one_tree(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    images: &Res<Assets<Image>>,
    window: &Window,
) {
    let spawn_x = 900.0; // Off-screen to the right
    let (image, size) = random_tree(asset_server);
    let tree_y = -window.height() / 2.0 + DINO_FLOOR_OFFSET + size.y / 2.0;

    // Try to load the pixel mask now; retry in `load_tree_masks` if not ready.
    let mask = try_load_mask(&image, images);

    let mut entity = commands.spawn((
        Sprite {
            image,
            ..Default::default()
        },
        Transform::from_xyz(spawn_x, tree_y, 0.0),
        Tree,
        TreeBounds { size },
    ));

    if let Some(mask) = mask {
        entity.insert(mask);
    }
}

/// Move all trees from right to left (scrolling).
///
/// Speed scales with `speed.multiplier` (increases as you score).
fn move_trees(
    time: Res<Time>,
    game: Res<GameRunning>,
    speed: Res<Speed>,
    mut query: Query<&mut Transform, With<Tree>>,
) {
    if !game.0 {
        return;
    }

    for mut transform in query.iter_mut() {
        transform.translation.x -= TREE_SPEED * speed.multiplier * time.delta_secs();
    }
}

/// Score trees that the dino has jumped over.
///
/// A tree is "scored" when its **right edge** passes behind the dino's
/// **left edge**. We add a `Scored` component to avoid double-counting.
fn score_trees(
    mut commands: Commands,
    mut speed: ResMut<Speed>,
    mut score: ResMut<Score>,
    dino_query: Query<&Transform, (With<Dino>, Without<Tree>)>,
    tree_query: Query<(Entity, &Transform, &TreeBounds), (With<Tree>, Without<Scored>)>,
) {
    let dino_tf = match dino_query.single() {
        Ok(t) => t,
        Err(_) => return,
    };

    // Left edge of the dino's sprite (in world space).
    let dino_left = dino_tf.translation.x - DINO_SIZE.x / 2.0;

    for (entity, tree_tf, bounds) in tree_query.iter() {
        // Right edge of the tree sprite.
        if tree_tf.translation.x + bounds.size.x / 2.0 < dino_left {
            // Tree passed behind dino → score it.
            commands.entity(entity).insert(Scored);
            speed.multiplier += 0.05;
            score.0 += 1;
        }
    }
}

/// Retry loading pixel masks for trees that were spawned before their
/// images were ready.
fn load_tree_masks(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &Sprite), (With<Tree>, Without<PixelMask>)>,
) {
    for (entity, sprite) in query.iter() {
        if let Some(mask) = try_load_mask(&sprite.image, &images) {
            commands.entity(entity).insert(mask);
        }
    }
}
