use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;

// ─── Components ──────────────────────────────────────────────────────────────

/// Marker for the dino entity
#[derive(Component)]
struct Dino;

/// Jump physics
#[derive(Component)]
struct Jump {
    velocity: f32,
    gravity: f32,
    base_y: f32,
    is_jumping: bool,
}

/// Marker for tree obstacles (parent entity)
#[derive(Component)]
struct Tree;

/// Marker for trees that have already been scored (dino jumped over them)
#[derive(Component)]
struct Scored;

/// Actual pixel size of the tree image (varies per variant)
#[derive(Component)]
struct TreeBounds {
    size: Vec2,
}

/// Marker for the "Game Over" text entity
#[derive(Component)]
struct GameOverText;

/// Marker for the "Press R" text entity
#[derive(Component)]
struct RestartText;

/// Marker for the sky background
#[derive(Component)]
struct Sky;

/// Marker for road segments
#[derive(Component)]
struct Road;

/// Marker for the score text entity
#[derive(Component)]
struct ScoreText;

/// Marker for the speed text entity
#[derive(Component)]
struct SpeedText;

// ─── Resources ───────────────────────────────────────────────────────────────

/// Whether the game is currently running (vs. game-over)
#[derive(Resource)]
struct GameRunning(bool);

/// Controls tree spawning
#[derive(Resource)]
struct TreeSpawner {
    timer: f32,
    interval: f32,
}

/// Current speed multiplier (increases as dino jumps over trees)
#[derive(Resource)]
struct Speed {
    multiplier: f32,
}

/// Current score (number of trees the dino has jumped over)
#[derive(Resource)]
struct Score(u32);

// ─── Constants ───────────────────────────────────────────────────────────────

const SKY_IMAGE_WIDTH: f32 = 1536.0;

const INITIAL_WINDOW_WIDTH: f32 = 1536.0;
const INITIAL_WINDOW_HEIGHT: f32 = 1024.0;

const TREE_SPEED: f32 = 700.0;
const SKY_SCROLL_SPEED: f32 = 350.0;
const TREE_SPAWN_INTERVAL: f32 = 2.2;
// Dino and tree Y are now computed dynamically from window height
const DINO_SIZE: Vec2 = Vec2::new(349.0, 200.0);
const DINO_HITBOX: Vec2 = Vec2::new(279.2, 160.0); // 80% of DINO_SIZE



const ROAD_IMAGE_WIDTH: f32 = 1536.0;
const ROAD_IMAGE_HEIGHT: f32 = 87.0;
const ROAD_SEGMENTS: u32 = 3;

const SKY_SEGMENTS: u32 = 3;

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "T-Rex Run".to_string(),
                resolution: (INITIAL_WINDOW_WIDTH as u32, INITIAL_WINDOW_HEIGHT as u32).into(),
                resizable: true,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(GameRunning(true))
        .insert_resource(Speed { multiplier: 1.0 })
        .insert_resource(Score(0))
        .insert_resource(TreeSpawner {
            timer: 1.5,
            interval: TREE_SPAWN_INTERVAL,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                jump_input,
                apply_gravity,
                spawn_trees,
                move_trees,
                move_road,
                score_trees,
                check_collisions,
                show_game_over,
                restart_game,
                move_sky,
                update_sky_y,
                update_road_y,
                update_dino_y,
                update_hud_text,
            )
                .chain(),
        )
        .run();
}

