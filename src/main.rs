use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowResolution,
};
use conway_gol_bevy::{camera::CamPlugin, life::LifePlugin, state::GameState};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: true,
                        focused: true,
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        mode: bevy::window::WindowMode::Windowed,
                        resolution: WindowResolution::new(1000., 1000.),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(MeshPickingPlugin)
        .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        .init_state::<GameState>()
        .add_plugins((CamPlugin, LifePlugin))
        .run();
}
