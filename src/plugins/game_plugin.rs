// ─── Game Plugin ─────────────────────────────────────────────────────────────
//!
//! Core game-logic plugin that ties everything together:
//!
//! - **Collision detection** – checks for dino↔tree collisions every frame.
//! - **Game Over** – displays "Game Over" + "Press R" text when the dino hits.
//! - **Restart** – resets all entities and state when the player presses R.
//!
//! This plugin owns the `GameRunning`, `Speed`, and `Score` resources and
//! is responsible for toggling game state.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::collision::check_dino_tree_collision;
use crate::components::{
    Dino, GameOverText, RestartText, Road, ScoreText, SpeedText, Tree,
    TreeBounds,
};
use crate::pixel_mask::PixelMask;
use crate::constants::{ROAD_IMAGE_HEIGHT, ROAD_IMAGE_WIDTH, ROAD_SEGMENTS, TREE_SPAWN_INTERVAL};
use crate::resources::{GameRunning, Score, Speed, TreeSpawner};

/// Bevy plugin for core game logic (collision, game-over, restart).
///
/// Also spawns the **camera** at startup – without a camera, Bevy renders
/// nothing (black screen).
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameRunning(true))
            .insert_resource(Speed { multiplier: 1.0 })
            .insert_resource(Score(0))
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (check_collisions, show_game_over, restart_game).chain(),
            );
    }
}

/// Spawn a 2D orthographic camera.
///
/// `Camera2d` is a Bevy bundle that sets up a 2D camera with sensible
/// defaults: `(-1, 1)` world units map to the window edges.
///
/// Without this, there's no viewport → **black screen**.
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ─── Collision Detection ─────────────────────────────────────────────────────

/// Every frame, check whether the dino collides with any tree.
///
/// Uses a two-phase approach:
/// 1. AABB (fast rectangle overlap) – cheap rejection.
/// 2. Pixel-perfect alpha mask check – expensive but accurate.
///
/// On collision, sets `game.0 = false` → all gameplay systems stop.
fn check_collisions(
    mut game: ResMut<GameRunning>,
    dino_query: Query<(&Transform, Option<&PixelMask>), (With<Dino>, Without<Tree>)>,
    tree_query: Query<(&Transform, &TreeBounds, Option<&PixelMask>), (With<Tree>, Without<Dino>)>,
) {
    if !game.0 {
        return;
    }

    let (dino_tf, dino_mask_opt) = match dino_query.single() {
        Ok(t) => t,
        Err(_) => return,
    };

    if check_dino_tree_collision(dino_tf, dino_mask_opt, &tree_query) {
        game.0 = false; // Game over!
    }
}

// ─── Game Over UI ────────────────────────────────────────────────────────────

/// Display "Game Over" and "Press R to Restart" when the game ends.
///
/// This runs every frame but only spawns the text entities **once**
/// (it checks `existing_text` before spawning).
fn show_game_over(
    mut commands: Commands,
    game: Res<GameRunning>,
    existing_text: Query<Entity, With<GameOverText>>,
) {
    if game.0 {
        return; // Game is still running – do nothing.
    }
    if !existing_text.is_empty() {
        return; // Already displayed – don't spawn duplicates.
    }

    // "Game Over" heading
    commands.spawn((
        Text2d::new("Game Over"),
        TextFont {
            font_size: 64.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
        Transform::from_xyz(0.0, 80.0, 10.0),
        GameOverText,
    ));

    // "Press R to Restart" instruction
    commands.spawn((
        Text2d::new("Press R to Restart"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 20.0, 10.0),
        RestartText,
    ));
}

// ─── Restart ─────────────────────────────────────────────────────────────────

/// Press **R** after game over to reset everything and start a new game.
///
/// This is the most complex system because it must:
/// 1. Despawn all trees, road segments, and game-over texts.
/// 2. Re-spawn road and HUD at their initial positions.
/// 3. Reset all resources (timer, speed, score).
/// 4. Set `game.0 = true`.
fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<GameRunning>,
    mut spawner: ResMut<TreeSpawner>,
    mut speed: ResMut<Speed>,
    mut score: ResMut<Score>,
    // Entities to despawn
    tree_query: Query<Entity, With<Tree>>,
    road_query: Query<Entity, With<Road>>,
    game_over_query: Query<Entity, With<GameOverText>>,
    restart_query: Query<Entity, With<RestartText>>,
    score_text_query: Query<Entity, With<ScoreText>>,
    speed_text_query: Query<Entity, With<SpeedText>>,
    // Assets needed for re-spawning
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    // Only react if game is over and R was just pressed.
    if game.0 {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    // ── Despawn dynamic entities ──

    for entity in tree_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in road_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in game_over_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in restart_query.iter() {
        commands.entity(entity).despawn();
    }

    // ── Reset state ──

    spawner.timer = 1.5;
    spawner.interval = TREE_SPAWN_INTERVAL;
    speed.multiplier = 1.0;
    score.0 = 0;

    // ── Re-spawn road segments ──

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

    // ── Re-spawn HUD (replaces old HUD entities) ──

    for entity in score_text_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in speed_text_query.iter() {
        commands.entity(entity).despawn();
    }

    // New score text
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        Transform::from_xyz(
            -window.width() / 2.0 + 10.0,
            window.height() / 2.0 - 15.0,
            10.0,
        ),
        ScoreText,
    ));

    // New speed text
    commands.spawn((
        Text2d::new("Speed: 1.00x"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        Transform::from_xyz(
            -window.width() / 2.0 + 10.0,
            window.height() / 2.0 - 45.0,
            10.0,
        ),
        SpeedText,
    ));

    // ── Resume game ──

    game.0 = true;
}
