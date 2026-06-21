use std::f32::consts::PI;
use rand::RngExt;
use argh::FromArgs;

use bevy::prelude::*;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::render::view::{VisibilityRange, NoCpuCulling};
use bevy::render::mesh::{PlaneMeshBuilder, VertexAttributeValues};
use bevy_rapier3d::prelude::*;

static NORMAL_VISIBILITY_RANGE: VisibilityRange = VisibilityRange {
    start_margin: 0.0..0.0,
    end_margin: 3.0..4.0,
    use_aabb: false,
};

static INVISIBLE_VISIBILITY_RANGE: VisibilityRange = VisibilityRange {
    start_margin: 4.0..5.0,
    end_margin: 8.0..9.0,
    use_aabb: false,
};

pub struct EnviromentPlugin;

impl Plugin for EnviromentPlugin {
    fn build(&self, app: &mut App) {
        let args: Args = argh::from_env();

        app
            .insert_resource(ClearColor(Color::Srgba(Srgba {
                red: 0.02,
                green: 0.02,
                blue: 0.02,
                alpha: 1.0,
            })))
            .insert_resource(AmbientLight {
                brightness: 100.0,
                ..default()
            })
            .insert_resource(args)
            .add_systems(Startup, setup_map)
            .add_systems(Update, set_visibility_ranges);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum MainModel {
    Showed
}

#[derive(FromArgs, Resource)]
#[argh(description = "GIT_HASH")]
struct Args {
    /// whether to use GPU culling only
    #[argh(switch)]
    no_cpu_culling: bool,
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

fn set_visibility_ranges(
    mut commands: Commands,
    mut new_meshes: Query<Entity, Added<Mesh3d>>,
    children: Query<(Option<&ChildOf>, Option<&MainModel>)>,
    args: Res<Args>,
) {
    for new_mesh in new_meshes.iter_mut() {
        let (mut current, mut main_model) = (new_mesh, None);
        while let Ok((child_of, maybe_main_model)) = children.get(current) {
            if let Some(model) = maybe_main_model {
                main_model = Some(model);
                break;
            }
            match child_of {
                Some(child_of) => current = child_of.parent(),
                None => break,
            }
        }

        match main_model {
            Some(MainModel::Showed) => {
                let mut entity_commands = commands.entity(new_mesh);
                entity_commands
                    .insert(NORMAL_VISIBILITY_RANGE.clone())
                    .insert(MainModel::Showed);
                if args.no_cpu_culling {
                    entity_commands.insert(NoCpuCulling);
                }
            }
            None => {}
        }
    }
}

