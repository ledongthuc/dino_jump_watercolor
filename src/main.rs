use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::Rng;

// ─── Components ──────────────────────────────────────────────────────────────

/// Marker for the dino entity
#[derive(Component)]
struct Dino;

/// Pixel-perfect alpha mask loaded from image data
#[derive(Component)]
struct PixelMask {
    width: u32,
    height: u32,
    /// One byte per pixel: 1 = opaque (non-transparent), 0 = transparent
    data: Vec<u8>,
}

impl PixelMask {
    fn from_image(image: &Image) -> Self {
        let width = image.texture_descriptor.size.width;
        let height = image.texture_descriptor.size.height;
        let mut data = vec![0u8; (width * height) as usize];

        if let Some(raw_data) = &image.data {
            for y in 0..height {
                for x in 0..width {
                    // RGBA format: 4 bytes per pixel, alpha at offset 3
                    let pixel_idx = ((y * width + x) * 4) as usize;
                    if pixel_idx + 3 < raw_data.len() && raw_data[pixel_idx + 3] > 0 {
                        data[(y * width + x) as usize] = 1;
                    }
                }
            }
        }

        PixelMask {
            width,
            height,
            data,
        }
    }

    /// Check if pixel at (x, y) in local image coordinates is opaque
    fn is_opaque(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        self.data[(y as u32 * self.width + x as u32) as usize] != 0
    }
}

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
// Full sprite size (used for AABB pre-check before pixel-perfect collision)



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
                load_masks,
            )
                .chain(),
        )
        .run();
}

