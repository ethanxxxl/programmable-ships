mod staws_core;

mod ships;
mod level;
mod physics;
mod user_interface;

#[allow(dead_code)]
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::rgb_u8(0,0,0)))
        .add_plugin(ships::ShipsPlugin)
        .add_plugin(level::LevelPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(user_interface::UserInterfacePlugin)
        .run();
}
