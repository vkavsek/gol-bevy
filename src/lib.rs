pub mod camera;
pub mod life;
pub mod state;

pub mod prelude {
    use bevy::{color::Color, math::Vec2};

    pub const UPDATE_INTERVAL_MS: u64 = 20;
    pub const BG_COLOR: Color = Color::srgb(0.0, 0.1, 0.3);

    pub const BOARD_SIZE: u32 = 128;
    pub const BOARD_POS: Vec2 = Vec2::ZERO;
    pub const BORDER_WIDTH_PX: f32 = 8.0;
    pub const BORDER_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

    pub const CELL_SIZE_PX: Vec2 = Vec2::splat(18.0);
    pub const CELL_SCALE: Vec2 = Vec2::splat(1.0);
    pub const CELL_ALIVE_COLOR: Color = Color::srgb(0.2, 1.0, 0.2);
    pub const CELL_CLICKED_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
    pub const CELL_HOVERED_ALIVE_COLOR: Color = Color::srgb(0.2, 0.4, 1.0);
    pub const CELL_HOVERED_DEAD_COLOR: Color = Color::srgb(0.7, 0.1, 0.1);
}
