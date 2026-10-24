use std::f32::consts::TAU;
use avian3d::prelude::*;
use bevy::{
    camera::Exposure,
    gltf::{Gltf, GltfMesh, GltfNode},
    math::Vec3Swizzles,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_fps_controller::controller::*;
use rand::Rng;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.625, 0.0);
const NORMAL_FOV: f32 = TAU / 5.0;
const AIM_FOV: f32 = NORMAL_FOV * 0.8;
const AIM_TRANSITION_SPEED: f32 = 12.0;
const NORMAL_SENSITIVITY: f32 = 0.001;
const AIM_SENSITIVITY: f32 = NORMAL_SENSITIVITY * 0.5;
const SENSITIVITY_TRANSITION_SPEED: f32 = 0.008;
const HIP_FIRE_SPREAD: f32 = 0.025;
const CROSSHAIR_LINE_LENGTH: f32 = 12.0;
const CROSSHAIR_NORMAL_GAP: f32 = 8.0;
const CROSSHAIR_AIM_GAP: f32 = 3.0;
const HIT_MARKER_DURATION: f32 = 0.15;

fn main() {
    App::new()
    .insert_resource(GlobalAmbientLight {
        brightness: 10000.0,
        ..default()
    })
    .insert_resource(ClearColor(Color::linear_rgb(0.83, 0.96, 0.96)))
    .insert_resource(AimState::default())
    .insert_resource(HitMarkerState::default())

    .add_plugins(DefaultPlugins)
    .add_plugins(PhysicsPlugins::default())
    .add_plugins(FpsControllerPlugin)
    .add_systems(Startup, setup)

    .add_systems(
        Update,
        (manage_cursor, update_aim_state, update_aim_camera, update_aim_sensitivity).chain(),
    )
    .add_systems(
        Update,
        (scene_colliders, display_text, respawn, despawn_defeated_enemies),
    )
    .add_systems(
        Update,
        (manage_cursor, update_aim_state, update_aim_camera, update_aim_sensitivity, shoot,).chain(),
    )
    .add_systems(
        Update,
        (manage_cursor, update_aim_state, update_aim_camera, update_aim_sensitivity, update_crosshair, shoot).chain(),
    )
    .add_systems(
        Update,
        (manage_cursor, update_aim_state, update_aim_camera, update_aim_sensitivity, update_crosshair, shoot, update_hit_marker).chain(),
    )

    .run();
}

fn setup(
    mut commands: Commands, 
    mut window: Query<&mut Window>, 
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut window = window.single_mut().unwrap();
    window.title = String::from("Gun N Run");

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let height = 3.0;
    let logical_entity = commands
        .spawn((
            Collider::cylinder(0.5, height),
            Friction {
                dynamic_coefficient: 0.0,
                static_coefficient: 0.0,
                combine_rule: CoefficientCombine::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombine::Min,
            },
            LinearVelocity::ZERO,
            RigidBody::Dynamic,
            LockedAxes:: ROTATION_LOCKED,
            Mass(1.0),
            GravityScale(0.0),
            Transform::from_translation(SPAWN_POINT),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            FpsController {
                air_acceleration: 80.0,
                sensitivity: NORMAL_SENSITIVITY,
                ..default()
            },
        ))
        .insert(CameraConfig {
            height_offset: -0.5,
        })
        .id();

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: NORMAL_FOV,
            ..default()
        }),
        Exposure:: SUNLIGHT,
        RenderPlayer { logical_entity },
    ));

    commands.insert_resource(MainScene {
        handle: assets.load("playground.glb"),
        is_loaded: false,
    });

    commands.spawn((
        Text(String::from("")),
        TextFont {
            font: assets.load("fira_mono.ttf").into(),
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::BLACK),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
    ));

    commands.spawn(
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            ..default()
        })
        .with_children(|parent| {
        parent.spawn((
            CrosshairPart::Top,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-1.0),
                top: Val::Px(-(CROSSHAIR_NORMAL_GAP + CROSSHAIR_LINE_LENGTH)),
                width: Val::Px(2.0),
                height: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ));
        parent.spawn((
            CrosshairPart::Bottom,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-1.0),
                top: Val::Px(CROSSHAIR_NORMAL_GAP),
                width: Val::Px(2.0),
                height: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ));
        parent.spawn((
            CrosshairPart::Left,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-(CROSSHAIR_NORMAL_GAP + CROSSHAIR_LINE_LENGTH)),
                top: Val::Px(-1.0),
                width: Val::Px(12.0),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ));
        parent.spawn((
            CrosshairPart::Right,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(CROSSHAIR_NORMAL_GAP),
                top: Val::Px(-1.0),
                width: Val::Px(12.0),
                height: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ));
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.1, 0.1),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, -8.0),
        Collider::cuboid(1.0, 2.0, 1.0),
        RigidBody::Static,
        Enemy,
        Health {
            current: 100.0,
            maximum: 100.0,
        },
    ));
}

