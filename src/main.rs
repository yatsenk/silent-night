use std::f32::consts::PI;
use rand::RngExt;

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin}, image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor}, input::{InputSystem, mouse::MouseMotion}, pbr::CascadeShadowConfigBuilder, prelude::*, render::mesh::{PlaneMeshBuilder, VertexAttributeValues}, text::FontSmoothing, window::{CursorGrabMode, WindowMode, WindowPlugin} 
};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};

const MOUSE_SENSITIVITY: f32 = 0.3;
const GROUND_TIMER: f32 = 0.5;  
const MOVEMENT_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 20.0;
const GRAVITY: f32 = -9.81;

struct OverlayColor;

impl OverlayColor {
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 200.0,
            ..default()
        })
        .init_resource::<MovementInput>()
        .init_resource::<LookInput>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 18.0,
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    text_color: OverlayColor::GREEN,
                    enabled: true,
                    ..default()
                },
            },
        ))
        .add_systems(Startup, (
            setup_player, 
            setup_map, 
            hide_cursor
        ))
        .add_systems(PreUpdate, handle_input.after(InputSystem))
        .add_systems(Update, player_look)
        .add_systems(FixedUpdate, player_movement)
        .run();
}   

fn hide_cursor(mut window: Single<&mut Window>) {
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = match window.cursor_options.grab_mode {
        CursorGrabMode::None => CursorGrabMode::Locked,
        CursorGrabMode::Locked | CursorGrabMode::Confined => CursorGrabMode::None,
    };
}

pub fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            Transform::from_xyz(0.0, 5.0, 0.0),
            Visibility::default(),
            Collider::round_cylinder(0.9, 0.3, 0.2),
            KinematicCharacterController {
                custom_mass: Some(5.0),
                up: Vec3::Y,
                offset: CharacterLength::Absolute(0.01),
                slide: true,
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Relative(0.3),
                    min_width: CharacterLength::Relative(0.5),
                    include_dynamic_bodies: false,
                }),
                // Don’t allow climbing slopes larger than 45 degrees.
                max_slope_climb_angle: 45.0_f32.to_radians(),
                // Automatically slide down on slopes smaller than 30 degrees.
                min_slope_slide_angle: 30.0_f32.to_radians(),
                apply_impulse_to_dynamic_bodies: true,
                snap_to_ground: None,
                ..default()
            },
        ))
        .with_children(|b| {
            // FPS Camera
            b.spawn((
                Camera3d::default(), 
                Transform::from_xyz(0.0, 7.0, -0.1),
            ));

            b.spawn((
                Transform::from_xyz(0.0, 0.2, -0.1),
                PointLight {
                    intensity: 15.0,
                    shadows_enabled: true,
                    range: 15.0, 
                    ..default()
                },
            ));
        });
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // sky
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("#000000").unwrap().into(),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(1_000_000.0)),
    ));

    // moon
    let moon_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("moon/scene.gltf"));

    commands.spawn(Transform::from_xyz(30.0, 300.0, 30.0))
    .with_children(|parent| {
        parent.spawn((
            SceneRoot(moon_handle.clone()),
            Transform {
                translation: Vec3::ZERO,
                rotation: Quat::from_rotation_y(0f32.to_radians()),
                scale: Vec3::splat(1.0),
            },
        ));
        parent.spawn((
            DirectionalLight {
                illuminance: light_consts::lux::FULL_MOON_NIGHT,
                shadows_enabled: true,
                ..default()
            },
            Transform {
                rotation: Quat::from_rotation_x(-PI / 4.),
                ..default()
            },
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .build(),
        ));
    });

    // forest
    let tree_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("pine_tree/scene.gltf"));

    let mut z = -100.0;
    let mut x1 = -30.0;
    for _ in 0..10 {
        commands.spawn(Transform::from_xyz(x1, rand::rng().random_range(-10.0..-5.0), z))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(tree_handle.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_y(0f32.to_radians()),
                    scale: Vec3::splat(0.1),
                },
            ));
        });
        x1 += rand::rng().random_range(25.0..40.0);
        z += rand::rng().random_range(30.0..45.0);
    }

    let mut z = -100.0;
    let mut x2 = -120.0;
    for _ in 0..10 {
        commands.spawn(Transform::from_xyz(x2, rand::rng().random_range(-10.0..-5.0), z))
        .with_children(|parent| {
            parent.spawn((
                SceneRoot(tree_handle.clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_y(0f32.to_radians()),
                    scale: Vec3::splat(0.1),
                },
            ));
        });
        x2 += rand::rng().random_range(25.0..40.0);
        z += rand::rng().random_range(30.0..45.0);
    }

    // ground
    let image_sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..default()
    });

    let ground_texture: Handle<Image> = asset_server.load_with_settings(
        "ground/Albedo.png",
        
        move |settings: &mut ImageLoaderSettings| {
            settings.sampler = image_sampler.clone();
        },
    );

    let ground_size = 500.0;
    let ground_height = 0.1;

    let mut mesh = PlaneMeshBuilder::from_size(Vec2::new(
        ground_size, 
        ground_size,     
    ))
    .build();

    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs {
            uv[0] *= 10.;
            uv[1] *= 8.;
        }
    };

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(ground_texture.clone()),
            unlit: false,
            cull_mode: None,
            ..default()
        }),
    )));

    commands.spawn((
        Transform::from_xyz(0.0, -ground_height, 0.0),
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));
}

/// Keyboard input vector
#[derive(Default, Resource, Deref, DerefMut)]
struct MovementInput(Vec3);

/// Mouse input vector
#[derive(Default, Resource, Deref, DerefMut)]
struct LookInput(Vec2);

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut movement: ResMut<MovementInput>,
    mut look: ResMut<LookInput>,
    mut mouse_events: EventReader<MouseMotion>,
) {
    if keyboard.pressed(KeyCode::KeyW) {
        movement.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement.z += 1.0
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement.x += 1.0
    }
    **movement = movement.normalize_or_zero();
    if keyboard.pressed(KeyCode::ShiftLeft) {
        **movement *= 2.0;
    }
    if keyboard.pressed(KeyCode::Space) {
        movement.y = 1.0;
    }

    for event in mouse_events.read() {
        look.x -= event.delta.x * MOUSE_SENSITIVITY;
        look.y -= event.delta.y * MOUSE_SENSITIVITY;
        look.y = look.y.clamp(-89.9, 89.9); // Limit pitch
    }
}

fn player_movement(
    time: Res<Time>,
    mut input: ResMut<MovementInput>,
    mut player: Query<(
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
) {
    let Ok((transform, mut controller, output)) = player.single_mut() else {
        return;
    };
    let delta_time = time.delta_secs();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * MOVEMENT_SPEED;
    let jump_speed = input.y * JUMP_SPEED;
    // Clear input
    **input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = GROUND_TIMER;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += GRAVITY * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));
}

fn player_look(
    mut player: Query<&mut Transform, (With<KinematicCharacterController>, Without<Camera>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
    input: Res<LookInput>,
) {
    let Ok(mut transform) = player.single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::Y, input.x.to_radians());
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::X, input.y.to_radians());
}
