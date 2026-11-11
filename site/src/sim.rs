//! Les deux animations du site, en 2D : un petit moteur physique (des disques, la
//! gravité, des parois, les chocs) et les deux scènes qui l'utilisent.
//!
//! - [`HeroScene`] : six cartes d'idées flottent au-dessus d'un plateau ; des jetons
//!   « oui » et « non » tombent, rebondissent et s'empilent ; un appui en lâche un de plus.
//! - [`BallotScene`] : le dépouillement : une bille par « oui » tombe dans l'urne de son
//!   idée, la plus remplie reçoit sa couronne.
//!
//! Tout est en Rust pur, sans navigateur : le serveur et les tests font avancer les
//! scènes pas à pas et vérifient où sont les objets ; le navigateur les dessine dans un
//! canvas 2D (`client::canvas`).

/// Un disque qui tombe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub r: f32,
    /// Sa couleur, ou son idée (selon la scène).
    pub kind: usize,
    /// Pour un jeton : oui ou non.
    pub yes: bool,
    /// Son angle, pour le dessiner en roulant.
    pub angle: f32,
}

/// Une paroi : un segment fixe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wall {
    pub ax: f32,
    pub ay: f32,
    pub bx: f32,
    pub by: f32,
}

/// Un monde 2D : l'axe y descend, l'unité est le pixel logique.
#[derive(Clone, Debug, Default)]
pub struct World {
    pub balls: Vec<Ball>,
    pub walls: Vec<Wall>,
    pub gravity: f32,
    /// Ce qui reste du temps à simuler (pas fixes).
    pending: f32,
}

/// Le pas de la simulation : 1/120 s, quel que soit le rythme des images.
pub const STEP: f32 = 1.0 / 120.0;
const RESTITUTION: f32 = 0.32;
const FRICTION: f32 = 0.985;
const PASSES: usize = 4;

impl World {
    pub fn new(gravity: f32) -> World {
        World {
            gravity,
            ..World::default()
        }
    }

    pub fn wall(&mut self, ax: f32, ay: f32, bx: f32, by: f32) {
        self.walls.push(Wall { ax, ay, bx, by });
    }

    /// Avance de `dt` secondes (au plus 1/20 s d'un coup, pour un onglet qui revient).
    pub fn advance(&mut self, dt: f32) {
        self.pending = (self.pending + dt).min(0.05);
        while self.pending >= STEP {
            self.pending -= STEP;
            self.step();
        }
    }

    fn step(&mut self) {
        for b in &mut self.balls {
            b.vy += self.gravity * STEP;
            b.vx *= 0.999;
            b.vy *= 0.999;
            b.x += b.vx * STEP;
            b.y += b.vy * STEP;
            b.angle += b.vx * STEP / b.r;
        }
        for _ in 0..PASSES {
            self.collide_walls();
            self.collide_balls();
        }
    }

    fn collide_walls(&mut self) {
        for b in &mut self.balls {
            for w in &self.walls {
                let (dx, dy) = (w.bx - w.ax, w.by - w.ay);
                let len2 = dx * dx + dy * dy;
                let t = (((b.x - w.ax) * dx + (b.y - w.ay) * dy) / len2).clamp(0.0, 1.0);
                let (cx, cy) = (w.ax + t * dx, w.ay + t * dy);
                let (nx, ny) = (b.x - cx, b.y - cy);
                let d = (nx * nx + ny * ny).sqrt();
                if d >= b.r || d == 0.0 {
                    continue;
                }
                let (nx, ny) = (nx / d, ny / d);
                b.x += nx * (b.r - d);
                b.y += ny * (b.r - d);
                let vn = b.vx * nx + b.vy * ny;
                if vn < 0.0 {
                    // Rebond le long de la normale, frottement le long de la paroi.
                    b.vx -= (1.0 + RESTITUTION) * vn * nx;
                    b.vy -= (1.0 + RESTITUTION) * vn * ny;
                    b.vx *= FRICTION;
                    b.vy *= FRICTION;
                }
            }
        }
    }

