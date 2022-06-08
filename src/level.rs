use super::physics::KinimaticsBundle;
use bevy::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup_system);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// :COMPONENT: An astronomical body, such as a planet, moon, star, etc.
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct AstroObject {
    pub radius: f32,
}

/// :BUNDLE: Provided for convenience. Describes a generic astronomical body.
#[derive(Bundle, Default)]
pub struct AstroObjectBundle {
    pub astro_object: AstroObject,
    #[bundle]
    pub kinimatics_bundle: KinimaticsBundle,
}

/// Resource which contains the sprites used to represents various astronomical
/// bodies on the display.
#[derive(Clone)]
struct LevelSprites {
    generic_planet: SpriteBundle,
}

fn startup_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let sprite_resource = LevelSprites {
        generic_planet: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::new(0.75, 0.75, 0.0)),
            texture: asset_server.load("../assets/planet.png"),
            ..Default::default()
        },
    };

    commands.insert_resource(sprite_resource.clone());

    fn spawn_planet(
        mut commands: &mut Commands,
        sprite_resource: &LevelSprites,
        mass: f32,
        translation: Vec3,
        velocity: Vec3,
    ) {
        commands
            .spawn()
            .insert_bundle(AstroObjectBundle {
                kinimatics_bundle: KinimaticsBundle::build()
                    .insert_mass(mass)
                    .insert_translation(translation)
                    .insert_velocity(velocity),
                ..Default::default()
            })
            .with_children(|p| {
                p.spawn_bundle(sprite_resource.generic_planet.clone());
            });
    }

    //spawn_planet(&mut commands, &sprite_resource, 2e16, Vec3::new(100.0, 0.0, 0.0), Vec3::new(0.0, 40.0, 0.0));
    //spawn_planet(&mut commands, &sprite_resource, 2e16, Vec3::new(-100.0, 0.0, 0.0), Vec3::new(0.0, -40.0, 0.0));

    // the sun
    spawn_planet(&mut commands, &sprite_resource, 2e15, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));

    //// Mercury
    spawn_planet(&mut commands, &sprite_resource, 3.285e8, Vec3::new(0.0, 60.0, 0.0), Vec3::new(-47.9, 0.0, 0.0));
    //// Venus
    //spawn_planet(&mut commands, &sprite_resource, 4.867e24, Vec3::new(0.0, 100e9, 0.0), Vec3::new(0.0, 35.0e9, 0.0));
    //// Earth
    //spawn_planet(&mut commands, &sprite_resource, 5.972e24, Vec3::new(0.0, 150e9, 0.0), Vec3::new(0.0, 28.9e9, 0.0));
    //// Mars
    //spawn_planet(&mut commands, &sprite_resource, 6.39e23, Vec3::new(0.0, 220e9, 0.0), Vec3::new(0.0, 24.1e9, 0.0));
    //// Jupiter
    //spawn_planet(&mut commands, &sprite_resource, 8.898e27, Vec3::new(0.0, 780e9, 0.0), Vec3::new(0.0, 13.1e9, 0.0));
    //// Saturn
    //spawn_planet(&mut commands, &sprite_resource, 5.683e26, Vec3::new(0.0, 1.42e12, 0.0), Vec3::new(0.0, 9.7e9, 0.0));
}
