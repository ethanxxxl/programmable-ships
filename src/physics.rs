use super::ships::{Engine, Throttle};
use bevy::{prelude::*, render::render_resource::AsBindGroupShaderType};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(kinimatics_system);
    }
}

/// :COMPONENT: For entities that abide by the laws of ~~physics~~ my choosing.
/// Note, currently this is a 2D game, therefore the Z field is not to be used.
/// A future version of the game might open up a third dimension.
#[derive(Reflect, Default, Clone, Copy, Component)]
#[reflect(Component)]
pub struct Kinimatics {
    //#[inspectable(label = "v")]
    pub velocity: Vec3,
    //#[inspectable(label = "a")]
    pub acceleration: Vec3,

    //#[inspectable(label = "m")]
    pub mass: f32,
}

/// :BUNDLE: Provided for convenience. the Kinimatics component doesn't track
/// the transform of the entity, so this bundle should be used when creating
/// a new entity.
#[derive(Bundle, Default)]
pub struct KinimaticsBundle {
    pub kinimatics: Kinimatics,
    #[bundle]
    pub spatial: SpatialBundle,
}

impl KinimaticsBundle {
    pub fn build() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn insert_transform(mut self, t: Transform) -> Self {
        self.spatial.transform = t;
        self
    }

    #[allow(dead_code)]
    pub fn insert_kinimatics(mut self, k: Kinimatics) -> Self {
        self.kinimatics = k;
        self
    }

    pub fn insert_translation(mut self, t: Vec3) -> Self {
        self.spatial.transform.translation = t;
        self
    }

    pub fn insert_velocity(mut self, v: Vec3) -> Self {
        self.kinimatics.velocity = v;
        self
    }

    #[allow(dead_code)]
    pub fn insert_acceleration(mut self, a: Vec3) -> Self {
        self.kinimatics.acceleration = a;
        self
    }

    pub fn insert_mass(mut self, m: f32) -> Self {
        self.kinimatics.mass = m;
        self
    }
}

/// :SYSTEM: Iterates through all of the kinimatic entities, and simulates physics
/// on them, updating their transforms when it is done.
pub fn kinimatics_system(
    mut k_bods: Query<(&mut Kinimatics, &mut Transform, Option<&Engine>)>,
    time: Res<Time>,
) {
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

    //  Calculate forces from gravity
    let mut entities: Vec<(Mut<Kinimatics>, Mut<Transform>, Option<&Engine>)> =
        k_bods.iter_mut().collect();

    for (i, q) in entities.iter().enumerate() {
        // NOTE do I need to do bounds checking here?
        entities
            .split_at(i + 1)
            .1
            .iter()
            .enumerate()
            .for_each(|(j, o)| {
                // calculate magnitude of the force
                let force_mag = GRAVITATIONAL_CONSTANT * (q.0.mass * o.0.mass)
                    / q.1.translation.distance_squared(o.1.translation);

                // calculate direction and magnitude of the forces for each object.
                let d1 = (o.1.translation - q.1.translation).normalize() * force_mag;
                let d2 = (q.1.translation - o.1.translation).normalize() * force_mag;

                // add these forces to a list of forces
                all_forces[i].push(d1);
                all_forces[i + j + 1].push(d2);
            });
    }

    // ## Calculate other forces and update kinimatics
    for (i, (kin, tran, engine)) in entities.iter_mut().enumerate() {
        // handle acceleration from ship engine
        if let Some(t) = engine {
            all_forces[i].push(
                tran.rotation.mul_vec3(Vec3::Y)
                    * match t.throttle {
                        Throttle::Fixed(true) => t.max_thrust,
                        Throttle::Fixed(false) => 0.0,
                        Throttle::Variable(amount) => amount * t.max_thrust,
                    },
            );
        }

        // add up forces, then apply them
        kin.acceleration = all_forces[i]
            .iter()
            .copied()
            .reduce(|acc, x| acc + x)
            .expect("0 forces")
            / kin.mass;

        kin.velocity = kin.velocity + kin.acceleration * dt;
        tran.translation = tran.translation + kin.velocity * dt;
    }
}
