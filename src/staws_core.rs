pub mod components {
    use bevy::prelude::*;
    use bevy_inspector_egui::Inspectable;

    /// while the game is 2D, the z dimension in the velocity and
    ///  acceleration fields will be either ignored or 0.
    ///  if/when the game becomes 3D, this will provide an easy transition.
    #[derive(Inspectable, Default)]
    pub struct Kinimatics {
        // kinimatic stuff
        #[inspectable(label = "v")]
        pub velocity: Vec3,
        #[inspectable(label = "a")]
        pub acceleration: Vec3,

        #[inspectable(label = "m")]
        pub mass: f32,
    }

    #[derive(Bundle, Default)]
    pub struct KinimaticsBundle {
        pub transform: Transform,
        pub _global_transform: GlobalTransform,
        pub kinimatics: Kinimatics,
    }

    impl KinimaticsBundle {
        pub fn build() -> Self {
            Self::default()
        }

        #[allow(dead_code)]
        pub fn insert_transform(mut self, t: Transform) -> Self {
            self.transform = t;
            self
        }

        #[allow(dead_code)]
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

    pub struct Controlled;

    /**
     * an engine is either always on max burn,
     *  or is able to be throttled. the floating point must
     *  be on the range [0,1]. Values that fall outside this range
     *  do not have any phyisical meaning.
     */
    #[derive(Inspectable)]
    pub enum Throttle {
        Fixed(bool),
        Variable(f32),
    }

    impl Default for Throttle {
        fn default() -> Self {
            Self::Variable(0.0)
        }
    }

    #[derive(Inspectable, Default)]
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

    #[derive(Default)]
    pub struct AstroObject {
        pub radius: f32,
    }

    #[derive(Bundle, Default)]
    pub struct AstroObjectBundle {
        pub astro_object: AstroObject,
        #[bundle]
        pub kinimatics_bundle: KinimaticsBundle,
    }
}

pub mod systems {
    use bevy::{
        input::mouse::{MouseButton, MouseMotion, MouseWheel},
        prelude::*,
        render::camera::{Camera, CameraProjection, OrthographicProjection},
    };
    //use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};

    use super::components::*;

    pub fn setup_system(
        mut commands: Commands,
        mut materials: ResMut<Assets<ColorMaterial>>,
        asset_server: ResMut<AssetServer>,
    ) {
        let ship_texture: Handle<Texture> = asset_server.load("../assets/ship_1.png");
        let planet_texture: Handle<Texture> = asset_server.load("../assets/DustPlanet.png");

        let ship_material = ColorMaterial {
            color: Color::rgb(1.0, 1.0, 1.0),
            texture: Some(ship_texture),
        };

        let planet_material = ColorMaterial {
            color: Color::rgb(1.0, 1.0, 1.0),
            texture: Some(planet_texture),
        };

        let planet_material = materials.add(planet_material);

        commands.spawn_bundle(OrthographicCameraBundle::new_2d());

        // Add a ship
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
                p.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                    material: materials.add(ship_material),
                    transform: Transform::from_scale(Vec3::new(1.8, 2.0, 0.0)),
                    ..Default::default()
                });
            });

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
                p.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                    material: planet_material.clone(),
                    transform: Transform::from_scale(Vec3::new(1.0, 1.0, 0.0)),
                    ..Default::default()
                });
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
                p.spawn_bundle(SpriteBundle {
                    sprite: Sprite::new(Vec2::new(20.0, 20.0)),
                    material: planet_material.clone(),
                    transform: Transform::from_scale(Vec3::new(1.0, 1.0, 0.0)),
                    ..Default::default()
                });
            });
    }

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

        // ## Calculate forces from gravity
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

    pub fn user_control_system(
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

    pub fn user_interface_system(
        cam_query: Query<(&mut OrthographicProjection, &mut Transform, &mut Camera)>,
        mouse_state: Res<Input<MouseButton>>,
        mut motion_evr: EventReader<MouseMotion>,
        mut wheel_evr: EventReader<MouseWheel>,
    ) {
        // handle zooming when the user scrolls
        for event in wheel_evr.iter() {
            cam_query.for_each_mut(|(mut op, _t, mut c)| {
                // make sure we are using the right camera
                if c.name != Some(bevy::render::render_graph::base::camera::CAMERA_2D.to_string()) {
                    return;
                }

                op.far += (1000.0 / (0.1 * event.y)).abs();
                op.scale += -0.1 * event.y;
                c.projection_matrix = op.get_projection_matrix();
            });
        }

        // handle paning when the user middle clicks and drags
        cam_query.for_each_mut(|(op, mut t, c)| {
            // make sure we are using the right camera
            if c.name != Some(bevy::render::render_graph::base::camera::CAMERA_2D.to_string()) {
                return;
            }

            if mouse_state.pressed(MouseButton::Middle) {
                motion_evr.iter().for_each(|m| {
                    t.translation += op.scale * Vec3::new(-m.delta.x, m.delta.y, 0.0)
                });
            }
        })
    }
}

pub mod user_interface {
    use bevy::prelude::*;

    // bare bones style of a button or group of buttons.
    //  text just defines the font size, color, etc. of the
    //  text. the  actual text should be filled in later.
    #[derive(Clone)]
    pub struct ButtonStyle {
        material_normal: Handle<ColorMaterial>,
        material_hovered: Handle<ColorMaterial>,
        material_pressed: Handle<ColorMaterial>,
        style: Style,
        text_style: TextStyle,
    }

    #[allow(dead_code)]
    pub struct CourseProjectionButton {
        is_on: bool,
        style: ButtonStyle,
    }

    // example button with functionality. this button toggles on/off the course projection.
    // input parameters: this function will need a list of all objects with paths to predict
    #[allow(dead_code)]
    pub fn course_projection_system(
        mut button_query: Query<(&CourseProjectionButton, &Interaction, &mut Handle<ColorMaterial>), Changed<Interaction>>,
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
    
    use bevy::ecs::system::EntityCommands;
    fn create_button<'a, 'b, 'c>(parent: &'c mut EntityCommands<'a, 'b>, style: &ButtonStyle) -> &'c mut EntityCommands<'a,'b> {
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

                                create_button(&mut parent.spawn(), &default_button)
                                    .insert(CourseProjectionButton { is_on: false, style: default_button.clone() });
                            });
                    });
            });
    }
}
