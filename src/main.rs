use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb_u8(0,0,0)))
        .add_startup_system(setup_system.system())
        .add_system(kinimatics_system.system())
        .add_system(control_system.system())
        .run();
}

/* you want to create a little that can move around with the arrow keys.
 */

struct Velocity {
    v: Vec2,
}

struct Acceleration {
    a: f32,
}

struct Controlled;

#[derive(Bundle)]
struct ShipBundle {
    transform: Transform,
    vel: Velocity,
    accel: Acceleration,

    _global_transform: GlobalTransform,
}



fn setup_system(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: ResMut<AssetServer>) {
    let texture: Handle<Texture> = asset_server.load("../assets/ship_1.png");

    let material = ColorMaterial {
        color: Color::rgb(1.0,1.0,1.0),
        texture: Some(texture),
    };

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(ShipBundle {
        transform: Transform::from_xyz(0.0,0.0,0.0),
        vel: Velocity {v: Vec2::new(0.0, 0.0)},
        accel: Acceleration { a: 0.0 },
        _global_transform: GlobalTransform::from_xyz(0.0,0.0,0.0),
    })
    .insert(Controlled) // make this ship a controlle ship
    .with_children(|p| {
        p.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            material: materials.add(material),
            transform: Transform::from_scale(Vec3::new(1.8,2.0,0.0)),
            ..Default::default()
        });
    });
}

fn kinimatics_system(
    query: Query<(&mut Transform, &mut Velocity, &mut Acceleration)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    query.for_each_mut(|mut f| {
        // update the velocity with the acceleration
        let dv = f.0.rotation.mul_vec3(Vec3::Y) * dt * f.2.a;
        f.1.v += Vec2::new(dv.x, dv.y);
        println!("f.2.a: {}\nf.1.v.normalize(): {}, {}", f.2.a, f.1.v.normalize_or_zero().x, f.1.v.normalize_or_zero().y);

        // update the position with the velocity
        let dx = f.1.v * dt;
        f.0.translation += Vec3::new(dx.x, dx.y, 0.0);
        println!("pos: {}, {}", f.0.translation.x, f.0.translation.y);
    })
}

fn control_system(query: Query<(&mut Acceleration, &mut Transform), With<Controlled>>, input: Res<Input<KeyCode>>, time: Res<Time>) {
    let daccel: f32 = 100.0 * time.delta_seconds();
    let drot: f32 = std::f32::consts::PI * time.delta_seconds();

    query.for_each_mut(|q| {
        let mut a = q.0;
        let mut transform = q.1;

        if input.get_pressed().count() == 0 {
            a.a = 0.0;
        }

        for i in input.get_pressed() {
            match i {
                KeyCode::W | KeyCode::Up    => a.a += daccel,
                KeyCode::S | KeyCode::Down  => a.a -= daccel,
                KeyCode::A | KeyCode::Left  => transform.rotate(Quat::from_rotation_z(drot)),
                KeyCode::D | KeyCode::Right => transform.rotate(Quat::from_rotation_z(-drot)),
                _ => {},
            }
        }
    })
}
