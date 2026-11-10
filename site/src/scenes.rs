//! Les deux scènes 3D du site, des jeux de `paquets::ecs` (le moteur d'Active) avec sa
//! physique : tout objet a une masse, la gravité et les collisions.
//!
//! - [`hero`] : six cartes d'idées tournent en anneau autour d'un jeton de vote ; des
//!   jetons « oui » et « non » tombent, rebondissent et s'empilent sur le plateau ; un
//!   appui en lâche un de plus, glisser tourne la caméra.
//! - [`ballot`] : le résultat d'un vote en urnes : une bille par « oui » tombe dans l'urne
//!   de son idée, la plus remplie gagne et reçoit sa couronne.
//!
//! Tout, sauf le dessin, tourne aussi sur le serveur : les tests font avancer les
//! scènes image par image et vérifient où sont les objets.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use paquets::ecs::prelude::*;

use crate::content::IDEAS;

/// Un générateur de nombres (xorshift64*), assez pour choisir des couleurs et des places.
struct Dice(u64);

impl Dice {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Un nombre entre -1 et 1.
    fn signed(&mut self) -> f32 {
        (self.next() % 2001) as f32 / 1000.0 - 1.0
    }
}

paquets::active::resource!(Dice);

/// Une couleur d'idée, depuis sa teinte.
fn idea_color(hue: u16, lightness: f32) -> Color {
    Color::hsl(f32::from(hue), 0.78, lightness)
}

/// La lumière d'une idée, plus ou moins forte.
fn glow(hue: u16, strength: f32) -> LinearRgba {
    let c = idea_color(hue, 0.5).to_linear();
    LinearRgba::rgb(c.red * strength, c.green * strength, c.blue * strength)
}

fn camera_clear() -> Camera {
    Camera {
        clear_color: ClearColorConfig::None,
        ..default()
    }
}

// ----- La scène d'accueil --------------------------------------------------------------

pub mod hero {
    use super::*;

    /// Ce qui tourne sur soi et flotte.
    pub struct Floating {
        pub speed: f32,
        pub phase: f32,
        pub base: f32,
    }

    /// Un jeton de vote tombé.
    pub struct Token;

    /// Le jeton du centre.
    pub struct Core;

    paquets::active::component!(Floating, Token, Core);

    /// Les jetons à lâcher : leurs formes et leurs matières.
    pub struct Tokens {
        meshes: Vec<Handle<Mesh>>,
        yes: Handle<StandardMaterial>,
        no: Handle<StandardMaterial>,
        /// Les jetons lâchés tout seuls au début.
        pub auto: usize,
        next_drop: f32,
    }

    /// La caméra et le glissement en cours.
    #[derive(Default)]
    pub struct Orbit {
        pub angle: f32,
        dragged: f32,
        pub still: bool,
    }

    paquets::active::resource!(Tokens, Orbit);

