use crate::TILE_SIZE;
use crate::gfx::{ShowSprite, ShowTag, background_stack};
use crate::rng::next_u16_in;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::object::{Object, Sprite, Tag};
use agb::display::tiled::RegularBackground;
use agb::display::{GraphicsFrame, WIDTH};
use agb::fixnum::{Num, Rect, Vector2D, vec2};
use agb::input::{Button, ButtonController};
use agb::rng::RandomNumberGenerator;
use alloc::vec;
use alloc::vec::Vec;
use resources::sprites::{
    BRICK_BALL, BRICK_BLUE, BRICK_GREEN, BRICK_ORANGE, BRICK_PADDLE_L, BRICK_PADDLE_M,
    BRICK_PADDLE_R, BRICK_RED, BRICK_YELLOW,
};
use resources::{bg, sprites};

type Fp = Num<i32, 8>;

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
enum Powerup {
    Life,
    Paddle,
    Ball,
}

impl Powerup {
    pub fn sprite(&self) -> &'static Sprite {
        match self {
            Powerup::Life => sprites::BRICKS_EXTRA_LIFE.sprite(0),
            Powerup::Paddle => sprites::BRICKS_EXTRA_LEN.sprite(0),
            Powerup::Ball => sprites::BRICKS_EXTRA_BALL.sprite(0),
        }
    }
}

const FRAMES_PER_ACCEL: u8 = 3;
const PADDLE_MAX_SPEED: i32 = 6;
const PADDLE_Y: i32 = 149;
const MAX_LEN: u8 = 6;
const BOUNDS: Rect<i32> = Rect {
    position: vec2(2, 11),
    size: vec2(236, 147),
};
const FLIP_OFFSET: Vector2D<i32> = vec2(16, 0);
const BALL_SIZE: Vector2D<i32> = vec2(4, 4);
const BRICK_OFFSET: Vector2D<i32> = vec2(2, 3);

const LIFE_COUNT: i32 = 8;
const LIFE_Y: i32 = 4;
const LIFE_STRIDE: i32 = 8;

const SERVE_OFFSET: Vector2D<i32> = vec2(80, 120);

const LEVEL_BADGE: &Tag = &sprites::BRICKS_LEVEL;
const LEVEL_NUMS: &Tag = &sprites::BRICKS_LVL_NUM;

const LVL_BADGE_OFFSET: Vector2D<i32> = vec2(70, 90);
const LVL_NUM_OFFSET: Vector2D<i32> = vec2(152, 90);

const POWERUP_FRAMES: u16 = 1800; //30s
const POWERUP_BLINK_SLOW: u16 = 1200; // 20s remaining
const POWERUP_BLINK_FAST: u16 = 600; // 10s remaining
const POWERUP_SIZE: Vector2D<i32> = vec2(32, 16);
const MAX_LIVES: u8 = 16;

const POST_LAUNCH_DELAY_PER_LEVEL: u8 = 16;

// Ball speed per level
const BALL_SPEEDS: [Fp; 12] = [
    Num::from_raw(256),
    Num::from_raw(287),
    Num::from_raw(328),
    Num::from_raw(359),
    Num::from_raw(390),
    Num::from_raw(452),
    Num::from_raw(493),
    Num::from_raw(554),
    Num::from_raw(605),
    Num::from_raw(635),
    Num::from_raw(665),
    Num::from_raw(705),
];

// Spawn probabilities for extra life brick (higher chance when lower on life)
const EXTRA_LIFE_DENOMS: [u16; 15] = [
    1000, 2357, 3714, 5071, 6428, 7785, 9142, 10500, 11857, 13214, 14571, 15928, 17285, 18642,
    20000,
];

#[derive(Debug, Clone, Copy)]
enum BrickKind {
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
}