    fn collide_balls(&mut self) {
        let n = self.balls.len();
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = (self.balls[i], self.balls[j]);
                let (dx, dy) = (b.x - a.x, b.y - a.y);
                let min = a.r + b.r;
                let d2 = dx * dx + dy * dy;
                if d2 >= min * min || d2 == 0.0 {
                    continue;
                }
                let d = d2.sqrt();
                let (nx, ny) = (dx / d, dy / d);
                let push = (min - d) / 2.0;
                let vn = (b.vx - a.vx) * nx + (b.vy - a.vy) * ny;
                let j_ = if vn < 0.0 {
                    -(1.0 + RESTITUTION) * vn / 2.0
                } else {
                    0.0
                };
                let a = &mut self.balls[i];
                a.x -= nx * push;
                a.y -= ny * push;
                a.vx -= j_ * nx;
                a.vy -= j_ * ny;
                let b = &mut self.balls[j];
                b.x += nx * push;
                b.y += ny * push;
                b.vx += j_ * nx;
                b.vy += j_ * ny;
            }
        }
    }
}

/// Un générateur de nombres (xorshift64*), assez pour des places et des couleurs.
#[derive(Clone, Debug)]
pub struct Dice(pub u64);

impl Dice {
    pub fn roll(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Un nombre entre 0 et 1.
    pub fn unit(&mut self) -> f32 {
        (self.roll() % 10_001) as f32 / 10_000.0
    }
}

// ----- L'accueil ---------------------------------------------------------------------------

/// La scène d'accueil, sur 400 × 400 pixels logiques.
#[derive(Clone, Debug)]
pub struct HeroScene {
    pub world: World,
    /// Le temps écoulé (s).
    pub time: f32,
    auto: usize,
    next_drop: f32,
    dice: Dice,
}

impl HeroScene {
    pub const WIDTH: f32 = 400.0;
    pub const HEIGHT: f32 = 400.0;
    /// Le plateau : de `TRAY_LEFT` à `TRAY_RIGHT`, fond à `TRAY_FLOOR`, bords à `TRAY_TOP`.
    pub const TRAY_LEFT: f32 = 56.0;
    pub const TRAY_RIGHT: f32 = 344.0;
    pub const TRAY_FLOOR: f32 = 360.0;
    pub const TRAY_TOP: f32 = 262.0;
    pub const TOKEN: f32 = 15.0;
    pub const AUTO_TOKENS: usize = 14;
    pub const MAX_TOKENS: usize = 34;

    pub fn new() -> HeroScene {
        let mut world = World::new(1500.0);
        let (l, r, f, t) = (
            Self::TRAY_LEFT,
            Self::TRAY_RIGHT,
            Self::TRAY_FLOOR,
            Self::TRAY_TOP,
        );
        // Un plateau aux bords légèrement évasés.
        world.wall(l + 12.0, f, r - 12.0, f);
        world.wall(l, t, l + 12.0, f);
        world.wall(r, t, r - 12.0, f);
        HeroScene {
            world,
            time: 0.0,
            auto: Self::AUTO_TOKENS,
            next_drop: 0.5,
            dice: Dice(0x9E37_79B9_7F4A_7C15),
        }
    }

