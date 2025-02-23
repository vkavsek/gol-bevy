use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::{prelude::BG_COLOR, state::GameState};

pub struct CamPlugin;

impl Plugin for CamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .insert_resource(ClearColor(BG_COLOR))
            .add_systems(OnEnter(GameState::Load), spawn_cam);
    }
}

// Init
fn spawn_cam(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCam {
            grab_buttons: vec![],
            ..default()
        },
        OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            scale: 0.8,
            near: -1000.0,
            far: 1000.0,
            ..OrthographicProjection::default_2d()
        },
        Msaa::Off,
    ));
}