fn respawn(mut query: Query<(&mut Transform, &mut LinearVelocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.0 = Vec3::ZERO;
        transform.translation = SPAWN_POINT;
    }
}

// Makes a resource for Bevy to keep track off that will check whether player is aiming or not
#[derive(Resource, Default)]
struct AimState {
    is_aiming: bool,
}

fn update_aim_state(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions>,
    mut aim_state: ResMut<AimState>
) {
    let cursor_locked = cursor.grab_mode == CursorGrabMode::Locked;
    let aim_button_held = mouse_buttons.pressed(MouseButton::Right);

    aim_state.is_aiming = cursor_locked && aim_button_held;
}

fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let difference = target - current;

    if difference.abs() <= max_delta {
        target
    } else {
        current + difference.signum() * max_delta
    }
}

fn update_aim_camera(
    aim_state: Res<AimState>,
    time: Res<Time>,
    mut camera_query: Query<&mut Projection, With<Camera3d>>,
) {
    let target_fov = if aim_state.is_aiming {
        AIM_FOV
    } else {
        NORMAL_FOV
    };

    let max_change = AIM_TRANSITION_SPEED * time.delta_secs();

    for mut projection in &mut camera_query {
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = move_toward(
                perspective.fov,
                target_fov,
                max_change,
            );
        }
    }
}

