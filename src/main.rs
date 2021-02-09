mod grid;
mod snake;

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use grid::*;
use rand::prelude::random;
use snake::*;
use std::time::Duration;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        // We'll update snake movement every 150ms
        .add_resource(SnakeMoveTimer(Timer::new(
            Duration::from_millis(150. as u64),
            true,
        )))
        .add_resource(SnakeSegments::default())
        .add_resource(LastTailPosition::default())
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_snake.system()))
        .add_system(snake_movement.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(food_spawner.system())
        .add_system(snake_timer.system())
        .add_system(snake_eating.system())
        .add_system(snake_growth.system())
        .add_plugins(DefaultPlugins)
        .add_event::<GrowthEvent>()
        .run();
}

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
    });
}

fn spawn_snake(
    commands: &mut Commands,
    materials: Res<Materials>,
    mut segments: ResMut<SnakeSegments>,
) {
    segments.0 = vec![
        commands
            .spawn(SpriteBundle {
                material: materials.head_material.clone(),
                ..Default::default()
            })
            .with(SnakeHead {
                direction: snake::Direction::Up,
            })
            .with(SnakeSegment)
            .with(Position { x: 3, y: 3})
            .with(grid::Size::square(0.8))
            .current_entity()
            .unwrap(),
        spawn_segment(
            commands,
            &materials.segment_material,
            Position { x: 3, y: 2 },
        ),
    ];
}

// Helper for spawning segments
//
fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .with(SnakeSegment)
        .with(position)
        .with(grid::Size::square(0.65))
        .current_entity()
        .unwrap()
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: ResMut<SnakeMoveTimer>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let dir: snake::Direction =
        if keyboard_input.pressed(KeyCode::Left) ||
           keyboard_input.pressed(KeyCode::A) {
            snake::Direction::Left
        } else if keyboard_input.pressed(KeyCode::Right) ||
                  keyboard_input.pressed(KeyCode::D) {
            snake::Direction::Right
        } else if keyboard_input.pressed(KeyCode::Up) ||
                  keyboard_input.pressed(KeyCode::W) {
            snake::Direction::Up
        } else if keyboard_input.pressed(KeyCode::Down) ||
                  keyboard_input.pressed(KeyCode::S) {
            snake::Direction::Down
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.direction = dir;
        }

        if !snake_timer.0.finished() {
            return;
        }

        match &head.direction {
            snake::Direction::Left  => { head_pos.x -= 1; }
            snake::Direction::Right => { head_pos.x += 1; }
            snake::Direction::Up    => { head_pos.y += 1; }
            snake::Direction::Down  => { head_pos.y -= 1; }
        };

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }
}

fn food_spawner(
    commands: &mut Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    if timer.0.tick(time.delta_seconds()).finished() {
        commands
            .spawn(SpriteBundle {
                material: materials.food_material.clone(),
                // use default for bevy init to start the timer
                ..Default::default()
            })
            .with(Food)
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(grid::Size::square(0.8));
    }
}

// Fn for ticking our snake timer.
// Kept separate from snake movement for cleanliness.
fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds());
}

fn snake_eating(
    commands: &mut Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    if !snake_timer.0.finished() {
        return;
    }

    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
            }
        }
    }
}


fn snake_growth(
    commands: &mut Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    materials: Res<Materials>,
) {
    if growth_reader.iter(&growth_events).next().is_some() {
        segments.0.push(
            spawn_segment(
                commands,
                &materials.segment_material,
                last_tail_position.0.unwrap(),
            )
        );
    }
}
