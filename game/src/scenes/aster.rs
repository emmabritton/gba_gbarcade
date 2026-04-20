use crate::gfx::background;
use crate::printer::WhiteVariWidthText;
use crate::rng::{next_i32, next_u16_in};
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::object::{AffineMatrixObject, AffineMode, Object, ObjectAffine, Sprite, Tag};
use agb::display::tiled::RegularBackground;
use agb::display::{AffineMatrix, GraphicsFrame, Priority};
use agb::fixnum::{Num, Vector2D, num, vec2};
use agb::input::{Button, ButtonController};
use agb::rng::RandomNumberGenerator;
use alloc::format;
use resources::{bg, sprites};

const PLAY_LEFT: i32 = 2;
const PLAY_RIGHT: i32 = 237;
const PLAY_TOP: i32 = 11;
const PLAY_BOTTOM: i32 = 157;
const CENTER_X: i32 = (PLAY_LEFT + PLAY_RIGHT) / 2;
const CENTER_Y: i32 = (PLAY_TOP + PLAY_BOTTOM) / 2;

type FP = Num<i32, 8>;
type Angle = Num<i32, 8>;

const FP_LEFT: FP = Num::from_raw(PLAY_LEFT << 8);
const FP_RIGHT: FP = Num::from_raw(PLAY_RIGHT << 8);
const FP_TOP: FP = Num::from_raw(PLAY_TOP << 8);
const FP_BOTTOM: FP = Num::from_raw(PLAY_BOTTOM << 8);
const FP_W: FP = Num::from_raw((PLAY_RIGHT - PLAY_LEFT) << 8);
const FP_H: FP = Num::from_raw((PLAY_BOTTOM - PLAY_TOP) << 8);

const ROTATE_SPEED: Angle = num!(0.006);
const THRUST: FP = num!(0.06);
const FRICTION: FP = num!(0.98);
const MAX_SPEED_SQ: FP = num!(9.0);

const BULLET_SPEED: FP = num!(3.5);
const BULLET_LIFE: u16 = 80;
const FIRE_COOLDOWN: u16 = 22;
const MAX_BULLETS: usize = 4;
const MAX_ASTEROIDS: usize = 32;
const MAX_POPUPS: usize = 8;

const INIT_LARGE: usize = 4;
const INIT_MEDIUM: usize = 2;

const UFO_SPEED: FP = num!(0.7);
const UFO_INTERVAL: u16 = 500;
const UFO_W: i32 = 32;
const UFO_H: i32 = 32;

const DEATH_ANIM_FRAMES: u16 = 20;
const DEATH_PAUSE: u16 = 50;
const INVINCIBLE_FRAMES: u16 = 120;
const ENGINE_PERIOD: u8 = 5;

const SAFE_SPAWN_DIST_SQ: i32 = 55 * 55;

const LIFE_X0: i32 = 120;
const LIFE_Y: i32 = 2;
const LIFE_STRIDE: i32 = 9;

const SCORE_BREAK: i32 = 100;
const SCORE_SMALL: i32 = 200;
const SCORE_UFO: i32 = 1000;
const SCORE_BULLET: i32 = -10;
const SCORE_DEATH: i32 = -1000;
const SPRITE_SCORE_BREAK: &Sprite = sprites::ASTER_SCORE_BREAK.sprite(0);
const SPRITE_SCORE_SMALL: &Sprite = sprites::ASTER_SCORE_SMALL.sprite(0);
const SPRITE_SCORE_UFO: &Sprite = sprites::ASTER_SCORE_UFO.sprite(0);
const SPRITE_SCORE_BULLET: &Sprite = sprites::ASTER_SCORE_FIRE.sprite(0);
const SPRITE_SCORE_DEATH: &Sprite = sprites::ASTER_SCORE_DEATH.sprite(0);
const TAG_DEATH: &Tag = &sprites::ASTER_DEATH;
const SPRITE_BULLET: &Sprite = sprites::ASTER_BULLET.sprite(0);
const SPRITE_LIFE: &Sprite = sprites::ASTER_LIFE.sprite(0);
const SPRITE_UFO: &Sprite = sprites::ASTER_UFO.sprite(0);
const TAG_ASTEROIDS_SMALL: &Tag = &sprites::ASTER_ROID_SMALL;
const TAG_ASTEROIDS_MEDIUM: &Tag = &sprites::ASTER_ROID_MEDIUM;
const TAG_ASTEROIDS_LARGE: &Tag = &sprites::ASTER_ROID_LARGE;

