use bevy::prelude::*;

pub const SCREENSIZE: Vec2 = Vec2::new(1280.0, 680.0);

pub const HALF_DIM: Vec2 = Vec2::new(SCREENSIZE.x / 2.0, SCREENSIZE.y / 2.0);
pub const NODE_RADIUS: f32 = 5.0;
pub const DEFAULT_RESTING_LENGTH: f32 = 100.0;

pub const GRAVITY: Vec2 = Vec2::new(0.0, 98.0); // increased it by 10x

pub const ITERATION_COUNT : i32 = 1;
pub const ITERATION_DELTA : f32 = 1.0 / (ITERATION_COUNT as f32); 
