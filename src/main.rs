mod level;
mod physics;
mod ships;
mod user_interface;

#[allow(dead_code)]
use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, RegisterInspectable};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)

        .add_plugin(WorldInspectorPlugin::new())
        .register_type::<physics::Kinimatics>()
        .register_type::<ships::Ship>()
        .register_type::<ships::Engine>()
        .register_type::<ships::Throttle>()
        .register_type::<ships::Missile>()
        .register_type::<level::AstroObject>()

        .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .add_plugin(ships::ShipsPlugin)
        .add_plugin(level::LevelPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(user_interface::UserInterfacePlugin)
        .run();
}
