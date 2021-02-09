use bevy::prelude::*;
use std::time::Duration;

pub struct SnakeHead {
    pub direction: Direction,
}

pub struct Materials {
    pub head_material: Handle<ColorMaterial>,
    pub food_material: Handle<ColorMaterial>,
    pub segment_material: Handle<ColorMaterial>,
}

pub struct SnakeSegment;
#[derive(Default)]
pub struct SnakeSegments(pub Vec<Entity>);

pub struct Food;
pub struct FoodSpawnTimer(pub Timer);

impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

// used to quantify the direction our snake is moving
#[derive(PartialEq, Copy, Clone)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

// Implementing step-wise motion (since Snake movement typically isn't smooth)
pub struct SnakeMoveTimer(pub Timer);