impl BrickKind {
    const fn idx(&self) -> usize {
        match self {
            BrickKind::Blue => 0,
            BrickKind::Green => 1,
            BrickKind::Yellow => 2,
            BrickKind::Orange => 3,
            BrickKind::Red => 4,
        }
    }

    fn next(&self) -> Option<Self> {
        match self {
            BrickKind::Blue => None,
            BrickKind::Green => Some(BrickKind::Blue),
            BrickKind::Yellow => Some(BrickKind::Green),
            BrickKind::Orange => Some(BrickKind::Yellow),
            BrickKind::Red => Some(BrickKind::Orange),
        }
    }
}

#[derive(Debug)]
struct Brick {
    kind: BrickKind,
    rect: Rect<i32>,
}

pub struct BricksState {
    paddle_len: u8,
    paddle_x: i32,
    paddle_speed: i32,
    paddle_dir: Option<bool>,
    paddle_accel_timer: u8,
    launched: bool,
    bricks: Vec<Brick>,
    empty_slots: Vec<Vector2D<i32>>,
    backgrounds: [RegularBackground; 1],
    brick_objs: [Object; 5],
    flipped_brick_objs: [Object; 5],
    paddle_cap_objs: [Object; 2],
    lives: u8,
    level: u8,
    show_level_badge: bool,
    powerup: Option<(Powerup, Rect<i32>, u16)>,
    extra_balls: Vec<(Vector2D<Fp>, Vector2D<i32>)>,
    rng: RandomNumberGenerator,
    trap_active: bool,
    //must be 0 before trap can be activated
    launch_timer: u8,
}

// Row order: index 0 = top row (smallest y on screen), index 4 = bottom row.
// Level spec is bottom-to-top, so rows[y] = spec[4 - y].
fn make_bricks(level: u8) -> Vec<Brick> {
    let rows: [BrickKind; 5] = match level {
        1 => [
            BrickKind::Blue,
            BrickKind::Blue,
            BrickKind::Blue,
            BrickKind::Blue,
            BrickKind::Blue,
        ],
        2 => [
            BrickKind::Green,
            BrickKind::Blue,
            BrickKind::Blue,
            BrickKind::Blue,
            BrickKind::Blue,
        ],
        3 => [
            BrickKind::Yellow,
            BrickKind::Green,
            BrickKind::Blue,
            BrickKind::Blue,
            BrickKind::Blue,
        ],
        4 => [
            BrickKind::Orange,
            BrickKind::Yellow,
            BrickKind::Green,
            BrickKind::Blue,
            BrickKind::Blue,
        ],
        5 => [
            BrickKind::Red,
            BrickKind::Orange,
            BrickKind::Yellow,
            BrickKind::Green,
            BrickKind::Blue,
        ],
        6 => [
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Orange,
            BrickKind::Yellow,
            BrickKind::Green,
        ],
        7 => [
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Orange,
            BrickKind::Yellow,
        ],
        8 => [
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Orange,
        ],
        9..=12 => [
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Red,
            BrickKind::Red,
        ],
        _ => unreachable!(),
    };

    let mut bricks = vec![];
    for (y, row) in rows.iter().enumerate() {
        for x in 0..6 {
            let rect = Rect::new(
                vec2(
                    BOUNDS.position.x + x * 40 + BRICK_OFFSET.x,
                    BOUNDS.position.y + y as i32 * 13 + BRICK_OFFSET.y,
                ),
                vec2(32, 10),
            );
            bricks.push(Brick { kind: *row, rect });
        }
    }
    bricks
}