#[derive(Copy, Clone, Eq, PartialEq)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(self) -> i32 {
        match self {
            AsteroidSize::Large => 13,
            AsteroidSize::Medium => 6,
            AsteroidSize::Small => 3,
        }
    }

    fn split(self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        }
    }

    fn sprite_half(self) -> i32 {
        match self {
            AsteroidSize::Large => 16,
            AsteroidSize::Medium => 8,
            AsteroidSize::Small => 4,
        }
    }

    fn tag(self) -> &'static Tag {
        match self {
            AsteroidSize::Large => TAG_ASTEROIDS_LARGE,
            AsteroidSize::Medium => TAG_ASTEROIDS_MEDIUM,
            AsteroidSize::Small => TAG_ASTEROIDS_SMALL,
        }
    }

    fn speed_range(self) -> (u16, u16) {
        match self {
            AsteroidSize::Large => (18, 55),
            AsteroidSize::Medium => (45, 110),
            AsteroidSize::Small => (80, 180),
        }
    }
}

#[derive(Copy, Clone)]
struct Asteroid {
    pos: Vector2D<FP>,
    vel: Vector2D<FP>,
    size: AsteroidSize,
    variant: usize,
    active: bool,
}

impl Asteroid {
    const NONE: Self = Self {
        pos: Vector2D {
            x: Num::from_raw(0),
            y: Num::from_raw(0),
        },
        vel: Vector2D {
            x: Num::from_raw(0),
            y: Num::from_raw(0),
        },
        size: AsteroidSize::Large,
        variant: 0,
        active: false,
    };
}

#[derive(Copy, Clone)]
struct Bullet {
    pos: Vector2D<FP>,
    vel: Vector2D<FP>,
    life: u16,
    active: bool,
}

impl Bullet {
    const NONE: Self = Self {
        pos: Vector2D {
            x: Num::from_raw(0),
            y: Num::from_raw(0),
        },
        vel: Vector2D {
            x: Num::from_raw(0),
            y: Num::from_raw(0),
        },
        life: 0,
        active: false,
    };
}

#[derive(Copy, Clone)]
struct ScorePopup {
    pos: Vector2D<FP>,
    sprite: &'static Sprite,
    timer: u16,
    active: bool,
}

impl ScorePopup {
    const NONE: Self = Self {
        pos: Vector2D {
            x: Num::from_raw(0),
            y: Num::from_raw(0),
        },
        sprite: SPRITE_SCORE_BREAK,
        timer: 0,
        active: false,
    };
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum GameState {
    Playing,
    PlayerDead,
}

pub struct AsterState {
    bg_back: RegularBackground,
    bg_fore: RegularBackground,
    rng: RandomNumberGenerator,

    player_pos: Vector2D<FP>,
    player_vel: Vector2D<FP>,
    player_angle: Angle,
    thrusting: bool,
    engine_frame: usize,
    engine_timer: u8,
    invincible: u16,

    bullets: [Bullet; MAX_BULLETS],
    fire_cooldown: u16,

    asteroids: [Asteroid; MAX_ASTEROIDS],

    ufo_active: bool,
    ufo_pos: Vector2D<FP>,
    ufo_dir: i32,
    ufo_timer: u16,

    lives: u8,

    popups: [ScorePopup; MAX_POPUPS],