// ─── Startup ─────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    commands.spawn((Camera2d, Camera::default()));

    let window = windows.single().expect("expected a window");
    let road_y = -window.height() / 2.0 + ROAD_IMAGE_HEIGHT / 2.0;
    let dino_y = -window.height() / 2.0 + 30.0 + DINO_SIZE.y / 2.0;

    // Road – 3 tiled segments scrolling right to left at the bottom
    let road_texture: Handle<Image> = asset_server.load("road.png");
    let road_count = ROAD_SEGMENTS;
    let half_total_width = (road_count as f32 * ROAD_IMAGE_WIDTH) / 2.0;
    for i in 0..road_count {
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

    // Sky – 3 tiled segments scrolling right to left (slower than road)
    let sky_texture: Handle<Image> = asset_server.load("sky.png");
    let sky_count = SKY_SEGMENTS;
    let half_sky_width = (sky_count as f32 * SKY_IMAGE_WIDTH) / 2.0;
    for i in 0..sky_count {
        let x = -half_sky_width + SKY_IMAGE_WIDTH / 2.0 + i as f32 * SKY_IMAGE_WIDTH;
        commands.spawn((
            Sprite {
                image: sky_texture.clone(),
                custom_size: Some(Vec2::new(SKY_IMAGE_WIDTH, INITIAL_WINDOW_HEIGHT)),
                ..Default::default()
            },
            Transform::from_xyz(x, 0.0, -1.0),
            Sky,
        ));
    }

    // Score text (top-left)
    let score_x = -window.width() / 2.0 + 10.0;
    let score_y = window.height() / 2.0 - 15.0;
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        Transform::from_xyz(score_x, score_y, 10.0),
        ScoreText,
    ));

    // Speed text (top-left, below score)
    let speed_x = -window.width() / 2.0 + 10.0;
    let speed_y = window.height() / 2.0 - 45.0;
    commands.spawn((
        Text2d::new("Speed: 1.00x"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        Transform::from_xyz(speed_x, speed_y, 10.0),
        SpeedText,
    ));

    // Dino character
    commands.spawn((
        Sprite {
            image: asset_server.load("dino.png"),
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
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Press Space to jump (only when game is running)
fn jump_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    game: Res<GameRunning>,
    mut query: Query<&mut Jump, With<Dino>>,
) {
    if !game.0 {
        return;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok(mut jump) = query.single_mut() {
            if !jump.is_jumping {
                jump.velocity = 2068.0;
                jump.is_jumping = true;
            }
        }
    }
}

/// Apply gravity to the dino when it's in the air
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
            jump.velocity += jump.gravity * time.delta_secs();
            transform.translation.y += jump.velocity * time.delta_secs();
            if transform.translation.y <= jump.base_y {
                transform.translation.y = jump.base_y;
                jump.velocity = 0.0;
                jump.is_jumping = false;
            }
        }
    }
}

/// Spawn new trees at regular intervals
fn spawn_trees(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<GameRunning>,
    mut spawner: ResMut<TreeSpawner>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    if !game.0 {
        return;
    }

    spawner.timer -= time.delta_secs();
    if spawner.timer <= 0.0 {
        spawner.timer = spawner.interval;

        // Small random variation to make it less predictable
        spawner.interval = TREE_SPAWN_INTERVAL + (rand::random::<f32>() - 0.5) * 0.8;

        if let Ok(window) = windows.single() {
            spawn_one_tree(&mut commands, &asset_server, window);
        }
    }
}

/// Pick a random tree image and return (handle, native pixel size)
fn random_tree(asset_server: &Res<AssetServer>) -> (Handle<Image>, Vec2) {
    let (name, size) = match rand::thread_rng().gen_range(1..=3) {
        1 => ("tree1.png", Vec2::new(153.0, 170.0)),
        2 => ("tree2.png", Vec2::new(245.0, 170.0)),
        3 => ("tree3.png", Vec2::new(153.0, 222.0)),
        _ => unreachable!(),
    };
    (asset_server.load(name), size)
}

/// Create a single tree using a random tree sprite
fn spawn_one_tree(commands: &mut Commands, asset_server: &Res<AssetServer>, window: &Window) {
    let tree_x = 900.0;
    let (image, size) = random_tree(asset_server);
    let tree_y = -window.height() / 2.0 + 30.0 + size.y / 2.0;

    commands.spawn((
        Sprite {
            image,
            ..Default::default()
        },
        Transform::from_xyz(tree_x, tree_y, 0.0),
        Tree,
        TreeBounds { size },
    ));
}