    pub const RADIUS: f32 = 11.0;
    pub const MAX_TOKENS: usize = 36;
    pub const AUTO_TOKENS: usize = 14;
    /// La hauteur du jeton du centre, au-dessus de la pile.
    pub const CORE_Y: f32 = 2.5;

    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // Le plateau : plat, il reste où il est.
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(4.6))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba_u8(40, 26, 74, 220),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.25,
                metallic: 0.4,
                ..default()
            })),
            Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        ));
        // Le rebord du plateau, qui garde les jetons.
        commands.spawn((
            Mesh3d(meshes.add(Torus::new(4.45, 4.85))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(166, 107, 255),
                emissive: LinearRgba::rgb(0.35, 0.12, 0.7),
                perceptual_roughness: 0.25,
                metallic: 0.6,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.2, 0.0),
            RigidBody::Static,
        ));
        // L'anneau : six cartes, enfants d'un parent qui tourne (elles poussent les jetons).
        let card = meshes.add(Cuboid::new(1.25, 1.75, 0.08));
        commands
            .spawn((
                Transform::default(),
                Floating {
                    speed: 0.22,
                    phase: 0.0,
                    base: 0.0,
                },
            ))
            .with_children(|ring| {
                for (i, idea) in IDEAS.iter().enumerate() {
                    let a = i as f32 / IDEAS.len() as f32 * TAU;
                    let base = 1.8 + 0.25 * (i % 2) as f32;
                    let color = idea_color(idea.hue, 0.62);
                    ring.spawn((
                        Mesh3d(card),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: color,
                            emissive: glow(idea.hue, 0.35),
                            perceptual_roughness: 0.3,
                            metallic: 0.15,
                            ..default()
                        })),
                        Transform::from_xyz(3.3 * a.cos(), base, 3.3 * a.sin()).with_rotation(
                            Quat::from_rotation_y(FRAC_PI_2 - a) * Quat::from_rotation_z(0.12),
                        ),
                        Floating {
                            speed: 0.0,
                            phase: a,
                            base,
                        },
                    ));
                }
            });
        // Le jeton du centre et son anneau, qui flottent sans tomber.
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.78).mesh().uv(48, 24))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 255, 255),
                emissive: LinearRgba::rgb(0.25, 0.12, 0.5),
                perceptual_roughness: 0.12,
                metallic: 0.65,
                ..default()
            })),
            Transform::from_xyz(0.0, CORE_Y, 0.0),
            RigidBody::Kinematic,
            Core,
        ));
        commands.spawn((
            Mesh3d(meshes.add(Torus::new(1.1, 1.28))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(62, 224, 208),
                emissive: LinearRgba::rgb(0.05, 0.4, 0.35),
                perceptual_roughness: 0.2,
                metallic: 0.8,
                ..default()
            })),
            Transform::from_xyz(0.0, CORE_Y, 0.0).with_rotation(Quat::from_rotation_x(1.1)),
            RigidBody::Kinematic,
            Floating {
                speed: 0.9,
                phase: 1.0,
                base: CORE_Y,
            },
        ));
        // La lumière : un corail chaud, un bleu froid, et un soleil.
        commands.spawn((
            PointLight {
                color: Color::srgb_u8(255, 107, 107),
                intensity: 2_600_000.0,
                range: 30.0,
                ..default()
            },
            Transform::from_xyz(4.0, 6.0, 3.0),
        ));
        commands.spawn((
            PointLight {
                color: Color::srgb_u8(110, 140, 255),
                intensity: 1_800_000.0,
                range: 30.0,
                ..default()
            },
            Transform::from_xyz(-5.0, 3.0, -4.0),
        ));
        commands.spawn((
            DirectionalLight {
                illuminance: 1_800.0,
                ..default()
            },
            Transform::from_xyz(1.0, 4.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        commands.spawn((
            Camera3d::default(),
            camera_clear(),
            Transform::from_xyz(0.0, 5.0, RADIUS).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y),
        ));
        commands.insert_resource(Tokens {
            meshes: vec![
                meshes.add(Sphere::new(0.32).mesh().uv(24, 12)),
                meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
                meshes.add(Cylinder::new(0.34, 0.16)),
            ],
            yes: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(62, 224, 160),
                perceptual_roughness: 0.3,
                metallic: 0.3,
                ..default()
            }),
            no: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 107, 129),
                perceptual_roughness: 0.3,
                metallic: 0.3,
                ..default()
            }),
            auto: AUTO_TOKENS,
            next_drop: 0.4,
        });
        commands.insert_resource(Dice(0x9E37_79B9_7F4A_7C15));
    }

    fn float(time: Res<Time>, orbit: Res<Orbit>, mut query: Query<(&mut Transform, &Floating)>) {
        if orbit.still {
            return;
        }
        let t = time.elapsed_secs();
        for (mut transform, floating) in &mut query {
            if floating.speed != 0.0 {
                transform.rotate_y(floating.speed * time.delta_secs());
            }
            if floating.base > 0.0 {
                transform.translation.y = floating.base + 0.22 * (t * 1.4 + floating.phase).sin();
            }
        }
    }

    /// Lâche un jeton au-dessus de `(x, z)`.
    fn drop_token(commands: &mut Commands, tokens: &Tokens, dice: &mut Dice, x: f32, z: f32) {
        let mesh = tokens.meshes[(dice.next() % tokens.meshes.len() as u64) as usize];
        let material = if dice.next().is_multiple_of(3) {
            tokens.no
        } else {
            tokens.yes
        };
        let tilt = dice.signed() * 0.6;
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(x, 6.5, z).with_rotation(
                Quat::from_rotation_y(dice.signed() * PI) * Quat::from_rotation_x(tilt),
            ),
            AngularVelocity(Vec3::new(tilt * 4.0, 0.0, -tilt * 3.0)),
            Restitution(0.35),
            Friction(0.8),
            RollingFriction(0.25),
            AngularDamping(1.2),
            Token,
        ));
    }

    /// Au début, les jetons tombent tout seuls, un par un.
    fn rain(
        mut commands: Commands,
        time: Res<Time>,
        mut tokens: ResMut<Tokens>,
        mut dice: ResMut<Dice>,
    ) {
        if tokens.auto == 0 || time.elapsed_secs() < tokens.next_drop {
            return;
        }
        tokens.auto -= 1;
        tokens.next_drop = time.elapsed_secs() + 0.45;
        let (r, a) = (1.7 + dice.signed().abs() * 0.9, dice.signed() * PI);
        drop_token(&mut commands, &tokens, &mut dice, r * a.cos(), r * a.sin());
    }

    /// Glisser tourne la caméra ; un appui sans glisser lâche un jeton là où il pointe.
    #[allow(clippy::too_many_arguments)]
    fn pointer(
        mut commands: Commands,
        buttons: Res<ButtonInput<MouseButton>>,
        motion: Res<AccumulatedMouseMotion>,
        time: Res<Time>,
        window: Single<&Window, With<PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform, &mut Transform)>,
        mut orbit: ResMut<Orbit>,
        tokens: Res<Tokens>,
        mut dice: ResMut<Dice>,
        dropped: Query<Entity, With<Token>>,
    ) {
        let (camera, at, mut transform) = camera.into_inner();
        if buttons.just_pressed(MouseButton::Left) {
            orbit.dragged = 0.0;
        }
        if buttons.pressed(MouseButton::Left) {
            orbit.dragged += motion.delta.length();
            orbit.angle -= motion.delta.x * 0.008;
        } else if !orbit.still {
            orbit.angle += 0.1 * time.delta_secs();
        }
        if buttons.just_released(MouseButton::Left) && orbit.dragged < 4.0 {
            let target =
                window
                    .cursor_position()
                    .and_then(|cursor| camera.viewport_to_world(&at, cursor).ok())
                    .and_then(|ray| {
                        Some(ray.get_point(
                            ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?,
                        ))
                    })
                    .filter(|p| p.length() < 4.4);
            if let Some(point) = target {
                // Trop de jetons : le plus ancien s'en va.
                if dropped.count() >= MAX_TOKENS
                    && let Some(oldest) = dropped.iter().next()
                {
                    commands.entity(oldest).despawn();
                }
                drop_token(&mut commands, &tokens, &mut dice, point.x, point.z);
            }
        }
        let eye = Vec3::new(RADIUS * orbit.angle.sin(), 5.0, RADIUS * orbit.angle.cos());
        *transform = Transform::from_translation(eye).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y);
    }

    /// Les jetons tombés du plateau s'en vont.
    fn fallen(mut commands: Commands, tokens: Query<(Entity, &Transform), With<Token>>) {
        for (entity, transform) in &tokens {
            if transform.translation.y < -8.0 {
                commands.entity(entity).despawn();
            }
        }
    }

    /// La scène, sans fenêtre : `still` fige l'anneau (mouvements réduits).
    pub fn app(still: bool) -> App {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .insert_resource(Orbit {
                still,
                ..Orbit::default()
            })
            .add_systems(Startup, setup)
            .add_systems(Update, (float, rain, pointer, fallen));
        app
    }
}