    state: GameState,
    state_timer: u16,
    death_pos: Vector2D<i32>,
    score: i32,
}

impl AsterState {
    pub fn new(seed: [u32; 4]) -> Self {
        let rng = RandomNumberGenerator::new_with_seed(seed);
        let bg_back = background(&bg::bg_aster, Priority::P3);
        let bg_fore = background(&bg::bg_aster_fore, Priority::P0);

        let player_pos = Vector2D::new(Num::from_raw(CENTER_X << 8), Num::from_raw(CENTER_Y << 8));

        let mut state = Self {
            bg_back,
            bg_fore,
            rng,
            player_pos,
            player_vel: Vector2D::new(num!(0), num!(0)),
            player_angle: num!(0),
            thrusting: false,
            engine_frame: 0,
            engine_timer: 0,
            invincible: 0,
            bullets: [Bullet::NONE; MAX_BULLETS],
            fire_cooldown: 0,
            asteroids: [Asteroid::NONE; MAX_ASTEROIDS],
            ufo_active: false,
            ufo_pos: Vector2D::new(num!(0), num!(0)),
            ufo_dir: 1,
            ufo_timer: UFO_INTERVAL,
            lives: 3,
            popups: [ScorePopup::NONE; MAX_POPUPS],
            state: GameState::Playing,
            state_timer: 0,
            death_pos: vec2(CENTER_X, CENTER_Y),
            score: 0,
        };

        state.spawn_initial_asteroids();
        state
    }

    fn spawn_initial_asteroids(&mut self) {
        for _ in 0..INIT_LARGE {
            self.spawn_asteroid(AsteroidSize::Large, None);
        }
        for _ in 0..INIT_MEDIUM {
            self.spawn_asteroid(AsteroidSize::Medium, None);
        }
    }

    fn spawn_asteroid(&mut self, size: AsteroidSize, pos: Option<Vector2D<FP>>) {
        let Some(slot) = self.asteroids.iter().position(|a| !a.active) else {
            return;
        };

        let spawn_pos = pos.unwrap_or_else(|| {
            for _ in 0..20 {
                let x = next_u16_in(&mut self.rng, PLAY_LEFT as u16 + 10, PLAY_RIGHT as u16 - 10)
                    as i32;
                let y = next_u16_in(&mut self.rng, PLAY_TOP as u16 + 10, PLAY_BOTTOM as u16 - 10)
                    as i32;
                let dx = x - CENTER_X;
                let dy = y - CENTER_Y;
                if dx * dx + dy * dy > SAFE_SPAWN_DIST_SQ {
                    return Vector2D::new(Num::from_raw(x << 8), Num::from_raw(y << 8));
                }
            }
            Vector2D::new(Num::from_raw(PLAY_LEFT << 8), Num::from_raw(PLAY_TOP << 8))
        });

        let (speed_min, speed_max) = size.speed_range();
        let angle = Angle::from_raw(next_i32(&mut self.rng) & 0xFF);
        let speed = FP::from_raw(next_u16_in(&mut self.rng, speed_min, speed_max) as i32);
        let vel = Vector2D::new(angle.sin() * speed, angle.cos() * speed);
        let variant = next_u16_in(&mut self.rng, 0, 2) as usize;

        self.asteroids[slot] = Asteroid {
            pos: spawn_pos,
            vel,
            size,
            variant,
            active: true,
        };
    }

    fn spawn_children(&mut self, parent_pos: Vector2D<FP>, child_size: AsteroidSize) {
        let (speed_min, speed_max) = child_size.speed_range();
        for _ in 0..2 {
            let Some(slot) = self.asteroids.iter().position(|a| !a.active) else {
                break;
            };
            let angle = Angle::from_raw(next_i32(&mut self.rng) & 0xFF);
            let speed = FP::from_raw(next_u16_in(&mut self.rng, speed_min, speed_max) as i32);
            let vel = Vector2D::new(angle.sin() * speed, angle.cos() * speed);
            let variant = next_u16_in(&mut self.rng, 0, 2) as usize;
            self.asteroids[slot] = Asteroid {
                pos: parent_pos,
                vel,
                size: child_size,
                variant,
                active: true,
            };
        }
    }

    fn add_popup(&mut self, pos: Vector2D<FP>, sprite: &'static Sprite, score: i32) {
        self.score += score;
        if let Some(slot) = self.popups.iter().position(|p| !p.active) {
            self.popups[slot] = ScorePopup {
                pos,
                sprite,
                timer: 30,
                active: true,
            };
        }
    }

