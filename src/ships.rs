use bevy::prelude::*;
use super::physics::{Kinimatics, KinimaticsBundle};

use bevy_inspector_egui::Inspectable;

pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_system(startup_system.system())
            .add_system(user_control_system.system());
    }
}

pub struct Controlled;

/// an engine is either always on max burn,
///      or is able to be throttled. the floating point must
///      be on the range [0,1]. Values that fall outside this range
///      do not have any phyisical meaning.
#[derive(Inspectable, Clone, Copy)]
pub enum Throttle {
    Fixed(bool),
    Variable(f32),
}

impl Default for Throttle {
    fn default() -> Self {
        Self::Variable(0.0)
    }
}

#[derive(Inspectable, Default, Clone, Copy)]
pub struct Engine {
    pub fuel: f32,
    pub max_thrust: f32, // units of force
    pub throttle: Throttle,
}

#[derive(Default)]
pub struct Ship;

#[derive(Bundle, Default)]
pub struct ShipBundle {
    pub ship: Ship,
    pub engine: Engine,

    #[bundle]
    pub kinimatics_bundle: KinimaticsBundle,
}

#[derive(Default)]
pub struct Missile {
    pub target: Option<Entity>,
    pub blast_radius: f32,
}

#[derive(Bundle, Default)]
pub struct MissileBundle {
    pub missile: Missile,
    pub engine: Engine,

    #[bundle]
    pub kinimatics_bundle: KinimaticsBundle,
}

#[derive(Clone)]
struct ShipSprites {
    generic_ship: SpriteBundle,
}

fn startup_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let ship_texture: Handle<Texture> = asset_server.load("../assets/ship_1.png");

    let ship_material = ColorMaterial {
        color: Color::rgb(1.0, 1.0, 1.0),
        texture: Some(ship_texture),
    };

    let ship_material = materials.add(ship_material);

    let sprite_resource = ShipSprites {
        generic_ship: SpriteBundle {
            sprite: Sprite::new(Vec2::new(20.0, 20.0)),
            material: ship_material.clone(),
            transform: Transform::from_scale(Vec3::new(0.75, 0.75, 0.0)),
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

fn user_control_system(
    query: Query<(&mut Ship, &mut Transform, &mut Engine), With<Controlled>>,
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
