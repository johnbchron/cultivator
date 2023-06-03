use bevy::prelude::Vec2;

/// Side length of hex grid in World units.
pub const HEX_SIZE: Vec2 = Vec2::splat(1.0);
/// Height of a hex tile in World units.
pub const HEX_HEIGHT: f32 = 1.0;
/// Pan speed of camera in world units per second.
pub const PAN_SPEED: f32 = 50.0;
/// Zoom speed of camera. Arbitrary units.
pub const ZOOM_SPEED: f32 = 1.0;
