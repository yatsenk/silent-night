use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod fog;
mod player;
mod window;
mod enviroment;

use fog::FogPlugin;
use player::PlayerPlugin;
use window::ScreenPlugin;
use enviroment::EnviromentPlugin;

fn main() {
    App::new()
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
            FogPlugin,
            PlayerPlugin,
            ScreenPlugin, // DefaultPlugin is here
            EnviromentPlugin,
        ))
        .run();
}   