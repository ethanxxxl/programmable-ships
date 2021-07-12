#![allow(dead_code)]
use bevy::{
    prelude::*,
    render::camera::{Camera, OrthographicProjection, CameraProjection},
    input::mouse::{MouseButton, MouseMotion, MouseWheel}
};
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::rgb_u8(0,0,0)))
        .add_startup_system(setup_system.system())
        .add_system(kinimatics_system.system())
        .add_system(user_control_system.system())
        .add_system(user_interface_system.system())
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

/**
 * an engine is either always on max burn,
 *  or is able to be throttled. the floating point must
 *  be on the range [0,1]. Values that fall outside this range
 *  do not have any phyisical meaning.
 */
#[derive(Inspectable)]
enum Throttle {
    Fixed(bool),
    Variable(f32),
}

impl Default for Throttle {
    fn default() -> Self {
        Self::Variable(0.0)
    }
}

#[derive(Inspectable, Default)]
struct Engine {
    fuel: f32,
    max_thrust: f32, // units of force
    throttle: Throttle,
}

#[derive(Default)]
struct Ship;

#[derive(Bundle, Default)]
struct ShipBundle {
    ship: Ship,
    engine: Engine,

    #[bundle]
    kinimatics_bundle: KinimaticsBundle,
}

#[derive(Default)]
struct Missile {
    target: Option<Entity>,
    blast_radius: f32,
}

#[derive(Bundle, Default)]
struct MissileBundle {
    missile: Missile,
    engine: Engine,

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

    let planet_material = materials.add(planet_material);

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Add a ship
    commands.spawn()
        .insert_bundle(ShipBundle {
            kinimatics_bundle: KinimaticsBundle::build().insert_mass(100.0).insert_translation(Vec3::new(500.0,500.0,0.0)),
            engine: Engine { max_thrust: 1000.0, ..Default::default() },
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

    // Add a planet
    commands.spawn()
        .insert_bundle(AstroObjectBundle {
            kinimatics_bundle: KinimaticsBundle::build()
                .insert_mass(8e15)
                .insert_translation(Vec3::new(0.0, -100.0, 0.0))
                .insert_velocity(Vec3::new(40.0, 0.0, 0.0)),
            .. Default::default()
        })
        .with_children(|p| {
            p.spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                material: planet_material.clone(),
                transform: Transform::from_scale(Vec3::new(1.0,1.0,0.0)),
                ..Default::default()
            });
        });

    // Add a planet
    commands.spawn()
        .insert_bundle(AstroObjectBundle {
            kinimatics_bundle: KinimaticsBundle::build()
                .insert_mass(8e15)
                .insert_translation(Vec3::new(0.0, 100.0, 0.0))
                .insert_velocity(Vec3::new(-40.0, 0.0, 0.0)),
            .. Default::default()
        })
        .with_children(|p| {
            p.spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                material: planet_material.clone(),
                transform: Transform::from_scale(Vec3::new(1.0,1.0,0.0)),
                ..Default::default()
            });
        });
}

fn kinimatics_system(mut k_bods: Query<(&mut Kinimatics, &mut Transform, Option<&Engine>)>,
                     time: Res<Time>) {

    // each element will have a corresponding entry in this list.
    let num_bods = k_bods.iter_mut().count();
    let mut all_forces: Vec<Vec<Vec3>> = Vec::new();
    all_forces.reserve(num_bods);

    // initialize a new vector for each k_bod
    for _ in 0..num_bods {
        all_forces.push(Vec::new());
    }

    let dt = time.delta_seconds();

    const GRAVITATIONAL_CONSTANT: f32 = 6.67430e-11;

    // ## Calculate forces from gravity
    let mut entities: Vec<(Mut<Kinimatics>, Mut<Transform>, Option<&Engine>)> = k_bods.iter_mut().collect();

    for (i, q) in entities.iter().enumerate() {
        // NOTE do I need to do bounds checking here?
        entities
            .split_at(i+1).1
            .iter()
            .enumerate()
            .for_each(|(j,o)| {
                // calculate magnitude of the force
                let force_mag = GRAVITATIONAL_CONSTANT * (q.0.mass * o.0.mass) /
                    q.1.translation.distance_squared(o.1.translation);

                // calculate direction and magnitude of the forces for each object.
                let d1 = (o.1.translation - q.1.translation).normalize() * force_mag;
                let d2 = (q.1.translation - o.1.translation).normalize() * force_mag;

                // add these forces to a list of forces
                all_forces[i].push(d1);
                all_forces[i+j+1].push(d2);
            });
    }

    // ## Calculate other forces and update kinimatics
    for (i, (kin, tran, engine)) in entities.iter_mut().enumerate() {
        // handle acceleration from ship engine
        if let Some(t) = engine {
            all_forces[i].push(
                tran.rotation.mul_vec3(Vec3::Y) * match t.throttle {
                    Throttle::Fixed(true) => t.max_thrust,
                    Throttle::Fixed(false) => 0.0,
                    Throttle::Variable(amount) => amount * t.max_thrust
                });
        }

        // add up forces, then apply them
        kin.acceleration = all_forces[i].iter().copied().reduce(|acc, x| acc + x ).expect("0 forces") / kin.mass;

        kin.velocity = kin.velocity + kin.acceleration*dt;
        tran.translation = tran.translation + kin.velocity*dt;
    }

}

fn user_control_system(query: Query<(&mut Ship, &mut Transform, &mut Engine), With<Controlled>>,
                       input: Res<Input<KeyCode>>,
                       time: Res<Time>) {
    let drot: f32 = std::f32::consts::PI * time.delta_seconds();

    query.for_each_mut(|(mut ship, mut k_bod, mut eng)| {
        if input.get_pressed().count() == 0 {
             eng.throttle = Throttle::Fixed(false);
        }

        for i in input.get_pressed() {
            match i {
                KeyCode::W | KeyCode::Up    => eng.throttle = Throttle::Fixed(true),
                KeyCode::S | KeyCode::Down  => eng.throttle = Throttle::Fixed(false),
                KeyCode::A | KeyCode::Left  => k_bod.rotate(Quat::from_rotation_z(drot)),
                KeyCode::D | KeyCode::Right => k_bod.rotate(Quat::from_rotation_z(-drot)),
                _ => {},
            }
        }
    })
}

fn user_interface_system(cam_query: Query<(&mut OrthographicProjection, &mut Transform, &mut Camera)>,
                         mouse_state: Res<Input<MouseButton>>,
                         mut motion_evr: EventReader<MouseMotion>,
                         mut wheel_evr: EventReader<MouseWheel> ) {
    // handle zooming when the user scrolls
    for event in wheel_evr.iter() {
        cam_query.for_each_mut(|(mut op, _t, mut c)| {
            op.far += (1000.0/(0.1*event.y)).abs();
            op.scale += -0.1*event.y;
            c.projection_matrix = op.get_projection_matrix();
        } );
    }

    // handle paning when the user middle clicks and drags
    cam_query.for_each_mut(|(op, mut t, _c)| {
        if mouse_state.pressed(MouseButton::Middle) {
            motion_evr.iter().for_each(|m| 
                t.translation += op.scale * Vec3::new(-m.delta.x, m.delta.y, 0.0)
            );
        }
    })
}
