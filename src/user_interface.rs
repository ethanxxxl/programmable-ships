use bevy::{
    input::mouse::{MouseButton, MouseMotion, MouseWheel},
    prelude::*,
    render::camera::{Camera, Camera2d, CameraProjection, OrthographicProjection},
    render::view::VisibleEntities,
};

use super::physics::Kinimatics;
use super::ships::{Engine, Throttle};

pub struct UserInterfacePlugin;

impl Plugin for UserInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup_system)
            .add_system(user_interface_system)
            .add_system(course_projection_system);
    }
}

/// :COMPONENT: Marker component
#[derive(Component)]
pub struct ProjectionDot;

/// :BUNDLE: Provided for convenience.
#[derive(Bundle)]
pub struct ProjectionDotBundle {
    pub projection_dot: ProjectionDot,

    #[bundle]
    pub sprite: SpriteBundle,
}

/// Resource which holds all the sprites that will be used in both the display and the UI.
pub struct UISprites {
    projection_dot: SpriteBundle,
}

fn startup_system(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let sprite_resource = UISprites {
        projection_dot: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(2.0, 2.0)),
                color: Color::rgb_u8(199, 199, 199),
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 0.0)),
            texture: asset_server.load("../assets/dot.png"),
            ..Default::default()
        },
    };

    commands.insert_resource(sprite_resource);
}

/// :SYSTEM: Allows the user to scroll, pan, and zoom the display.
///
/// Note: zooming does
/// not visually scale visible entities, because the display is more of a map than a camera.
/// because of the vast distances of outer space, sprites would be way to small to see if they
/// zoomed to scale.
fn user_interface_system(
    mut cam_query: Query<
        (
            &mut OrthographicProjection,
            &mut Transform,
            &mut Camera,
            &mut VisibleEntities,
        ),
        With<Camera2d>,
    >,
    mut transform_query: Query<&mut Transform, (With<Sprite>, Without<Camera>)>,
    mouse_state: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut wheel_evr: EventReader<MouseWheel>,
) {
    // handle zooming when the user scrolls
    for event in wheel_evr.iter() {
        for (mut ortho, _transform, mut camera, _entities) in cam_query.iter_mut() {
            const ZOOM_SPEED: f32 = 0.1;
            let scale_difference = (10.0 as f32).powf(event.y as f32 * ZOOM_SPEED);

            // adjust camera scaling
            ortho.scale *= scale_difference;
            camera.projection_matrix = ortho.get_projection_matrix();

            // scale visible entities
            for mut t in transform_query.iter_mut() {
                t.scale *= Vec3::ONE * scale_difference;
            }
        }
    }

    // handle paning when the user middle clicks and drags

    for (op, mut t, _c, _e) in cam_query.iter_mut() {
        if mouse_state.pressed(MouseButton::Middle) {
            motion_evr
                .iter()
                .for_each(|m| t.translation += op.scale * Vec3::new(-m.delta.x, m.delta.y, 0.0));
        }
    }
}