// ----- Le résultat en urnes ------------------------------------------------------------

pub mod ballot {
    use super::*;

    /// Les « oui » de chaque urne (les idées 1 à 4 de la démo).
    pub const VOTES: [usize; 4] = [5, 9, 6, 3];
    /// L'idée de chaque urne, dans [`IDEAS`].
    pub const IDEA_OF: [usize; 4] = [0, 1, 2, 3];
    /// La largeur d'une urne.
    pub const WIDTH: f32 = 1.9;
    pub const BALL: f32 = 0.27;
    /// La hauteur des cloisons.
    pub const HEIGHT: f32 = 3.0;

    /// Une bille de vote, et son urne.
    pub struct Ball(pub usize);

    /// La couronne du gagnant.
    pub struct Crown;

    paquets::active::component!(Ball, Crown);

    /// Ce qu'il reste à lâcher, et quand.
    pub struct Drops {
        /// Les urnes, dans l'ordre où les billes tombent.
        queue: Vec<usize>,
        next: f32,
        pub dropped: usize,
    }

    /// Dit à l'îlot combien de billes sont tombées.
    pub struct Counter(pub Option<paquets::active::State<usize>>);

    paquets::active::resource!(Drops, Counter);

    /// Le centre de l'urne `bin`.
    pub fn center(bin: usize) -> f32 {
        (bin as f32 - (VOTES.len() as f32 - 1.0) / 2.0) * WIDTH
    }