/// Move all road segments from right to left and wrap them around
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

    // First pass: move all segments
    for mut transform in query.iter_mut() {
        transform.translation.x -= TREE_SPEED * speed.multiplier * time.delta_secs();
    }

    // Find the rightmost segment AFTER movement
    let rightmost_x = query
        .iter()
        .map(|tf| tf.translation.x)
        .fold(f32::NEG_INFINITY, f32::max);

    // Second pass: wrap segments that scrolled off the left side
    for mut transform in query.iter_mut() {
        if transform.translation.x < left_edge {
            transform.translation.x = rightmost_x + ROAD_IMAGE_WIDTH;
        }
    }
}

/// Keep the dino at the bottom of the window (+30) when resized
fn update_dino_y(
    windows: Query<&Window>,
    mut query: Query<(&mut Transform, &mut Jump), With<Dino>>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let new_base_y = -window.height() / 2.0 + 30.0 + DINO_SIZE.y / 2.0;

    if let Ok((mut transform, mut jump)) = query.single_mut() {
        jump.base_y = new_base_y;
        if !jump.is_jumping {
            transform.translation.y = new_base_y;
        }
    }
}

/// Keep the road at the bottom of the window when resized
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

/// Move all trees from right to left
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

/// Detect when a tree has passed behind the dino and increase speed
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

    // Dino left edge (the tree is scored when it passes behind the dino)
    let dino_left = dino_tf.translation.x - DINO_HITBOX.x / 2.0;

    for (entity, tree_tf, bounds) in tree_query.iter() {
        // Tree right edge (80% hitbox of its native size)
        let tree_hitbox_x = bounds.size.x * 0.8;
        if tree_tf.translation.x + tree_hitbox_x / 2.0 < dino_left {
            // Tree has passed behind the dino — score it and speed up
            commands.entity(entity).insert(Scored);
            speed.multiplier += 0.05;
            score.0 += 1;
        }
    }
}

/// AABB collision between dino and every tree
fn check_collisions(
    mut game: ResMut<GameRunning>,
    dino_query: Query<&Transform, (With<Dino>, Without<Tree>)>,
    tree_query: Query<(&Transform, &TreeBounds), (With<Tree>, Without<Dino>)>,
) {
    if !game.0 {
        return;
    }

    let dino_tf = match dino_query.single() {
        Ok(t) => t,
        Err(_) => return,
    };

    // Dino AABB (use the hitbox size, 80% of sprite)
    let dino_half = DINO_HITBOX / 2.0;
    let dino_min = dino_tf.translation.truncate() - dino_half;
    let dino_max = dino_tf.translation.truncate() + dino_half;

    for (tree_tf, bounds) in tree_query.iter() {
        // Tree hitbox (80% of its native size)
        let tree_hitbox = bounds.size * 0.8;
        let tree_half = tree_hitbox / 2.0;
        let tree_center = tree_tf.translation.truncate();
        let tree_min = tree_center - tree_half;
        let tree_max = tree_center + tree_half;

        // AABB overlap test
        if dino_min.x < tree_max.x
            && dino_max.x > tree_min.x
            && dino_min.y < tree_max.y
            && dino_max.y > tree_min.y
        {
            game.0 = false;
            break;
        }
    }
}