impl BricksState {
    pub fn new(seed: [u32; 4]) -> Self {
        let backgrounds = background_stack([&bg::bg_brick_break]);

        let brick_objs = [
            Object::new(BRICK_BLUE.sprite(0)),
            Object::new(BRICK_GREEN.sprite(0)),
            Object::new(BRICK_YELLOW.sprite(0)),
            Object::new(BRICK_ORANGE.sprite(0)),
            Object::new(BRICK_RED.sprite(0)),
        ];

        let mut flipped_brick_objs = [
            Object::new(BRICK_BLUE.sprite(0)),
            Object::new(BRICK_GREEN.sprite(0)),
            Object::new(BRICK_YELLOW.sprite(0)),
            Object::new(BRICK_ORANGE.sprite(0)),
            Object::new(BRICK_RED.sprite(0)),
        ];

        flipped_brick_objs.iter_mut().for_each(|obj| {
            obj.set_hflip(true);
        });

        let paddle_cap_objs = [
            Object::new(BRICK_PADDLE_L.sprite(0)),
            Object::new(BRICK_PADDLE_R.sprite(0)),
        ];

        let paddle_len = 3;
        let paddle_x = 140;
        let paddle_w = (paddle_len + 2) * TILE_SIZE;
        let initial_pos = vec2(
            Fp::from(paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1)),
            Fp::from(PADDLE_Y - BALL_SIZE.y),
        );

        Self {
            paddle_len: paddle_len as u8,
            paddle_x,
            launched: false,
            bricks: make_bricks(1),
            empty_slots: vec![],
            backgrounds,
            brick_objs,
            flipped_brick_objs,
            paddle_cap_objs,
            paddle_speed: 0,
            paddle_dir: None,
            paddle_accel_timer: 0,
            lives: LIFE_COUNT as u8,
            level: 1,
            show_level_badge: true,
            powerup: None,
            extra_balls: vec![(initial_pos, vec2(1, -1))],
            rng: RandomNumberGenerator::new_with_seed(seed),
            trap_active: false,
            launch_timer: 0,
        }
    }
}

fn check_brick_collision(
    bricks: &mut Vec<Brick>,
    empty_slots: &mut Vec<Vector2D<i32>>,
    pos: &mut Vector2D<Fp>,
    vel: &mut Vector2D<i32>,
    prev_pos: Vector2D<i32>,
    sound_controller: &mut SoundController,
) -> bool {
    let ball_rect = Rect::new(vec2(pos.x.floor(), pos.y.floor()), BALL_SIZE);

    let mut impact = None;
    for (i, brick) in bricks.iter().enumerate() {
        if brick.rect.touches(ball_rect) {
            impact = Some(i);
            break;
        }
    }

    let Some(idx) = impact else { return false };
    let brick_rect = bricks[idx].rect;
    let prev_rect = Rect::new(prev_pos, BALL_SIZE);

    let mut invert_x = false;
    let mut invert_y = false;

    if prev_rect.bottom_right().x <= brick_rect.top_left().x {
        invert_x = true;
        pos.x = Fp::from(brick_rect.top_left().x - BALL_SIZE.x);
    } else if prev_rect.top_left().x >= brick_rect.bottom_right().x {
        invert_x = true;
        pos.x = Fp::from(brick_rect.bottom_right().x);
    }

    if prev_rect.bottom_right().y <= brick_rect.top_left().y {
        invert_y = true;
        pos.y = Fp::from(brick_rect.top_left().y - BALL_SIZE.y);
    } else if prev_rect.top_left().y >= brick_rect.bottom_right().y {
        invert_y = true;
        pos.y = Fp::from(brick_rect.bottom_right().y);
    }

    if !invert_x && !invert_y {
        let ball_mid = ball_rect.centre();
        let brick_mid = brick_rect.centre();

        let overlap_x = if ball_mid.x < brick_mid.x {
            ball_rect.bottom_right().x - brick_rect.top_left().x
        } else {
            brick_rect.bottom_right().x - ball_rect.top_left().x
        };

        let overlap_y = if ball_mid.y < brick_mid.y {
            ball_rect.bottom_right().y - brick_rect.top_left().y
        } else {
            brick_rect.bottom_right().y - ball_rect.top_left().y
        };

        if overlap_x < overlap_y {
            invert_x = true;
            if ball_mid.x < brick_mid.x {
                pos.x -= Fp::from(overlap_x);
            } else {
                pos.x += Fp::from(overlap_x);
            }
        } else {
            invert_y = true;
            if ball_mid.y < brick_mid.y {
                pos.y -= Fp::from(overlap_y);
            } else {
                pos.y += Fp::from(overlap_y);
            }
        }
    }

    if invert_x {
        vel.x *= -1;
    }
    if invert_y {
        vel.y *= -1;
    }

    if let Some(next) = bricks[idx].kind.next() {
        bricks[idx].kind = next;
        sound_controller.play_sfx(SoundEffect::BrickDamage);
    } else {
        empty_slots.push(brick_rect.position);
        bricks.remove(idx);
        sound_controller.play_sfx(SoundEffect::BrickBreak);
    }
    true
}