    /// L'urne gagnante.
    pub fn winner() -> usize {
        (0..VOTES.len()).max_by_key(|&i| VOTES[i]).unwrap_or(0)
    }

    /// Les billes, une urne après l'autre, comme des votes qui arrivent.
    fn queue() -> Vec<usize> {
        let mut left = VOTES;
        let mut queue = Vec::new();
        while left.iter().any(|&n| n > 0) {
            for (bin, n) in left.iter_mut().enumerate() {
                if *n > 0 {
                    *n -= 1;
                    queue.push(bin);
                }
            }
        }
        queue
    }

    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let n = VOTES.len();
        let total = WIDTH * n as f32;
        let (height, depth) = (HEIGHT, 1.25);
        // Le sol.
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(total + 2.0, 4.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(30, 20, 58),
                perceptual_roughness: 0.6,
                ..default()
            })),
        ));
        let glass = materials.add(StandardMaterial {
            base_color: Color::srgba_u8(200, 190, 255, 46),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.05,
            metallic: 0.1,
            ..default()
        });
        let edge = materials.add(StandardMaterial {
            base_color: Color::srgb_u8(166, 107, 255),
            emissive: LinearRgba::rgb(0.25, 0.1, 0.55),
            perceptual_roughness: 0.3,
            ..default()
        });
        // Les cloisons entre les urnes, et les vitres devant et derrière.
        let wall = meshes.add(Cuboid::new(0.08, height, depth));
        for i in 0..=n {
            let x = (i as f32 - n as f32 / 2.0) * WIDTH;
            commands.spawn((
                Mesh3d(wall),
                MeshMaterial3d(edge),
                Transform::from_xyz(x, height / 2.0, 0.0),
                RigidBody::Static,
            ));
        }
        let pane = meshes.add(Cuboid::new(total, height, 0.04));
        for z in [-depth / 2.0, depth / 2.0] {
            commands.spawn((
                Mesh3d(pane),
                MeshMaterial3d(glass),
                Transform::from_xyz(0.0, height / 2.0, z),
                RigidBody::Static,
            ));
        }
        // La couronne, cachée jusqu'au résultat.
        commands.spawn((
            Mesh3d(meshes.add(Torus::new(0.32, 0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb_u8(255, 200, 61),
                emissive: LinearRgba::rgb(0.6, 0.4, 0.05),
                metallic: 0.9,
                perceptual_roughness: 0.2,
                ..default()
            })),
            Transform::from_xyz(center(winner()), height + 0.9, 0.0),
            RigidBody::Kinematic,
            Visibility::Hidden,
            Crown,
        ));
        commands.spawn((
            PointLight {
                color: Color::srgb_u8(255, 180, 140),
                intensity: 2_000_000.0,
                range: 30.0,
                ..default()
            },
            Transform::from_xyz(3.0, 6.0, 5.0),
        ));
        commands.spawn((
            DirectionalLight {
                illuminance: 2_200.0,
                ..default()
            },
            Transform::from_xyz(-2.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        commands.spawn((
            Camera3d::default(),
            camera_clear(),
            Transform::from_xyz(0.0, 3.0, 7.4).looking_at(Vec3::new(0.0, 1.45, 0.0), Vec3::Y),
        ));
        commands.insert_resource(Drops {
            queue: queue(),
            next: 0.3,
            dropped: 0,
        });
        commands.insert_resource(Dice(0xD1B5_4A32_D192_ED03));
    }

    /// Une bille toutes les 0,16 s, au-dessus de son urne.
    fn drop_balls(
        mut commands: Commands,
        time: Res<Time>,
        mut drops: ResMut<Drops>,
        mut dice: ResMut<Dice>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        if drops.dropped >= drops.queue.len() || time.elapsed_secs() < drops.next {
            return;
        }
        let bin = drops.queue[drops.dropped];
        drops.dropped += 1;
        drops.next = time.elapsed_secs() + 0.16;
        let hue = IDEAS[IDEA_OF[bin]].hue;
        let jitter = dice.signed() * (WIDTH / 2.0 - BALL - 0.1);
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(BALL).mesh().uv(20, 10))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: idea_color(hue, 0.6),
                emissive: glow(hue, 0.2),
                perceptual_roughness: 0.25,
                metallic: 0.2,
                ..default()
            })),
            Transform::from_xyz(center(bin) + jitter, HEIGHT + 0.5, dice.signed() * 0.25),
            Restitution(0.12),
            Ball(bin),
        ));
    }

    /// Quand tout est tombé, la couronne apparaît et tourne.
    fn crown(
        time: Res<Time>,
        drops: Res<Drops>,
        mut crown: Query<(&mut Transform, &mut Visibility), With<Crown>>,
    ) {
        let done = drops.dropped >= drops.queue.len();
        for (mut transform, mut visibility) in &mut crown {
            if done && time.elapsed_secs() > drops.next + 0.8 {
                *visibility = Visibility::Visible;
                transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 1.6)
                    * Quat::from_rotation_x(FRAC_PI_2 * 0.9);
                transform.translation.y = HEIGHT + 0.9 + 0.12 * (time.elapsed_secs() * 2.0).sin();
            }
        }
    }

    fn count(drops: Res<Drops>, counter: Res<Counter>) {
        if let Some(state) = counter.0
            && state.is_alive()
            && state.get() != drops.dropped
        {
            state.set(drops.dropped);
        }
    }

    /// La scène, sans fenêtre.
    pub fn app(counter: Option<paquets::active::State<usize>>) -> App {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .insert_resource(Counter(counter))
            .add_systems(Startup, setup)
            .add_systems(Update, (drop_balls, crown, count));
        app
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn step(app: &mut App, frames: usize) {
        for _ in 0..frames {
            app.update();
        }
    }

    #[test]
    fn hero_tokens_fall_and_rest_on_the_tray() {
        let mut app = hero::app(false);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));
        step(&mut app, 60 * 14);
        let world = app.world_mut();
        let tokens: Vec<Vec3> = world
            .query_filtered::<&Transform, With<hero::Token>>()
            .iter(world)
            .map(|t| t.translation)
            .collect();
        assert_eq!(
            tokens.len(),
            hero::AUTO_TOKENS,
            "every token is still there"
        );
        for at in &tokens {
            assert!(
                at.y > -0.2 && at.y < 3.0,
                "a token rests on the tray: {at:?}"
            );
            assert!(Vec2::new(at.x, at.z).length() < 4.7, "on the tray: {at:?}");
        }
        let frame = RenderFrame::extract(app.world(), Vec2::new(800.0, 600.0)).unwrap();
        // Le plateau et son rebord, six cartes, le jeton, son anneau et les jetons tombés.
        assert_eq!(frame.mesh_count(), 2 + 6 + 2 + hero::AUTO_TOKENS);
    }

    #[test]
    fn every_ball_lands_in_its_bin_and_the_crown_appears() {
        let mut app = ballot::app(None);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));
        step(&mut app, 60 * 12);
        let world = app.world_mut();
        let mut per_bin = [0usize; 4];
        for (transform, ball) in world.query::<(&Transform, &ballot::Ball)>().iter(world) {
            let at = transform.translation;
            let left = ballot::center(ball.0) - ballot::WIDTH / 2.0;
            assert!(
                at.x > left && at.x < left + ballot::WIDTH,
                "ball of bin {} at {at:?}",
                ball.0
            );
            assert!(at.y < ballot::HEIGHT, "inside the bin: {at:?}");
            per_bin[ball.0] += 1;
        }
        assert_eq!(per_bin, ballot::VOTES);
        let visible = world
            .query_filtered::<&Visibility, With<ballot::Crown>>()
            .iter(world)
            .all(|v| *v == Visibility::Visible);
        assert!(visible, "the crown shows the winner");
        assert_eq!(ballot::winner(), 1);
    }
}