    fn respawn_player(&mut self) {
        self.player_pos = Vector2D::new(Num::from_raw(CENTER_X << 8), Num::from_raw(CENTER_Y << 8));
        self.player_vel = Vector2D::new(num!(0), num!(0));
        self.player_angle = num!(0);
        self.bullets = [Bullet::NONE; MAX_BULLETS];
        self.fire_cooldown = 0;
        self.invincible = INVINCIBLE_FRAMES;
    }
}

impl AsterState {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        if self.state == GameState::PlayerDead {
            if self.state_timer > 0 {
                self.state_timer -= 1;
            } else if self.lives == 0 {
                return Some(SceneAction::Lose);
            } else {
                self.lives -= 1;
                self.respawn_player();
                self.state = GameState::Playing;
            }
            return None;
        }

        if button_controller.is_pressed(Button::Left) {
            self.player_angle += ROTATE_SPEED;
        }
        if button_controller.is_pressed(Button::Right) {
            self.player_angle -= ROTATE_SPEED;
        }

        self.thrusting = button_controller.is_pressed(Button::Up);
        if self.thrusting {
            let dir = Vector2D::new(-self.player_angle.sin(), -self.player_angle.cos());
            self.player_vel += dir * THRUST;
            // Clamp speed
            if self.player_vel.magnitude_squared() > MAX_SPEED_SQ {
                self.player_vel = self.player_vel.normalise() * num!(3.0);
            }
        }

        self.player_vel *= FRICTION;

        self.player_pos = wrap_pos(self.player_pos + self.player_vel);

        // Fire
        self.fire_cooldown = self.fire_cooldown.saturating_sub(1);
        if button_controller.is_just_pressed(Button::A)
            && self.fire_cooldown == 0
            && let Some(slot) = self.bullets.iter().position(|b| !b.active)
        {
            let dir = Vector2D::new(-self.player_angle.sin(), -self.player_angle.cos());
            self.bullets[slot] = Bullet {
                pos: self.player_pos,
                vel: dir * BULLET_SPEED + self.player_vel,
                life: BULLET_LIFE,
                active: true,
            };
            self.fire_cooldown = FIRE_COOLDOWN;
            sound_controller.play_sfx(SoundEffect::Place);
            self.add_popup(self.player_pos, SPRITE_SCORE_BULLET, SCORE_BULLET);
        }

        for bullet in &mut self.bullets {
            if bullet.active {
                bullet.pos = wrap_pos(bullet.pos + bullet.vel);
                bullet.life = bullet.life.saturating_sub(1);
                if bullet.life == 0 {
                    bullet.active = false;
                }
            }
        }

        // Move asteroids
        for i in 0..MAX_ASTEROIDS {
            if self.asteroids[i].active {
                self.asteroids[i].pos = wrap_pos(self.asteroids[i].pos + self.asteroids[i].vel);
            }
        }

        // Bullet vs asteroid collision
        let mut hits: [(usize, usize); MAX_ASTEROIDS] = [(usize::MAX, usize::MAX); MAX_ASTEROIDS];
        let mut hit_count = 0;

        'outer: for bi in 0..MAX_BULLETS {
            if !self.bullets[bi].active {
                continue;
            }
            for ai in 0..MAX_ASTEROIDS {
                if !self.asteroids[ai].active {
                    continue;
                }
                let threshold = self.asteroids[ai].size.radius() + 2;
                if dist_sq(self.bullets[bi].pos, self.asteroids[ai].pos) < threshold * threshold {
                    hits[hit_count] = (bi, ai);
                    hit_count += 1;
                    self.bullets[bi].active = false;
                    continue 'outer;
                }
            }
        }

        for (_, ai) in hits.iter().take(hit_count) {
            let aster = self.asteroids[*ai];
            if !aster.active {
                continue;
            }
            self.asteroids[*ai].active = false;

            let sfx = match aster.size {
                AsteroidSize::Large => SoundEffect::BrickBreak,
                AsteroidSize::Medium => SoundEffect::InvaderPlayerDeath,
                AsteroidSize::Small => SoundEffect::InvaderCrumble,
            };
            sound_controller.play_sfx(sfx);
            self.add_popup(aster.pos, SPRITE_SCORE_BREAK, SCORE_BREAK);

            if let Some(child_size) = aster.size.split() {
                if child_size == AsteroidSize::Small {
                    self.add_popup(aster.pos, SPRITE_SCORE_SMALL, SCORE_SMALL);
                }
                self.spawn_children(aster.pos, child_size);
            }
        }

