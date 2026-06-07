use bevy::prelude::*;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::window::{CursorGrabMode, WindowMode, WindowPlugin};
use bevy::text::FontSmoothing;

pub struct ScreenPlugin;

struct OverlayColor;

impl OverlayColor {
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }),
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
        .add_systems(Startup, hide_cursor);
    }
}

fn hide_cursor(mut window: Single<&mut Window>) {
    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = match window.cursor_options.grab_mode {
        CursorGrabMode::None => CursorGrabMode::Locked,
        CursorGrabMode::Locked | CursorGrabMode::Confined => CursorGrabMode::None,
    };
}