    /// Lâche un jeton au-dessus de `x` (un « non » sur trois).
    pub fn drop_at(&mut self, x: f32) {
        let x = x.clamp(Self::TRAY_LEFT + 30.0, Self::TRAY_RIGHT - 30.0);
        if self.world.balls.len() >= Self::MAX_TOKENS {
            self.world.balls.remove(0);
        }
        let yes = !self.dice.roll().is_multiple_of(3);
        let kind = (self.dice.roll() % 3) as usize;
        let vx = (self.dice.unit() - 0.5) * 120.0;
        self.world.balls.push(Ball {
            x,
            y: -Self::TOKEN,
            vx,
            vy: 0.0,
            r: Self::TOKEN,
            kind,
            yes,
            angle: 0.0,
        });
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        if self.auto > 0 && self.time >= self.next_drop {
            self.auto -= 1;
            self.next_drop = self.time + 0.42;
            let x = Self::TRAY_LEFT
                + 40.0
                + self.dice.unit() * (Self::TRAY_RIGHT - Self::TRAY_LEFT - 80.0);
            self.drop_at(x);
        }
        self.world.advance(dt);
        // Ce qui est tombé du plateau s'en va.
        self.world.balls.retain(|b| b.y < Self::HEIGHT + 60.0);
    }

    /// Une carte d'idée qui flotte : son centre et son angle à l'instant `time`.
    pub fn card(&self, i: usize) -> (f32, f32, f32) {
        let t = self.time;
        let phase = i as f32 * 1.1;
        let x = 52.0 + i as f32 * 59.0 + 5.0 * (t * 0.7 + phase).sin();
        let y =
            128.0 + 16.0 * (t * 1.3 + phase).sin() - if i.is_multiple_of(2) { 0.0 } else { 26.0 };
        let angle = 0.14 * (t * 0.9 + phase).sin() + if i.is_multiple_of(2) { -0.08 } else { 0.08 };
        (x, y, angle)
    }
}

impl Default for HeroScene {
    fn default() -> Self {
        Self::new()
    }
}

// ----- Le dépouillement --------------------------------------------------------------------

/// Les « oui » de chaque urne (les idées 1 à 4 de la démo).
pub const VOTES: [usize; 4] = [5, 9, 6, 3];

/// Les urnes, sur 400 × 260 pixels logiques.
#[derive(Clone, Debug)]
pub struct BallotScene {
    pub world: World,
    pub time: f32,
    queue: Vec<usize>,
    next: f32,
    pub dropped: usize,
    /// Quand la dernière bille est tombée.
    finished_at: Option<f32>,
    dice: Dice,
}

impl BallotScene {
    pub const WIDTH: f32 = 400.0;
    pub const HEIGHT: f32 = 260.0;
    pub const LEFT: f32 = 20.0;
    pub const BIN: f32 = 90.0;
    pub const TOP: f32 = 92.0;
    pub const FLOOR: f32 = 244.0;
    pub const BALL: f32 = 11.0;

    pub fn new() -> BallotScene {
        let mut world = World::new(1300.0);
        let right = Self::LEFT + Self::BIN * VOTES.len() as f32;
        world.wall(Self::LEFT, Self::FLOOR, right, Self::FLOOR);
        for i in 0..=VOTES.len() {
            let x = Self::LEFT + i as f32 * Self::BIN;
            world.wall(x, Self::TOP, x, Self::FLOOR);
        }
        // Les billes, une urne après l'autre, comme des votes qui arrivent.
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
        BallotScene {
            world,
            time: 0.0,
            queue,
            next: 0.35,
            dropped: 0,
            finished_at: None,
            dice: Dice(0xD1B5_4A32_D192_ED03),
        }
    }

    pub fn total(&self) -> usize {
        self.queue.len()
    }

    /// Le centre de l'urne `bin`.
    pub fn center(bin: usize) -> f32 {
        Self::LEFT + (bin as f32 + 0.5) * Self::BIN
    }

    /// L'urne gagnante.
    pub fn winner() -> usize {
        (0..VOTES.len()).max_by_key(|&i| VOTES[i]).unwrap_or(0)
    }