        // ufo logic
        if self.ufo_active {
            let move_delta = Vector2D::new(UFO_SPEED * Num::from_raw(self.ufo_dir << 8), num!(0));
            self.ufo_pos += move_delta;
            let ufo_x = self.ufo_pos.x.floor();
            if !(PLAY_LEFT - UFO_W..=PLAY_RIGHT + UFO_W).contains(&ufo_x) {
                self.ufo_active = false;
                self.ufo_timer = UFO_INTERVAL;
            }
        } else {
            self.ufo_timer = self.ufo_timer.saturating_sub(1);
            if self.ufo_timer == 0 {
                self.ufo_active = true;
                self.ufo_dir = if (next_i32(&mut self.rng) & 1) == 0 {
                    1
                } else {
                    -1
                };
                let start_x = if self.ufo_dir > 0 {
                    PLAY_LEFT - UFO_W
                } else {
                    PLAY_RIGHT + UFO_W
                };
                let start_y = next_u16_in(
                    &mut self.rng,
                    (PLAY_TOP + 20) as u16,
                    (PLAY_BOTTOM - 20) as u16,
                ) as i32;
                self.ufo_pos =
                    Vector2D::new(Num::from_raw(start_x << 8), Num::from_raw(start_y << 8));
            }
        }

        // Bullet vs UFO
        if self.ufo_active {
            let ufo_pos_i = vec2(self.ufo_pos.x.floor(), self.ufo_pos.y.floor());
            for bullet in &mut self.bullets {
                if !bullet.active {
                    continue;
                }
                let bx = bullet.pos.x.floor();
                let by = bullet.pos.y.floor();
                if bx >= ufo_pos_i.x - UFO_W / 2
                    && bx <= ufo_pos_i.x + UFO_W / 2
                    && by >= ufo_pos_i.y - UFO_H / 2
                    && by <= ufo_pos_i.y + UFO_H / 2
                {
                    bullet.active = false;
                    self.ufo_active = false;
                    self.ufo_timer = UFO_INTERVAL;
                    sound_controller.play_sfx(SoundEffect::InvaderDeath);
                    self.add_popup(self.ufo_pos, SPRITE_SCORE_UFO, SCORE_UFO);
                    break;
                }
            }
        }

        // player collision
        if self.invincible == 0 {
            let px = self.player_pos.x.floor();
            let py = self.player_pos.y.floor();

            // vs asteroids
            let mut player_hit = false;
            for aster in &self.asteroids {
                if !aster.active {
                    continue;
                }
                let threshold = aster.size.radius() + 5;
                if dist_sq(self.player_pos, aster.pos) < threshold * threshold {
                    player_hit = true;
                    break;
                }
            }

            // vs ufo
            if !player_hit && self.ufo_active {
                let ufo_x = self.ufo_pos.x.floor();
                let ufo_y = self.ufo_pos.y.floor();
                if (px - ufo_x).abs() < UFO_W / 2 + 5 && (py - ufo_y).abs() < UFO_H / 2 + 5 {
                    player_hit = true;
                }
            }

            if player_hit {
                self.death_pos = vec2(px, py);
                self.state = GameState::PlayerDead;
                self.state_timer = DEATH_ANIM_FRAMES + DEATH_PAUSE;
                sound_controller.play_sfx(SoundEffect::InvaderPlayerDeath);
                self.add_popup(self.player_pos, SPRITE_SCORE_DEATH, SCORE_DEATH);
            }
        } else {
            self.invincible -= 1;
        }

        for popup in &mut self.popups {
            if popup.active {
                popup.timer = popup.timer.saturating_sub(1);
                if popup.timer == 0 {
                    popup.active = false;
                }
            }
        }

        if self.thrusting {
            self.engine_timer = self.engine_timer.saturating_sub(1);
            if self.engine_timer == 0 {
                self.engine_timer = ENGINE_PERIOD;
                self.engine_frame = (self.engine_frame + 1) % 3;
            }
        }

