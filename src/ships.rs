use std::clone;

use super::physics::KinimaticsBundle;
use bevy::prelude::*;

use bevy_inspector_egui::Inspectable;
pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup_system)
            .add_system(user_control_system);
    }
}

/// :COMPONENT: Temporary marker compenent
#[derive(Component)]
pub struct Controlled;

/// :COMPONENT: Describes how an engine is controlled.
#[derive(Reflect, Component, Clone, Copy)]
#[reflect(Component)]
pub enum Throttle {
    /// Either all on (true) or all off (false).
    Fixed(bool),
    /// Must be on the range \[0,1\]. Numbers outside of this region don't
    /// have any physical meaning.
    Variable(f32),
}

impl Default for Throttle {
    fn default() -> Self {
        Self::Variable(0.0)
    }
}

/// :COMPONENT: An engine which can be attached a ship.
/// The physics plugin looks for Engine components, and will apply the
/// applicable forces on its entity.
#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct Engine {
    pub fuel: f32,
    pub max_thrust: f32,
    /// Units of force
    pub throttle: Throttle,
}

/// :COMPONENT: Marker component for ships (in general).
#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub struct Ship;

/// :BUNDLE: Provided for convenience. Describes a generic ship.
#[derive(Bundle, Default)]
pub struct ShipBundle {
    pub ship: Ship,
    pub engine: Engine,

    #[bundle]
    pub kinimatics_bundle: KinimaticsBundle,
}

/// :COMPONENT: Missiles which can be spawned in from ships.
/// When launched, if they have a target, the missile will
/// do its best to navigate to that target.
#[derive(Reflect, Default, Component)]
#[reflect(Component)]
pub struct Missile {
    pub target: Option<Entity>,
    pub blast_radius: f32,
}

/// :BUNDLE: Provided for convenience. Describes a generic missile.
#[derive(Bundle, Default)]
pub struct MissileBundle {
    pub missile: Missile,
    pub engine: Engine,

    #[bundle]
    pub kinimatics_bundle: KinimaticsBundle,
}

/// Resource which holds all the sprites used to represent ships on the display.
#[derive(Clone)]
struct ShipSprites {
    generic_ship: SpriteBundle,
}

fn startup_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let sprite_resource = ShipSprites {
        generic_ship: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::new(0.75, 0.75, 0.0)),
            texture: asset_server.load("../assets/ship_1.png"),
            ..Default::default()
        },
    };

    commands.insert_resource(sprite_resource.clone());

    // Add a ship (temporary)
    commands
        .spawn()
        .insert_bundle(ShipBundle {
            kinimatics_bundle: KinimaticsBundle::build()
                .insert_mass(100.0)
                .insert_translation(Vec3::new(500.0, 500.0, 0.0)),
            engine: Engine {
                max_thrust: 1000.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Controlled {})
        .with_children(|p| {
            p.spawn_bundle(sprite_resource.generic_ship.clone());
        });
}

/// Temporary system which give the user control over a ship.
fn user_control_system(
    mut query: Query<(&mut Ship, &mut Transform, &mut Engine), With<Controlled>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let drot: f32 = std::f32::consts::PI * time.delta_seconds();

    query.for_each_mut(|(_ship, mut k_bod, mut eng)| {
        if input.get_pressed().count() == 0 {
            eng.throttle = Throttle::Fixed(false);
        }

        for i in input.get_pressed() {
            match i {
                KeyCode::W | KeyCode::Up => eng.throttle = Throttle::Fixed(true),
                KeyCode::S | KeyCode::Down => eng.throttle = Throttle::Fixed(false),
                KeyCode::A | KeyCode::Left => k_bod.rotate(Quat::from_rotation_z(drot)),
                KeyCode::D | KeyCode::Right => k_bod.rotate(Quat::from_rotation_z(-drot)),
                _ => {}
            }
        }
    })
}
