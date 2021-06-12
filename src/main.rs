#![allow(dead_code)]
use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::rgb_u8(0,0,0)))
        .add_startup_system(setup_system.system())
        .add_system(kinimatics_system.system())
        .add_system(user_control_system.system())
        .run();
}


/// while the game is 2D, the z dimension in the velocity and
///  acceleration fields will be either ignored or 0.
///  if/when the game becomes 3D, this will provide an easy transition.
#[derive(Inspectable, Default)]
struct Kinimatics {
    // kinimatic stuff
    #[inspectable(label="v")]
    velocity: Vec3,
    #[inspectable(label="a")]
    acceleration: Vec3,
    
    #[inspectable(label="m")]
    mass: f32,
}

#[derive(Bundle, Default)]
struct KinimaticsBundle {
    transform: Transform,
    _global_transform: GlobalTransform,
    kinimatics: Kinimatics,
}

impl KinimaticsBundle {
    pub fn build() -> Self {
        Self::default()
    }
    
    pub fn insert_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }

    pub fn insert_kinimatics(mut self, k: Kinimatics) -> Self {
        self.kinimatics = k;
        self
    }

    pub fn insert_translation(mut self, t: Vec3) -> Self {
        self.transform.translation = t;
        self
    }

    pub fn insert_velocity(mut self, v: Vec3) -> Self {
        self.kinimatics.velocity = v;
        self
    }

    pub fn insert_acceleration(mut self, a: Vec3) -> Self {
        self.kinimatics.acceleration = a;
        self
    }

    pub fn insert_mass(mut self, m: f32) -> Self {
        self.kinimatics.mass = m;
        self
    }
}

struct Controlled;

#[derive(Default)]
struct Ship {
    // engines provide constant acceleration.
    engine_accel: f32,
}

#[derive(Bundle, Default)]
struct ShipBundle {
    ship: Ship,
    
    #[bundle]
    kinimatics_bundle: KinimaticsBundle,
}

#[derive(Default)]
struct AstroObject {
    radius: f32,
}

#[derive(Bundle, Default)]
struct AstroObjectBundle {
    astro_object: AstroObject,
    #[bundle]
    kinimatics_bundle: KinimaticsBundle,
}

fn setup_system(mut commands: Commands,
                mut materials: ResMut<Assets<ColorMaterial>>,
                asset_server: ResMut<AssetServer>) {
    let ship_texture: Handle<Texture> = asset_server.load("../assets/ship_1.png");
    let planet_texture: Handle<Texture> = asset_server.load("../assets/DustPlanet.png");

    let ship_material = ColorMaterial {
        color: Color::rgb(1.0,1.0,1.0),
        texture: Some(ship_texture),
    };
    
    let planet_material = ColorMaterial {
        color: Color::rgb(1.0,1.0,1.0),
        texture: Some(planet_texture),
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn()
        .insert_bundle(ShipBundle {
            kinimatics_bundle: KinimaticsBundle::build().insert_mass(100.0).insert_translation(Vec3::new(500.0,500.0,0.0)),
            ..Default::default()
        })
        .insert(Controlled {})
        .with_children(|p| {
            p.spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                material: materials.add(ship_material),
                transform: Transform::from_scale(Vec3::new(1.8,2.0,0.0)),
                ..Default::default()
            });
        });

    commands.spawn()
        .insert_bundle(AstroObjectBundle {
            kinimatics_bundle: KinimaticsBundle::build().insert_mass(8e14).insert_translation(Vec3::new(0.0, 0.0, 0.0)),
            .. Default::default()
        })
        .with_children(|p| {
            p.spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(200.0, 200.0)),
                material: materials.add(planet_material),
                transform: Transform::from_scale(Vec3::new(1.0,1.0,0.0)),
                ..Default::default()
            });
        });
}

fn kinimatics_system(mut query: Query<(&mut Kinimatics, &mut Transform, Option<&Ship>)>,
                     time: Res<Time>) {
    let dt = time.delta_seconds();

    const GRAVITATIONAL_CONSTANT: f32 = 6.67430e-11;

    let mut entities: Vec<(Mut<Kinimatics>, Mut<Transform>, Option<&Ship>)> = query.iter_mut().collect();

    let mut all_a: Vec<Vec3> = Vec::new();
    all_a.resize(entities.len(), Vec3::default());

    for (i, q) in entities.iter().enumerate() {
        // NOTE do I need to do bounds checking here?
        entities
            .split_at(i+1).1
            .iter()
            .enumerate()
            .for_each(|(j,o)| {
                // calculate magnitude of acceleration
                let a_mag = GRAVITATIONAL_CONSTANT * (q.0.mass * o.0.mass) /
                    q.1.translation.distance_squared(o.1.translation);

                // calculate direction of acceleration NOTE: IF THESE are the same, there will be
                // problems!!!
                let d1 = (o.1.translation - q.1.translation).normalize() * a_mag;
                let d2 = (q.1.translation - o.1.translation).normalize() * a_mag;
                all_a[i] = all_a[i] + d1;
                all_a[i+j+1] = all_a[i+j] + d2;
            });
    }

    for (i, q) in entities.iter_mut().enumerate() {
        q.0.acceleration = all_a[i];

        // handle acceleration from ship engine
        if let Some(ship) = q.2 {
            q.0.acceleration += q.1.rotation.mul_vec3(Vec3::Y) * ship.engine_accel;
        }

        q.0.velocity = q.0.velocity + q.0.acceleration*dt;
        q.1.translation =  q.1.translation + q.0.velocity*dt;
        //println!("i: {}, pos: {}", i, q.1.translation);
    }

}

fn user_control_system(query: Query<(&mut Ship, &mut Transform), With<Controlled>>,
                       input: Res<Input<KeyCode>>,
                       time: Res<Time>) {
    let daccel: f32 = 100.0 * time.delta_seconds();
    let drot: f32 = std::f32::consts::PI * time.delta_seconds();

    query.for_each_mut(|q| {
        let mut ship = q.0;
        let mut k_bod = q.1;

        if input.get_pressed().count() == 0 {
            ship.engine_accel = 0.0;
        }

        for i in input.get_pressed() {
            match i {
                KeyCode::W | KeyCode::Up    => ship.engine_accel += daccel,
                KeyCode::S | KeyCode::Down  => ship.engine_accel -= daccel,
                KeyCode::A | KeyCode::Left  => k_bod.rotate(Quat::from_rotation_z(drot)),
                KeyCode::D | KeyCode::Right => k_bod.rotate(Quat::from_rotation_z(-drot)),
                _ => {},
            }
        }
    })
}