    /// L'urne où se trouve `x`.
    pub fn bin_at(x: f32) -> Option<usize> {
        let i = ((x - Self::LEFT) / Self::BIN).floor();
        (i >= 0.0 && (i as usize) < VOTES.len()).then_some(i as usize)
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        if self.dropped < self.queue.len() && self.time >= self.next {
            let bin = self.queue[self.dropped];
            self.dropped += 1;
            self.next = self.time + 0.15;
            let jitter = (self.dice.unit() - 0.5) * (Self::BIN - 2.0 * Self::BALL - 16.0);
            self.world.balls.push(Ball {
                x: Self::center(bin) + jitter,
                y: Self::TOP - 40.0,
                vx: 0.0,
                vy: 0.0,
                r: Self::BALL,
                kind: bin,
                yes: true,
                angle: 0.0,
            });
            if self.dropped == self.queue.len() {
                self.finished_at = Some(self.time);
            }
        }
        self.world.advance(dt);
    }

    /// La couronne : visible 0,9 s après la dernière bille, avec sa hauteur.
    pub fn crown(&self) -> Option<f32> {
        let since = self.time - self.finished_at? - 0.9;
        (since >= 0.0).then(|| Self::TOP - 46.0 + 5.0 * (self.time * 3.0).sin())
    }
}

impl Default for BallotScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(f: &mut impl FnMut(f32), seconds: f32) {
        for _ in 0..(seconds * 60.0) as usize {
            f(1.0 / 60.0);
        }
    }

    fn no_deep_overlap(balls: &[Ball]) {
        for (i, a) in balls.iter().enumerate() {
            for b in &balls[i + 1..] {
                let d = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
                assert!(d > (a.r + b.r) * 0.85, "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn hero_tokens_fall_and_rest_in_the_tray() {
        let mut scene = HeroScene::new();
        run(&mut |dt| scene.update(dt), 12.0);
        let balls = &scene.world.balls;
        assert_eq!(
            balls.len(),
            HeroScene::AUTO_TOKENS,
            "every token is still there"
        );
        for b in balls {
            assert!(
                b.x > HeroScene::TRAY_LEFT && b.x < HeroScene::TRAY_RIGHT,
                "{b:?}"
            );
            assert!(
                b.y <= HeroScene::TRAY_FLOOR - b.r + 1.0,
                "above the floor: {b:?}"
            );
            assert!(b.vx.abs() < 30.0 && b.vy.abs() < 30.0, "at rest: {b:?}");
        }
        no_deep_overlap(balls);
        // Les « oui » et les « non » sont mêlés.
        assert!(balls.iter().any(|b| b.yes) && balls.iter().any(|b| !b.yes));
    }

    #[test]
    fn a_tap_drops_one_more_and_the_oldest_goes_when_full() {
        let mut scene = HeroScene::new();
        for i in 0..HeroScene::MAX_TOKENS + 5 {
            scene.drop_at(100.0 + i as f32);
        }
        assert_eq!(scene.world.balls.len(), HeroScene::MAX_TOKENS);
    }

    #[test]
    fn every_ball_lands_in_its_bin_and_the_crown_appears() {
        let mut scene = BallotScene::new();
        assert!(scene.crown().is_none());
        run(&mut |dt| scene.update(dt), 9.0);
        assert_eq!(scene.dropped, scene.total());
        let mut per_bin = [0usize; 4];
        for b in &scene.world.balls {
            assert_eq!(BallotScene::bin_at(b.x), Some(b.kind), "{b:?}");
            assert!(
                b.y < BallotScene::FLOOR && b.y > BallotScene::TOP - b.r,
                "{b:?}"
            );
            per_bin[b.kind] += 1;
        }
        assert_eq!(per_bin, VOTES);
        no_deep_overlap(&scene.world.balls);
        assert!(scene.crown().is_some());
        assert_eq!(BallotScene::winner(), 1);
    }

    #[test]
    fn the_cards_float_without_leaving_the_scene() {
        let mut scene = HeroScene::new();
        for _ in 0..600 {
            scene.update(1.0 / 60.0);
            for i in 0..6 {
                let (x, y, a) = scene.card(i);
                assert!(x > 20.0 && x < HeroScene::WIDTH - 20.0 && y > 60.0 && y < 200.0);
                assert!(a.abs() < 0.3);
            }
        }
    }
}
