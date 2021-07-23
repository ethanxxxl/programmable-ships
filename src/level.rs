use super::physics::KinimaticsBundle;
use bevy::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(startup_system.system());
    }
}

/// :COMPONENT: An astronomical body, such as a planet, moon, star, etc.
#[derive(Default)]
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
    let planet_texture: Handle<Texture> = asset_server.load("../assets/planet.png");

    let planet_material = ColorMaterial {
        color: Color::rgb(0.0, 1.0, 0.0),
        texture: Some(planet_texture),
    };

    let planet_material = materials.add(planet_material);

    let sprite_resource = LevelSprites {
        generic_planet: SpriteBundle {
            sprite: Sprite::new(Vec2::new(20.0, 20.0)),
            material: planet_material.clone(),
            transform: Transform::from_scale(Vec3::new(0.75, 0.75, 0.0)),
            ..Default::default()
        },
    };

    commands.insert_resource(sprite_resource.clone());

    // Add a planet
    commands
        .spawn()
        .insert_bundle(AstroObjectBundle {
            kinimatics_bundle: KinimaticsBundle::build()
                .insert_mass(8e15)
                .insert_translation(Vec3::new(0.0, -100.0, 0.0))
                .insert_velocity(Vec3::new(40.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with_children(|p| {
            p.spawn_bundle(sprite_resource.generic_planet.clone());
        });

    // Add a planet
    commands
        .spawn()
        .insert_bundle(AstroObjectBundle {
            kinimatics_bundle: KinimaticsBundle::build()
                .insert_mass(8e15)
                .insert_translation(Vec3::new(0.0, 100.0, 0.0))
                .insert_velocity(Vec3::new(-40.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with_children(|p| {
            p.spawn_bundle(sprite_resource.generic_planet.clone());
        });
}
