mod staws_core;

#[allow(dead_code)]
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use crate::staws_core::{systems, user_interface};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::rgb_u8(0,0,0)))
        .add_startup_system(systems::setup_system.system())
        .add_startup_system(user_interface::init_ui.system())
        .add_system(user_interface::course_projection_system.system())
        .add_system(systems::kinimatics_system.system())
        .add_system(systems::user_control_system.system())
        .add_system(systems::user_interface_system.system())
        .run();
}