        if self.asteroids.iter().all(|a| !a.active) {
            return Some(SceneAction::Win);
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame, is_running: bool) {
        self.bg_back.show(frame);
        self.bg_fore.show(frame);

        WhiteVariWidthText::new(&format!("Score: {: >5}", self.score), 0).show(vec2(180, 1), frame);

        for i in 0..self.lives as i32 {
            Object::new(SPRITE_LIFE)
                .set_pos(vec2(LIFE_X0 + i * LIFE_STRIDE, LIFE_Y))
                .show(frame);
        }

        for aster in &self.asteroids {
            if !aster.active {
                continue;
            }
            let ax = aster.pos.x.floor();
            let ay = aster.pos.y.floor();
            let half = aster.size.sprite_half();
            Object::new(aster.size.tag().sprite(aster.variant))
                .set_pos(vec2(ax - half, ay - half))
                .set_priority(Priority::P2)
                .show(frame);
        }

        if self.ufo_active {
            let ux = self.ufo_pos.x.floor();
            let uy = self.ufo_pos.y.floor();
            Object::new(SPRITE_UFO)
                .set_pos(vec2(ux - UFO_W / 2, uy - UFO_H / 2))
                .set_priority(Priority::P2)
                .show(frame);
        }

        for bullet in &self.bullets {
            if bullet.active {
                Object::new(SPRITE_BULLET)
                    .set_pos(vec2(bullet.pos.x.floor(), bullet.pos.y.floor()))
                    .set_priority(Priority::P2)
                    .show(frame);
            }
        }

        // draw player
        match self.state {
            GameState::Playing => {
                let visible = self.invincible == 0 || (self.invincible / 4).is_multiple_of(2);
                if visible {
                    let engine_power = if self.thrusting {
                        1 + self.engine_frame
                    } else {
                        0
                    };
                    let px = self.player_pos.x.floor();
                    let py = self.player_pos.y.floor();
                    draw_player(vec2(px, py), self.player_angle, engine_power, frame);
                }
            }
            GameState::PlayerDead => {
                let elapsed = (DEATH_ANIM_FRAMES + DEATH_PAUSE).saturating_sub(self.state_timer);
                if elapsed < DEATH_ANIM_FRAMES {
                    let anim_frame = (elapsed / 4) as usize % 5;
                    Object::new(TAG_DEATH.sprite(anim_frame))
                        .set_pos(self.death_pos - vec2(16, 16))
                        .set_priority(Priority::P2)
                        .show(frame);
                }
            }
        }

        if is_running {
            for popup in &self.popups {
                if popup.active {
                    let px = popup.pos.x.floor();
                    let py = popup.pos.y.floor() - 12;
                    Object::new(popup.sprite)
                        .set_pos(vec2(px - 4, py))
                        .show(frame);
                }
            }
        }
    }
}

fn draw_player(pos: Vector2D<i32>, angle: Angle, engine_power: usize, frame: &mut GraphicsFrame) {
    let scale: Vector2D<Num<i32, 8>> = (4, 4).into();
    let matrix = AffineMatrix::from_rotation(angle) * AffineMatrix::from_scale(scale);
    let matrix_object = AffineMatrixObject::new(matrix);
    ObjectAffine::new(
        sprites::ASTER_SHIP.sprite(engine_power),
        matrix_object,
        AffineMode::Affine,
    )
    .set_pos(pos - vec2(32, 32))
    .set_priority(Priority::P2)
    .show(frame)
}

fn wrap_pos(pos: Vector2D<FP>) -> Vector2D<FP> {
    let mut x = pos.x;
    let mut y = pos.y;
    if x < FP_LEFT {
        x += FP_W;
    } else if x >= FP_RIGHT {
        x -= FP_W;
    }
    if y < FP_TOP {
        y += FP_H;
    } else if y >= FP_BOTTOM {
        y -= FP_H;
    }
    Vector2D::new(x, y)
}

fn dist_sq(a: Vector2D<FP>, b: Vector2D<FP>) -> i32 {
    let dx = (a.x - b.x).floor();
    let dy = (a.y - b.y).floor();
    dx * dx + dy * dy
}