fn update_aim_sensitivity(
    aim_state: Res<AimState>,
    time: Res<Time>,
    mut controller_query: Query<&mut FpsController, With<LogicalPlayer>>,
) {
    let target_sensitivity = if aim_state.is_aiming {
        AIM_SENSITIVITY
    } else {
        NORMAL_SENSITIVITY
    };

    let max_change = SENSITIVITY_TRANSITION_SPEED * time.delta_secs();

    for mut controller in &mut controller_query {
        controller.sensitivity = move_toward(
            controller.sensitivity,
            target_sensitivity,
            max_change,
        );
    }
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Health {
    current: f32,
    maximum: f32,
}

fn despawn_defeated_enemies(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Health), With<Enemy>>,
) {
    for (entity, health) in &enemy_query {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn shoot(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions>,
    aim_state: Res<AimState>,
    spatial_query: SpatialQuery,
    camera_query: Single<&GlobalTransform, With<Camera3d>>,
    player_query: Single<Entity, With<LogicalPlayer>>,
    mut enemy_query: Query<&mut Health, With<Enemy>>,
    mut hit_marker: ResMut<HitMarkerState>,
) {
    // Prevents shooting if player is not tabbed into game
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    // This check makes sure that only 1 bullet is fired per left click
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let camera_transform = camera_query.into_inner();
    let player_entity = player_query.into_inner();
    let origin = camera_transform.translation(); // gets camera world position
    
    let rotation = camera_transform.rotation();
    let forward = rotation * Vec3::NEG_Z;
    let right = rotation * Vec3::X;
    let up = rotation * Vec3::Y;

    let direction = if aim_state.is_aiming {
        forward
    } else {
        let mut rng = rand::rng();

        let horizontal_spread = rng.random_range(-HIP_FIRE_SPREAD..=HIP_FIRE_SPREAD);
        let vertical_spread = rng.random_range(-HIP_FIRE_SPREAD..=HIP_FIRE_SPREAD);

        (forward + right * horizontal_spread + up * vertical_spread).normalize()
    };
    let direction = Dir3:: new(direction).unwrap();

    let filter = SpatialQueryFilter::default().with_excluded_entities([player_entity]); // makes sure the ray does not hit the player
    let max_distance = 100.0; // max distance for ray
    let solid = true;

    if let Some(hit) = spatial_query.cast_ray(
        origin,
        direction,
        max_distance,
        solid,
        &filter,
    ) {
        // if an entity is hit make sure it looses 25.0 health
        if let Ok(mut health) = enemy_query.get_mut(hit.entity) {
            health.current -= 25.0;
            hit_marker.remaining_time = HIT_MARKER_DURATION;
        }
    }
}

#[derive(Component)]
enum CrosshairPart {
    Top,
    Bottom,
    Left,
    Right,
}

fn update_crosshair(
    aim_state: Res<AimState>,
    mut crosshair_query: Query<(&CrosshairPart, &mut Node)>,
) {
    let gap = if aim_state.is_aiming {
        CROSSHAIR_AIM_GAP
    } else {
        CROSSHAIR_NORMAL_GAP
    };

    for (part, mut node) in &mut crosshair_query {
        match part {
            CrosshairPart::Top => {
                node.top = Val::Px(-(gap + CROSSHAIR_LINE_LENGTH));
            }
            CrosshairPart::Bottom => {
                node.top = Val::Px(gap);
            }
            CrosshairPart::Left => {
                node.left = Val::Px(-(gap + CROSSHAIR_LINE_LENGTH));
            }
            CrosshairPart::Right => {
                node.left = Val::Px(gap);
            }
        }
    }
}

#[derive(Resource, Default)]
struct HitMarkerState {
    remaining_time: f32,
}

fn update_hit_marker(
    time: Res<Time>,
    mut hit_marker: ResMut<HitMarkerState>,
    mut crosshair_query: Query<&mut BackgroundColor, With<CrosshairPart>>,
) {
    hit_marker.remaining_time = (hit_marker.remaining_time - time.delta_secs()).max(0.0);

    let crosshair_color = if hit_marker.remaining_time > 0.0 {
        Color::srgb(1.0, 0.05, 0.05)
    } else {
        Color::WHITE
    };

    for mut background_color in &mut crosshair_query {
        background_color.0 = crosshair_color;
    }
}

#[derive(Resource)]
struct MainScene {
    handle: Handle<Gltf>,
    is_loaded: bool,
}

fn scene_colliders(
    mut commands: Commands,
    mut main_scene: ResMut<MainScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if main_scene.is_loaded {
        return;
    }

    let gltf = gltf_assets.get(&main_scene.handle);

    if let Some(gltf) = gltf {
        let scene = gltf.scenes.first().unwrap().clone();
        commands.spawn(WorldAssetRoot(scene));
        for node in &gltf.nodes {
            let node = gltf_node_assets.get(node).unwrap();
            if let Some(gltf_mesh) = node.mesh.clone() {
                let gltf_mesh = gltf_mesh_assets.get(&gltf_mesh).unwrap();
                for mesh_primitive in &gltf_mesh.primitives {
                    let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();
                    commands.spawn((
                        Collider::trimesh_from_mesh_with_config(mesh, TrimeshFlags::all())
                            .unwrap(),
                        RigidBody::Static,
                        node.transform,
                    ));
                }
            }
        }
        main_scene.is_loaded = true;
    }
}

fn manage_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut cursor: Single<&mut CursorOptions>,
    mut controller_query: Query<&mut FpsController>,
) {
    if btn.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
        for mut controller in &mut controller_query {
            controller.enable_input = true;
        }
    }
    if key.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
        for mut controller in &mut controller_query {
            controller.enable_input = false;
        }
    }
}

fn display_text(
    mut controller_query: Query<(&Transform, &LinearVelocity), With<LogicalPlayer>>,
    mut text_query: Query<&mut Text>,
    aim_state: Res<AimState>,
) {
    for (transform, velocity) in &mut controller_query {
        for mut text in &mut text_query {
            text.0 = format!(
                "vel: {:.2}, {:.2}, {:.2}\npos: {:.2}, {:.2}, {:.2}\nspd: {:.2}\naiming: {}",
                velocity.0.x,
                velocity.0.y,
                velocity.0.z,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                velocity.0.xz().length(),
                aim_state.is_aiming,
            );
        }
    }
}