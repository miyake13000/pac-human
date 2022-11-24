use bevy::{prelude::*, sprite::collide_aabb::collide, time::FixedTimestep};
use rand::Rng;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PACMAN_SIZE: Vec3 = Vec3::new(60.0, 60.0, 0.0);
const GAP_BETWEEN_PACMAN_AND_FLOOR: f32 = 60.0;
const PACMAN_SPEED: f32 = 500.0;
// How close can the pacman get to the wall
const PACMAN_PADDING: f32 = 10.0;

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PACMAN_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const ENEMY_SPAWN_STEP: f64 = 1.0; //seconds
const ENEMY_COLOR: Color = Color::rgb(1.0, 1.0, 0.5);
const ENEMY_SIZE: Vec2 = Vec2::new(30.0, 30.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(move_pacman.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions))
                .with_system(play_collision_sound.after(check_for_collisions)),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(ENEMY_SPAWN_STEP as f64))
                .with_system(spawn_enemy),
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Pacman;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Component)]
struct Enemy;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

// Add the game's entities to our world
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let pacman_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(pacman_collision_sound));

    // Pacman
    let pacman_y = BOTTOM_WALL + GAP_BETWEEN_PACMAN_AND_FLOOR;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, pacman_y, 0.0),
                scale: PACMAN_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PACMAN_COLOR,
                ..default()
            },
            ..default()
        },
        Pacman,
        Collider,
    ));

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        }),
    );

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
}

fn spawn_enemy(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: ENEMY_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec2 {
                    x: rand::thread_rng().gen_range(LEFT_WALL..RIGHT_WALL),
                    y: rand::thread_rng().gen_range(BOTTOM_WALL..TOP_WALL),
                }
                .extend(0.0),
                scale: ENEMY_SIZE.extend(1.0),
                ..default()
            },
            ..default()
        },
        Enemy,
        Collider,
    ));
}

fn move_pacman(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Pacman>>,
) {
    let mut pacman_transform = query.single_mut();

    let x_direction = if keyboard_input.pressed(KeyCode::Left) {
        -1.0
    } else if keyboard_input.pressed(KeyCode::Right) {
        1.0
    } else {
        0.0
    };
    let y_direction = if keyboard_input.pressed(KeyCode::Down) {
        -1.0
    } else if keyboard_input.pressed(KeyCode::Up) {
        1.0
    } else {
        0.0
    };

    let new_pacan_x_position =
        pacman_transform.translation.x + x_direction * PACMAN_SPEED * TIME_STEP;
    let new_pacman_y_position =
        pacman_transform.translation.y + y_direction * PACMAN_SPEED * TIME_STEP;

    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PACMAN_SIZE.x / 2.0 + PACMAN_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PACMAN_SIZE.x / 2.0 - PACMAN_PADDING;
    let up_bound = TOP_WALL + WALL_THICKNESS / 2.0 + PACMAN_SIZE.y / 2.0 + PACMAN_PADDING;
    let bottom_bound = BOTTOM_WALL - WALL_THICKNESS / 2.0 - PACMAN_SIZE.y / 2.0 - PACMAN_PADDING;

    pacman_transform.translation.x = new_pacan_x_position.clamp(left_bound, right_bound);
    pacman_transform.translation.y = new_pacman_y_position.clamp(bottom_bound, up_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    pacman_query: Query<&Transform, With<Pacman>>,
    collider_query: Query<(Entity, &Transform, Option<&Enemy>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let pacman_transform = pacman_query.single();
    let pacman_size = pacman_transform.scale.truncate();

    // check collision with walls
    for (collider_entity, transform, maybe_enemy) in &collider_query {
        let collision = collide(
            pacman_transform.translation,
            pacman_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if collision.is_some() {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            // Enemy should be despawned and increment the scoreboard on collision
            if maybe_enemy.is_some() {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }
        }
    }
}

fn play_collision_sound(
    collision_events: EventReader<CollisionEvent>,
    audio: Res<Audio>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        audio.play(sound.0.clone());
    }
}