/// When the game ends, display a "Game Over" message.
/// This system runs every frame but only spawns the text once.
fn show_game_over(
    mut commands: Commands,
    game: Res<GameRunning>,
    // Detect whether we've already spawned the text
    existing_text: Query<Entity, With<GameOverText>>,
) {
    if game.0 {
        return;
    }
    if !existing_text.is_empty() {
        return; // already shown
    }

    // "Game Over"
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

    // "Press R to Restart"
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

/// Press R after game over to reset everything and start again
fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<GameRunning>,
    mut spawner: ResMut<TreeSpawner>,
    mut speed: ResMut<Speed>,
    mut score: ResMut<Score>,
    tree_query: Query<Entity, With<Tree>>,
    road_query: Query<Entity, With<Road>>,
    game_over_query: Query<Entity, With<GameOverText>>,
    restart_query: Query<Entity, With<RestartText>>,
    score_text_query: Query<Entity, With<ScoreText>>,
    speed_text_query: Query<Entity, With<SpeedText>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
) {
    if game.0 {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    // Despawn all trees
    for entity in tree_query.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn all road segments
    for entity in road_query.iter() {
        commands.entity(entity).despawn();
    }

    // Despawn game-over texts
    for entity in game_over_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in restart_query.iter() {
        commands.entity(entity).despawn();
    }

    // Reset spawner and speed
    spawner.timer = 1.5;
    spawner.interval = TREE_SPAWN_INTERVAL;
    speed.multiplier = 1.0;

    // Re-spawn HUD (score + speed texts)
    let window = windows.single().expect("expected a window");
    let score_x = -window.width() / 2.0 + 10.0;
    let score_y = window.height() / 2.0 - 15.0;
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        Transform::from_xyz(score_x, score_y, 10.0),
        ScoreText,
    ));
    let speed_x = -window.width() / 2.0 + 10.0;
    let speed_y = window.height() / 2.0 - 45.0;
    commands.spawn((
        Text2d::new("Speed: 1.00x"),
        TextFont {
            font_size: 28.0,
            ..Default::default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        Transform::from_xyz(speed_x, speed_y, 10.0),
        SpeedText,
    ));

    // Re-spawn road segments at their initial positions
    let window = windows.single().expect("expected a window");
    let road_y = -window.height() / 2.0 + ROAD_IMAGE_HEIGHT / 2.0;
    let road_texture: Handle<Image> = asset_server.load("road.png");
    let road_count = ROAD_SEGMENTS;
    let half_total_width = (road_count as f32 * ROAD_IMAGE_WIDTH) / 2.0;
    for i in 0..road_count {
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

    // Despawn HUD texts
    for entity in score_text_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in speed_text_query.iter() {
        commands.entity(entity).despawn();
    }

    // Reset score
    score.0 = 0;

    // Resume game
    game.0 = true;
}

/// Move all sky segments from right to left, slower than road/trees.
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

    // First pass: move all segments
    for mut transform in query.iter_mut() {
        transform.translation.x -= SKY_SCROLL_SPEED * speed.multiplier * time.delta_secs();
    }

    // Find the rightmost segment AFTER movement
    let rightmost_x = query
        .iter()
        .map(|tf| tf.translation.x)
        .fold(f32::NEG_INFINITY, f32::max);

    // Second pass: wrap segments that scrolled off the left side
    for mut transform in query.iter_mut() {
        if transform.translation.x < left_edge {
            transform.translation.x = rightmost_x + SKY_IMAGE_WIDTH;
        }
    }
}

/// Keep the sky vertically centered and scaled to window height when resized.
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

/// Update score and speed HUD text content and position (top-right).
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

    // Update score text
    if let Ok(mut text) = score_text_query.single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
    if let Ok(mut tf) = score_tf_query.single_mut() {
        tf.translation.x = -window.width() / 2.0 + 10.0;
        tf.translation.y = window.height() / 2.0 - 15.0;
    }

    // Update speed text
    if let Ok(mut text) = speed_text_query.single_mut() {
        text.0 = format!("Speed: {:.2}x", speed.multiplier);
    }
    if let Ok(mut tf) = speed_tf_query.single_mut() {
        tf.translation.x = -window.width() / 2.0 + 10.0;
        tf.translation.y = window.height() / 2.0 - 45.0;
    }
}