/// :SYSTEM: Projects the motion of all kinimatic bodies.
///
/// Currently, the projection is displayed by using a bunch of `ProjectionDot entities which
/// are moved to the entities projected locations. In the future, the plan is to transition to
/// a shader to display the dot.
pub fn course_projection_system(
    mut commands: Commands,
    k_bods: Query<(&Kinimatics, &Transform, Option<&Engine>), Without<ProjectionDot>>,
    mut dots: Query<(Entity, &mut Transform), With<ProjectionDot>>,
    sprites: Res<UISprites>,
) {
    // make a copy of all the entities
    let entities: Vec<(Kinimatics, Transform, Option<Engine>)> = k_bods
        .iter()
        .map(|(kinimatics, transform, engine)| {
            if let Some(e) = engine {
                return (kinimatics.clone(), transform.clone(), Some(e.clone()));
            } else {
                return (kinimatics.clone(), transform.clone(), None);
            }
        })
        .collect();

    let num_seconds = 1; // number of seconds to look ahead
    let step_precision = 5; // steps/second

    let mut steps: Vec<Vec<(Kinimatics, Transform, Option<Engine>)>> = Vec::new();
    steps.reserve(num_seconds * step_precision);

    let mut forces: Vec<Vec3> = Vec::new();
    forces.reserve(entities.len());
    for _ in 0..entities.len() {
        forces.push(Vec3::ZERO);
    }

    // initial state
    steps.push(entities.clone());

    // account for force due to gravity
    const GRAVITATIONAL_CONSTANT: f32 = 6.67430e-11;
    let dt = 1.0 / (step_precision as f32);
    for step in 1..num_seconds * step_precision {
        steps.push(steps[step - 1].clone());

        // calculate forces for each body
        for (i, bod1) in steps[step].iter().enumerate() {
            let (k1, t1, engine) = bod1;

            // add forces due to gravity
            steps[step]
                .split_at(i + 1)
                .1
                .iter()
                .enumerate()
                .for_each(|(j, bod2)| {
                    let (k2, t2, _) = bod2;

                    // calculate magnitude of the force
                    let force_mag = GRAVITATIONAL_CONSTANT * (k1.mass * k2.mass)
                        / t1.translation.distance_squared(t2.translation);

                    // calculate direction and magnitude of the forces for each object.
                    let d1 = (t2.translation - t1.translation).normalize() * force_mag;
                    let d2 = (t1.translation - t2.translation).normalize() * force_mag;

                    forces[i] += d1;
                    forces[i + j + 1] += d2;
                });

            // handle force from ship engine
            if let Some(t) = engine {
                forces[i] += t1.rotation.mul_vec3(Vec3::Y)
                    * match t.throttle {
                        Throttle::Fixed(true) => t.max_thrust,
                        Throttle::Fixed(false) => 0.0,
                        Throttle::Variable(amount) => amount * t.max_thrust,
                    };
            }
        }

        // update kinimatics
        steps[step]
            .iter_mut()
            .enumerate()
            .for_each(|(j, (kin, trans, _))| {
                kin.acceleration = forces[j] / kin.mass;
                kin.velocity = kin.velocity + kin.acceleration * dt;
                trans.translation = trans.translation + kin.velocity * dt;
            });

        forces.clear();
        for _ in 0..entities.len() {
            forces.push(Vec3::ZERO);
        }
    }

    // total number of dots needed for projection
    let total_dots = steps.len() * entities.len();
    let available_dots = dots.iter_mut().count();

    if available_dots > total_dots {
        // remove extra dots
        let mut dots = dots.iter_mut();
        for _ in 0..(available_dots - total_dots) {
            if let Some(d) = dots.next() {
                commands.entity(d.0).despawn();
            }
        }
    } else if available_dots < total_dots {
        // spawn in missing dots
        for _ in 0..(total_dots - available_dots) {
            commands
                .spawn()
                .insert(ProjectionDot {})
                .insert(Transform::default())
                .insert(GlobalTransform::default())
                .with_children(|p| {
                    p.spawn_bundle(sprites.projection_dot.clone());
                });
        }
    }

    let steps: Vec<Transform> = steps.into_iter().flatten().map(|k_bod| k_bod.1).collect();

    for (i, (_, mut transform)) in dots.iter_mut().enumerate() {
        *transform = steps[i];
    }
}

/// Temporary init function.
///
/// Soon™ this will be unified into normal [startup_system()] system. Currently,
/// this builds the UI.
/*
pub fn init_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    //button_materials: Res<ButtonStyle>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    let default_button = ButtonStyle {
        material_normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
        material_hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        material_pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        style: Style {
            size: Size::new(Val::Px(100.0), Val::Px(65.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        text_style: TextStyle {
            font: asset_server.load("/usr/share/fonts/gnu-free/FreeSans.otf"),
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    };

    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexStart,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            // Bottom Tray (border)
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(15.0)),
                        border: Rect::all(Val::Px(2.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb(0.65, 0.65, 0.65).into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Bottom Tray Fill
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                justify_content: JustifyContent::SpaceAround,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            material: materials.add(Color::rgb_u8(57, 67, 74).into()),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            create_button(&mut parent.spawn(), &default_button).insert(
                                CourseProjectionButton {
                                    is_on: false,
                                    style: default_button.clone(),
                                },
                            );
                        });
                });
        });
}
*/

/// :COMPONENT: Material Handles for different button states.
///
/// This describes the bare bones style of a button or group of buttons.
#[derive(Clone, Component)]
pub struct ButtonStyle {
    material_normal: Handle<ColorMaterial>,
    material_hovered: Handle<ColorMaterial>,
    material_pressed: Handle<ColorMaterial>,
    style: Style,
    text_style: TextStyle,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct CourseProjectionButton {
    is_on: bool,
    style: ButtonStyle,
}

// example button with functionality. this button toggles on/off the course projection.
// input parameters: this function will need a list of all objects with paths to predict
#[allow(dead_code)]
pub fn course_projection_system_button(
    mut button_query: Query<
        (
            &CourseProjectionButton,
            &Interaction,
            &mut Handle<ColorMaterial>,
        ),
        Changed<Interaction>,
    >,
) {
    for (state, interaction, mut material) in button_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *material = state.style.material_pressed.clone();
            }
            Interaction::Hovered => {
                *material = state.style.material_hovered.clone();
            }
            Interaction::None => {
                *material = state.style.material_normal.clone();
            }
        }
    }
}

/// Handles the state transition (think colors) of buttons
pub fn button_system(
    button_materials: Res<ButtonStyle>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *material = button_materials.material_pressed.clone();
            }
            Interaction::Hovered => {
                *material = button_materials.material_hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.material_normal.clone();
            }
        }
    }
}

/// Helper function to easily create buttons.
use bevy::ecs::system::EntityCommands;
/*
fn create_button<'a, 'b, 'c>(
    parent: &'c mut EntityCommands<'a, 'b>,
    style: &ButtonStyle,
) -> &'c mut EntityCommands<'a, 'b> {
    parent
        .insert_bundle(ButtonBundle {
            style: style.style.clone(),
            material: style.material_normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section("", style.text_style.clone(), Default::default()),
                ..Default::default()
            });
        })
}
*/
