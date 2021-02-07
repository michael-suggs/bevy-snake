use bevy::prelude::*;

mod snake;

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_snake.system()))
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(snake::Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
    });
}

fn spawn_snake(commands: &mut Commands, materials: Res<snake::Materials>) {
    commands
        .spawn(SpriteBundle {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(snake::SnakeHead);
}
