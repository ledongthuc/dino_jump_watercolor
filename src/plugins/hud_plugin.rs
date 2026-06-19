// ─── HUD Plugin ──────────────────────────────────────────────────────────────
//!
//! HUD = **H**eads-**U**p **D**isplay – the text overlay that shows the
//! current score and speed multiplier.
//!
//! This plugin:
//! - Spawns the score and speed text entities at startup.
//! - Updates their content and position every frame (handles window resize).

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::components::{ScoreText, SpeedText};
use crate::resources::{Score, Speed};

/// Bevy plugin for the score/speed HUD.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud_text);
    }
}

/// Spawn score and speed text at the top-left corner of the window.
fn spawn_hud(mut commands: Commands, windows: Query<&Window>) {
    let window = windows.single().expect("expected a window");

    // Score text (top-left)
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

    // Speed text (below score)
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
}

/// Update the HUD text content and reposition it (handles resize).
///
/// We use separate queries for each text type (with `Without` filters) to
/// avoid ambiguity – Bevy requires queries for distinct components to be
/// unambiguous.
fn update_hud_text(
    windows: Query<&Window>,
    score: Res<Score>,
    speed: Res<Speed>,
    mut score_text_query: Query<&mut Text2d, (With<ScoreText>, Without<SpeedText>)>,
    mut speed_text_query: Query<&mut Text2d, (With<SpeedText>, Without<ScoreText>)>,
    mut score_tf_query: Query<&mut Transform, (With<ScoreText>, Without<SpeedText>)>,
    mut speed_tf_query: Query<&mut Transform, (With<SpeedText>, Without<ScoreText>)>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let left_x = -window.width() / 2.0 + 10.0;
    let top_y = window.height() / 2.0;

    // Update score
    if let Ok(mut text) = score_text_query.single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
    if let Ok(mut tf) = score_tf_query.single_mut() {
        tf.translation.x = left_x;
        tf.translation.y = top_y - 15.0;
    }

    // Update speed
    if let Ok(mut text) = speed_text_query.single_mut() {
        text.0 = format!("Speed: {:.2}x", speed.multiplier);
    }
    if let Ok(mut tf) = speed_tf_query.single_mut() {
        tf.translation.x = left_x;
        tf.translation.y = top_y - 45.0;
    }
}