impl BricksState {
    fn ball_speed(&self) -> Fp {
        BALL_SPEEDS[(self.level - 1) as usize]
    }

    fn paddle_rect(&self) -> Rect<i32> {
        Rect::new(
            vec2(self.paddle_x, PADDLE_Y),
            vec2((self.paddle_len as i32 + 2) * TILE_SIZE, TILE_SIZE),
        )
    }
}

impl BricksState {
    pub fn cheat(&mut self) {
        if !self.launched {
            self.launched = true;
            self.show_level_badge = false;
            self.launch_timer = POST_LAUNCH_DELAY_PER_LEVEL * self.level;
            self.extra_balls[0].1 = vec2(-1, -1);
        }
        let pos = self.extra_balls[0].0;
        for _ in 0..5 {
            let x: i32 = match next_u16_in(&mut self.rng, 0, 3) {
                0 => -2,
                1 => -1,
                2 => 1,
                _ => 2,
            };
            self.extra_balls.push((pos, vec2(x, -1)));
        }
    }
    
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        let max_x = BOUNDS.bottom_right().x - ((self.paddle_len as i32 + 2) * TILE_SIZE);
        let desired_dir = if button_controller.is_pressed(Button::Left) {
            Some(false)
        } else if button_controller.is_pressed(Button::Right) {
            Some(true)
        } else {
            None
        };
        if desired_dir != self.paddle_dir {
            self.paddle_dir = desired_dir;
            self.paddle_speed = if desired_dir.is_some() { 1 } else { 0 };
            self.paddle_accel_timer = 0;
        } else if desired_dir.is_some() {
            self.paddle_accel_timer += 1;
            if self.paddle_accel_timer >= FRAMES_PER_ACCEL * self.paddle_speed as u8 {
                self.paddle_accel_timer = 0;
                self.paddle_speed = (self.paddle_speed + 1).min(PADDLE_MAX_SPEED);
            }
        }
        match self.paddle_dir {
            Some(true) => self.paddle_x = self.paddle_x.saturating_add(self.paddle_speed),
            Some(false) => self.paddle_x = self.paddle_x.saturating_sub(self.paddle_speed),
            None => {}
        }
        self.paddle_x = self.paddle_x.clamp(BOUNDS.position.x, max_x);

        self.launch_timer = self.launch_timer.saturating_sub(1);

        let paddle_rect = self.paddle_rect();
        let speed = self.ball_speed();
        let left = BOUNDS.top_left().x;
        let right = BOUNDS.bottom_right().x;
        let top = BOUNDS.top_left().y;
        let bottom = BOUNDS.bottom_right().y;