// ─── Startup ─────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, images: Res<Assets<Image>>, windows: Query<&Window>) {
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
    let dino_handle: Handle<Image> = asset_server.load("dino.png");
    let dino_mask = images.get(&dino_handle).map(PixelMask::from_image);

    let mut dino_entity = commands.spawn((
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
    if let Some(mask) = dino_mask {
        dino_entity.insert(mask);
    }
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
    images: Res<Assets<Image>>,
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
            spawn_one_tree(&mut commands, &asset_server, &images, window);
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
fn spawn_one_tree(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    images: &Res<Assets<Image>>,
    window: &Window,
) {
    let tree_x = 900.0;
    let (image, size) = random_tree(asset_server);
    let tree_y = -window.height() / 2.0 + 30.0 + size.y / 2.0;

    let tree_mask = images.get(&image).map(PixelMask::from_image);

    let mut entity = commands.spawn((
        Sprite {
            image,
            ..Default::default()
        },
        Transform::from_xyz(tree_x, tree_y, 0.0),
        Tree,
        TreeBounds { size },
    ));
    if let Some(mask) = tree_mask {
        entity.insert(mask);
    }
}

/// Load pixel masks for any entities that were spawned before their images were ready.
fn load_masks(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    dino_query: Query<(Entity, &Sprite), (With<Dino>, Without<PixelMask>)>,
    tree_query: Query<(Entity, &Sprite), (With<Tree>, Without<PixelMask>)>,
) {
    for (entity, sprite) in dino_query.iter() {
        if let Some(image) = images.get(&sprite.image) {
            commands.entity(entity).insert(PixelMask::from_image(image));
        }
    }
    for (entity, sprite) in tree_query.iter() {
        if let Some(image) = images.get(&sprite.image) {
            commands.entity(entity).insert(PixelMask::from_image(image));
        }
    }
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
    // Use the dino's non-transparent left edge (offset from center)
    let dino_left = dino_tf.translation.x - DINO_SIZE.x / 2.0;

    for (entity, tree_tf, bounds) in tree_query.iter() {
        // Tree right edge (use full sprite size for scoring)
        if tree_tf.translation.x + bounds.size.x / 2.0 < dino_left {
            // Tree has passed behind the dino — score it and speed up
            commands.entity(entity).insert(Scored);
            speed.multiplier += 0.05;
            score.0 += 1;
        }
    }
}

/// Check if two rectangles overlap (AABB fast rejection)
fn aabb_overlap(
    dino_center: Vec2, dino_size: Vec2,
    tree_center: Vec2, tree_size: Vec2,
) -> bool {
    let dino_half = dino_size / 2.0;
    let tree_half = tree_size / 2.0;
    let dino_min = dino_center - dino_half;
    let dino_max = dino_center + dino_half;
    let tree_min = tree_center - tree_half;
    let tree_max = tree_center + tree_half;

    dino_min.x < tree_max.x
        && dino_max.x > tree_min.x
        && dino_min.y < tree_max.y
        && dino_max.y > tree_min.y
}

/// Pixel-perfect collision check using alpha masks.
/// Returns true if any overlapping pixel position has non-transparent alpha in BOTH sprites.
fn pixel_perfect_collision(
    dino_tf: &Transform,
    dino_size: Vec2,
    dino_mask: &PixelMask,
    tree_tf: &Transform,
    tree_size: Vec2,
    tree_mask: &PixelMask,
) -> bool {
    let dino_half = dino_size / 2.0;
    let tree_half = tree_size / 2.0;

    let dino_min = dino_tf.translation.truncate() - dino_half;
    let dino_max = dino_tf.translation.truncate() + dino_half;
    let tree_min = tree_tf.translation.truncate() - tree_half;
    let tree_max = tree_tf.translation.truncate() + tree_half;

    // Compute the overlap region in world space
    let overlap_min_x = dino_min.x.max(tree_min.x) as i32;
    let overlap_max_x = dino_max.x.min(tree_max.x) as i32;
    let overlap_min_y = dino_min.y.max(tree_min.y) as i32;
    let overlap_max_y = dino_max.y.min(tree_max.y) as i32;

    // Image pixel (0,0) maps to top-left corner of the sprite in world space.
    // In Bevy's y-up coordinate system, the top of the sprite is at (center.y + half_height).
    let dino_left = (dino_tf.translation.x - dino_half.x) as i32;
    let dino_top = (dino_tf.translation.y + dino_half.y) as i32;
    let tree_left = (tree_tf.translation.x - tree_half.x) as i32;
    let tree_top = (tree_tf.translation.y + tree_half.y) as i32;

    for px in overlap_min_x..=overlap_max_x {
        for py in overlap_min_y..=overlap_max_y {
            // Convert world coordinates to image pixel coordinates
            // Image x = world_x - left_edge
            // Image y = top_edge - world_y  (y is flipped: image origin is top-left)
            let dx = px - dino_left;
            let dy = dino_top - py;
            let tx = px - tree_left;
            let ty = tree_top - py;

            if dino_mask.is_opaque(dx, dy) && tree_mask.is_opaque(tx, ty) {
                return true;
            }
        }
    }

    false
}

/// Collision detection: AABB pre-check + pixel-perfect alpha mask check.
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
    let dino_center = dino_tf.translation.truncate();

    for (tree_tf, bounds, tree_mask_opt) in tree_query.iter() {
        let tree_center = tree_tf.translation.truncate();
        let tree_size = bounds.size;

        // Phase 1: AABB pre-check using full sprite sizes (fast rejection)
        if !aabb_overlap(dino_center, DINO_SIZE, tree_center, tree_size) {
            continue;
        }

        // Phase 2: Pixel-perfect check using alpha masks
        if let (Some(dino_mask), Some(tree_mask)) = (dino_mask_opt, tree_mask_opt) {
            if pixel_perfect_collision(
                dino_tf, DINO_SIZE, dino_mask,
                tree_tf, tree_size, tree_mask,
            ) {
                game.0 = false;
                return;
            }
        } else {
            // Fallback when masks are not loaded yet: use tighter AABB (80%)
            let dino_half = (DINO_SIZE * 0.8) / 2.0;
            let tree_half = (tree_size * 0.8) / 2.0;
            let dino_min = dino_center - dino_half;
            let dino_max = dino_center + dino_half;
            let tree_min = tree_center - tree_half;
            let tree_max = tree_center + tree_half;

            if dino_min.x < tree_max.x
                && dino_max.x > tree_min.x
                && dino_min.y < tree_max.y
                && dino_max.y > tree_min.y
            {
                game.0 = false;
                return;
            }
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
