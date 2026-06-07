use bevy::pbr::VolumetricLight;
use bevy::prelude::*;

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AppSettings>()
            .add_systems(Update, tweak_scene);
    }
}

#[allow(unused)]
#[derive(Resource)]
pub struct AppSettings {
    volumetric_spotlight: bool,
    volumetric_pointlight: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            volumetric_spotlight: true,
            volumetric_pointlight: true,
        }
    }
}

pub fn tweak_scene(
    mut commands: Commands,
    mut lights: Query<(Entity, &mut DirectionalLight), Changed<DirectionalLight>>,
) {
    for (light, mut directional_light) in lights.iter_mut() {
        directional_light.shadows_enabled = true;
        commands.entity(light).insert(VolumetricLight);
    }
}