        if !self.launched {
            self.trap_active = false;
            let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
            self.extra_balls[0].0.x =
                Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1));
            self.extra_balls[0].0.y = Fp::from(PADDLE_Y - BALL_SIZE.y);
            if button_controller.is_just_pressed(Button::A) {
                self.launched = true;
                self.show_level_badge = false;
                self.launch_timer = POST_LAUNCH_DELAY_PER_LEVEL * self.level;
                let x = match self.paddle_dir {
                    Some(dir) => {
                        let sign: i32 = if dir { 1 } else { -1 };
                        sign * if self.paddle_speed > (PADDLE_MAX_SPEED >> 1) {
                            2
                        } else {
                            1
                        }
                    }
                    None => -1,
                };
                self.extra_balls[0].1 = vec2(x, -1);
            }
        } else {
            self.trap_active = self.launch_timer == 0 && button_controller.is_pressed(Button::A);

            let mut has_bounced = false;
            let mut pos = self.extra_balls[0].0;
            let mut vel = self.extra_balls[0].1;
            let prev_pos = vec2(pos.x.floor(), pos.y.floor());

            pos.x += Fp::from(vel.x) * speed;
            pos.y += Fp::from(vel.y) * speed;

            let ball_rect = Rect::new(vec2(pos.x.floor(), pos.y.floor()), BALL_SIZE);
            if vel.y > 0 && paddle_rect.touches(ball_rect) {
                let captured = self.trap_active && {
                    let start_x = self.paddle_x + 8;
                    let end_x = start_x + (self.paddle_len as i32 * 8);
                    (start_x..end_x).contains(&pos.x.round())
                };
                if captured {
                    self.launched = false;
                    self.trap_active = false;
                    let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
                    self.extra_balls[0].0.x =
                        Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1));
                    self.extra_balls[0].0.y = Fp::from(PADDLE_Y - BALL_SIZE.y);
                } else {
                    if pos.y.floor() > PADDLE_Y {
                        vel.x *= 2;
                    } else if let Some(dir) = self.paddle_dir {
                        let sign: i32 = if dir { 1 } else { -1 };
                        vel.x = sign
                            * if self.paddle_speed > (PADDLE_MAX_SPEED >> 1) {
                                2
                            } else {
                                1
                            };
                    } else if next_u16_in(&mut self.rng, 0, 3) == 0 {
                        vel.x = -vel.x.abs();
                    }
                    pos.y = Fp::from(paddle_rect.top_left().y - BALL_SIZE.y);
                    vel.y = -vel.y.abs();
                    has_bounced = true;
                }
            }

            if self.launched {
                let hit_brick = check_brick_collision(
                    &mut self.bricks,
                    &mut self.empty_slots,
                    &mut pos,
                    &mut vel,
                    prev_pos,
                    sound_controller,
                );

                if pos.x.floor() < left {
                    pos.x = Fp::from(left);
                    vel.x *= -1;
                    has_bounced = true;
                } else if pos.x.floor() + BALL_SIZE.x > right {
                    pos.x = Fp::from(right - BALL_SIZE.x);
                    vel.x *= -1;
                    has_bounced = true;
                }

                if pos.y.floor() < top {
                    pos.y = Fp::from(top);
                    vel.y *= -1;
                    has_bounced = true;
                } else if pos.y.floor() + BALL_SIZE.y > bottom {
                    if self.extra_balls.len() > 1 {
                        self.extra_balls.remove(0);
                        pos = self.extra_balls[0].0;
                        vel = self.extra_balls[0].1;
                    } else {
                        sound_controller.play_sfx(SoundEffect::BrickFloor);
                        self.lives = self.lives.saturating_sub(1);
                        if self.lives == 0 {
                            return Some(SceneAction::Lose);
                        }
                        self.launched = false;
                        let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
                        pos.x = Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1));
                        pos.y = Fp::from(PADDLE_Y - BALL_SIZE.y);
                    }
                }

                if has_bounced && !hit_brick {
                    sound_controller.play_sfx(SoundEffect::BrickBounce);
                }

                let current_ball_rect = Rect::new(vec2(pos.x.floor(), pos.y.floor()), BALL_SIZE);

                if let Some((powerup, bounds, frames)) = self.powerup {
                    if current_ball_rect.touches(bounds) {
                        match powerup {
                            Powerup::Life => self.lives += 1,
                            Powerup::Paddle => self.paddle_len += 1,
                            Powerup::Ball => {
                                let second_x = if vel.x >= 0 { -1 } else { 1 };
                                self.extra_balls.push((pos, vec2(second_x, -1)));
                            }
                        }
                        sound_controller.play_sfx(SoundEffect::PowerUp);
                        self.powerup = None;
                    } else if frames == 0 {
                        self.powerup = None;
                    } else {
                        self.powerup = Some((powerup, bounds, frames - 1));
                    }
                } else if !self.empty_slots.is_empty() {
                    let rand_num = next_u16_in(&mut self.rng, 0, 10000);
                    let life = self.lives < MAX_LIVES && {
                        let denom = EXTRA_LIFE_DENOMS[(self.lives - 1) as usize];
                        next_u16_in(&mut self.rng, 0, denom - 1) < 2
                    };
                    let paddle = self.paddle_len < MAX_LEN && rand_num == 1;
                    let ball = rand_num < 3;

                    let mut candidates = [Powerup::Life; 3];
                    let mut count = 0u16;
                    if life {
                        candidates[count as usize] = Powerup::Life;
                        count += 1;
                    }
                    if paddle {
                        candidates[count as usize] = Powerup::Paddle;
                        count += 1;
                    }
                    if ball {
                        candidates[count as usize] = Powerup::Ball;
                        count += 1;
                    }
                    if count > 0 {
                        let chosen = candidates[next_u16_in(&mut self.rng, 0, count - 1) as usize];
                        let slot_pos = self.empty_slots[next_u16_in(
                            &mut self.rng,
                            0,
                            (self.empty_slots.len() - 1) as u16,
                        ) as usize];
                        self.powerup =
                            Some((chosen, Rect::new(slot_pos, POWERUP_SIZE), POWERUP_FRAMES));
                    }
                }

                self.extra_balls[0] = (pos, vel);
            }
        }

        let secondary: Vec<_> = self.extra_balls.drain(1..).collect();
        let mut surviving_balls = vec![];
        for (mut pos2, mut vel2) in secondary {
            let prev_pos2 = vec2(pos2.x.floor(), pos2.y.floor());
            pos2.x += Fp::from(vel2.x) * speed;
            pos2.y += Fp::from(vel2.y) * speed;

            let ball_rect2 = Rect::new(vec2(pos2.x.floor(), pos2.y.floor()), BALL_SIZE);
            if vel2.y > 0 && paddle_rect.touches(ball_rect2) {
                if let Some(dir) = self.paddle_dir {
                    let sign: i32 = if dir { 1 } else { -1 };
                    vel2.x = sign
                        * if self.paddle_speed > (PADDLE_MAX_SPEED >> 1) {
                            2
                        } else {
                            1
                        };
                }
                pos2.y = Fp::from(paddle_rect.top_left().y - BALL_SIZE.y);
                vel2.y = -vel2.y.abs();
                sound_controller.play_sfx(SoundEffect::BrickBounce);
            }

            check_brick_collision(
                &mut self.bricks,
                &mut self.empty_slots,
                &mut pos2,
                &mut vel2,
                prev_pos2,
                sound_controller,
            );

            if pos2.x.floor() < left {
                pos2.x = Fp::from(left);
                vel2.x *= -1;
            } else if pos2.x.floor() + BALL_SIZE.x > right {
                pos2.x = Fp::from(right - BALL_SIZE.x);
                vel2.x *= -1;
            }
            let fallen = pos2.y.floor() + BALL_SIZE.y > bottom;
            if pos2.y.floor() < top {
                pos2.y = Fp::from(top);
                vel2.y *= -1;
            }

            if let Some((powerup, bounds, _)) = self.powerup {
                let ball_rect2 = Rect::new(vec2(pos2.x.floor(), pos2.y.floor()), BALL_SIZE);
                if ball_rect2.touches(bounds) {
                    match powerup {
                        Powerup::Life => self.lives += 1,
                        Powerup::Paddle => self.paddle_len += 1,
                        Powerup::Ball => {
                            let second_x = if vel2.x >= 0 { -1 } else { 1 };
                            surviving_balls.push((pos2, vec2(second_x, -1)));
                        }
                    }
                    sound_controller.play_sfx(SoundEffect::PowerUp);
                    self.powerup = None;
                }
            }

            if !fallen {
                surviving_balls.push((pos2, vel2));
            }
        }
        self.extra_balls.extend(surviving_balls);

        if self.bricks.is_empty() {
            if self.level >= 12 {
                return Some(SceneAction::Win);
            } else {
                self.level += 1;
                self.bricks = make_bricks(self.level);
                self.launched = false;
                self.show_level_badge = true;
                self.powerup = None;
                sound_controller.play_sfx(SoundEffect::LevelUp);
                self.empty_slots.clear();
                let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
                let reset_pos = vec2(
                    Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1)),
                    Fp::from(PADDLE_Y - BALL_SIZE.y),
                );
                self.extra_balls.clear();
                self.extra_balls.push((reset_pos, vec2(1, -1)));
            }
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        self.backgrounds[0].show(frame);

        if !self.launched {
            sprites::SERVE_LABEL
                .sprites()
                .iter()
                .enumerate()
                .for_each(|(i, s)| {
                    let pos = SERVE_OFFSET + vec2(16 * i as i32, 0);
                    Object::new(s).set_pos(pos).show(frame);
                });

            if self.show_level_badge {
                for i in 0..3 {
                    Object::new(LEVEL_BADGE.sprite(i as usize))
                        .set_pos(LVL_BADGE_OFFSET + vec2(32 * i, 0))
                        .show(frame);
                }
                Object::new(LEVEL_NUMS.sprite((self.level - 1) as usize))
                    .set_pos(LVL_NUM_OFFSET)
                    .show(frame);
            }
        }

        for brick in &self.bricks {
            self.brick_objs[brick.kind.idx()]
                .set_pos(brick.rect.position)
                .show(frame);
            self.flipped_brick_objs[brick.kind.idx()]
                .set_pos(brick.rect.position + FLIP_OFFSET)
                .show(frame);
        }

        if let Some((powerup, rect, frames)) = &mut self.powerup {
            let visible = if *frames <= POWERUP_BLINK_FAST {
                *frames & 8 != 0
            } else if *frames <= POWERUP_BLINK_SLOW {
                *frames & 16 != 0
            } else {
                true
            };
            if visible {
                powerup.sprite().show(rect.position, frame);
            }
        }

        for (pos, _) in &self.extra_balls {
            Object::new(BRICK_BALL.sprite(0))
                .set_pos(vec2(pos.x.floor(), pos.y.floor()))
                .show(frame);
        }

        let mut x = self.paddle_x;
        self.paddle_cap_objs[0]
            .set_pos(vec2(x, PADDLE_Y))
            .show(frame);
        for _ in 0..self.paddle_len {
            x += TILE_SIZE;
            BRICK_PADDLE_M.show(0, vec2(x, PADDLE_Y), frame);
        }
        self.paddle_cap_objs[1]
            .set_pos(vec2(x + TILE_SIZE, PADDLE_Y))
            .show(frame);

        if self.trap_active {
            let y = PADDLE_Y - 8;
            for i in 0..self.paddle_len {
                sprites::BRICK_TRAP.show(0, vec2(self.paddle_x + 8 + (i as i32 * 8), y), frame);
            }
        }

        let life_start_x = WIDTH - (LIFE_STRIDE * self.lives as i32);
        for i in 0..self.lives as i32 {
            Object::new(BRICK_BALL.sprite(0))
                .set_pos(vec2(life_start_x + i * LIFE_STRIDE, LIFE_Y))
                .show(frame);
        }
    }
}
