use crate::TILE_SIZE;
use crate::gfx::{ShowTag, background_stack};
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

const PADDLE_SPEED: i32 = 4;
const PADDLE_Y: i32 = 149;
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

const EXTRA_LIFE_FRAMES: u16 = 1800; //30s
const EXTRA_LIFE_BLINK_SLOW: u16 = 1200; // 20s remaining
const EXTRA_LIFE_BLINK_FAST: u16 = 600; // 10s remaining
const EXTRA_LIFE_SIZE: Vector2D<i32> = vec2(32, 16);
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

const EXTRA_LIFE: &Sprite = sprites::BRICKS_EXTRA_LIFE.sprite(0);

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
    paddle_len: usize,
    paddle_x: i32,
    ball_pos: Vector2D<Fp>,
    ball_vel: Vector2D<i32>,
    launched: bool,
    bricks: Vec<Brick>,
    empty_slots: Vec<Vector2D<i32>>,
    backgrounds: [RegularBackground; 1],
    brick_objs: [Object; 5],
    flipped_brick_objs: [Object; 5],
    ball_obj: Object,
    extra_life_obj: Object,
    paddle_cap_objs: [Object; 2],
    lives: u8,
    level: u8,
    show_level_badge: bool,
    extra_life: Option<(Rect<i32>, u16)>,
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

        let ball_obj = Object::new(BRICK_BALL.sprite(0));
        let extra_life_obj = Object::new(EXTRA_LIFE);

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

        let paddle_x = 140;
        let paddle_w = (1 + 2) * TILE_SIZE;
        let ball_pos = vec2(
            Fp::from(paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1)),
            Fp::from(PADDLE_Y - BALL_SIZE.y),
        );

        Self {
            ball_obj,
            extra_life_obj,
            paddle_len: 3,
            paddle_x,
            ball_pos,
            ball_vel: vec2(1, -1),
            launched: false,
            bricks: make_bricks(1),
            empty_slots: vec![],
            backgrounds,
            brick_objs,
            flipped_brick_objs,
            paddle_cap_objs,
            lives: LIFE_COUNT as u8,
            level: 1,
            show_level_badge: true,
            extra_life: None,
            rng: RandomNumberGenerator::new_with_seed(seed),
            trap_active: false,
            launch_timer: 0,
        }
    }
}

impl BricksState {
    fn ball_rect(&self) -> Rect<i32> {
        Rect::new(
            vec2(self.ball_pos.x.floor(), self.ball_pos.y.floor()),
            BALL_SIZE,
        )
    }

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
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        let max_x = BOUNDS.bottom_right().x - ((self.paddle_len as i32 + 2) * TILE_SIZE);
        if button_controller.is_pressed(Button::Left) {
            self.paddle_x = self.paddle_x.saturating_sub(PADDLE_SPEED);
        }
        if button_controller.is_pressed(Button::Right) {
            self.paddle_x = self.paddle_x.saturating_add(PADDLE_SPEED);
        }
        self.paddle_x = self.paddle_x.clamp(BOUNDS.position.x, max_x);

        self.launch_timer = self.launch_timer.saturating_sub(1);

        if !self.launched {
            self.trap_active = false;
            let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
            self.ball_pos.x = Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1));
            self.ball_pos.y = Fp::from(PADDLE_Y - BALL_SIZE.y);
            if button_controller.is_just_pressed(Button::A) {
                self.launched = true;
                self.show_level_badge = false;
                self.launch_timer = POST_LAUNCH_DELAY_PER_LEVEL * self.level;
            }
            return None;
        } else {
            self.trap_active = self.launch_timer == 0 && button_controller.is_pressed(Button::A);
        }

        let mut has_bounced = false;
        let mut hit_brick = false;
        let prev_pos = vec2(self.ball_pos.x.floor(), self.ball_pos.y.floor());

        let speed = self.ball_speed();
        self.ball_pos.x += Fp::from(self.ball_vel.x) * speed;
        self.ball_pos.y += Fp::from(self.ball_vel.y) * speed;

        let ball_rect = self.ball_rect();
        let paddle_rect = self.paddle_rect();
        if self.ball_vel.y > 0 && paddle_rect.touches(ball_rect) {
            if self.trap_active {
                let start_x = self.paddle_x + 8;
                let end_x = start_x + (self.paddle_len as i32 * 8);
                if (start_x..end_x).contains(&self.ball_pos.x.round()) {
                    self.launched = false;
                    return None;
                }
            }

            if self.ball_pos.y.floor() > PADDLE_Y {
                self.ball_vel.x *= 2;
            }

            self.ball_pos.y = Fp::from(paddle_rect.top_left().y - BALL_SIZE.y);

            if next_u16_in(&mut self.rng, 0, 3) == 0 {
                self.ball_vel.x = -self.ball_vel.x.abs();
            }
            self.ball_vel.y = -self.ball_vel.y.abs();
            has_bounced = true;
        }

        let ball_rect = self.ball_rect();

        let mut impact = None;
        for (i, brick) in self.bricks.iter().enumerate() {
            if brick.rect.touches(ball_rect) {
                impact = Some(i);
                break;
            }
        }

        if let Some(idx) = impact {
            let brick_rect = self.bricks[idx].rect;
            let prev_rect = Rect::new(prev_pos, BALL_SIZE);

            let mut invert_x = false;
            let mut invert_y = false;

            if prev_rect.bottom_right().x <= brick_rect.top_left().x {
                invert_x = true;
                self.ball_pos.x = Fp::from(brick_rect.top_left().x - BALL_SIZE.x);
            } else if prev_rect.top_left().x >= brick_rect.bottom_right().x {
                invert_x = true;
                self.ball_pos.x = Fp::from(brick_rect.bottom_right().x);
            }

            if prev_rect.bottom_right().y <= brick_rect.top_left().y {
                invert_y = true;
                self.ball_pos.y = Fp::from(brick_rect.top_left().y - BALL_SIZE.y);
            } else if prev_rect.top_left().y >= brick_rect.bottom_right().y {
                invert_y = true;
                self.ball_pos.y = Fp::from(brick_rect.bottom_right().y);
            }

            if !invert_x && !invert_y {
                let ball_left = ball_rect.top_left().x;
                let ball_right = ball_rect.bottom_right().x;
                let ball_top = ball_rect.top_left().y;
                let ball_bottom = ball_rect.bottom_right().y;

                let brick_left = brick_rect.top_left().x;
                let brick_right = brick_rect.bottom_right().x;
                let brick_top = brick_rect.top_left().y;
                let brick_bottom = brick_rect.bottom_right().y;

                let ball_mid = ball_rect.centre();
                let brick_mid = brick_rect.centre();

                let overlap_x = if ball_mid.x < brick_mid.x {
                    ball_right - brick_left
                } else {
                    brick_right - ball_left
                };

                let overlap_y = if ball_mid.y < brick_mid.y {
                    ball_bottom - brick_top
                } else {
                    brick_bottom - ball_top
                };

                if overlap_x < overlap_y {
                    invert_x = true;
                    if ball_mid.x < brick_mid.x {
                        self.ball_pos.x -= Fp::from(overlap_x);
                    } else {
                        self.ball_pos.x += Fp::from(overlap_x);
                    }
                } else {
                    invert_y = true;
                    if ball_mid.y < brick_mid.y {
                        self.ball_pos.y -= Fp::from(overlap_y);
                    } else {
                        self.ball_pos.y += Fp::from(overlap_y);
                    }
                }
            }

            if invert_x {
                self.ball_vel.x *= -1;
            }
            if invert_y {
                self.ball_vel.y *= -1;
            }

            if let Some(next) = self.bricks[idx].kind.next() {
                self.bricks[idx].kind = next;
                sound_controller.play_sfx(SoundEffect::BrickDamage);
            } else {
                self.empty_slots.push(brick_rect.position);
                self.bricks.remove(idx);
                sound_controller.play_sfx(SoundEffect::BrickBreak);
            }
            hit_brick = true;
        }

        let left = BOUNDS.top_left().x;
        let right = BOUNDS.bottom_right().x;
        let top = BOUNDS.top_left().y;
        let bottom = BOUNDS.bottom_right().y;

        if self.ball_pos.x.floor() < left {
            self.ball_pos.x = Fp::from(left);
            self.ball_vel.x *= -1;
            has_bounced = true;
        } else if self.ball_pos.x.floor() + BALL_SIZE.x > right {
            self.ball_pos.x = Fp::from(right - BALL_SIZE.x);
            self.ball_vel.x *= -1;
            has_bounced = true;
        }

        if self.ball_pos.y.floor() < top {
            self.ball_pos.y = Fp::from(top);
            self.ball_vel.y *= -1;
            has_bounced = true;
        } else if self.ball_pos.y.floor() + BALL_SIZE.y > bottom {
            sound_controller.play_sfx(SoundEffect::BrickFloor);
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                return Some(SceneAction::Lose);
            }
            self.launched = false;
            let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
            self.ball_pos.x = Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1));
            self.ball_pos.y = Fp::from(PADDLE_Y - BALL_SIZE.y);
        }

        if has_bounced && !hit_brick {
            sound_controller.play_sfx(SoundEffect::BrickBounce);
        }

        let current_ball_rect = self.ball_rect();
        if let Some((rect, ref mut frames)) = self.extra_life {
            if current_ball_rect.touches(rect) {
                self.lives = self.lives.saturating_add(1).min(MAX_LIVES);
                self.extra_life = None;
                sound_controller.play_sfx(SoundEffect::ExtraLife);
            } else if *frames == 0 {
                self.extra_life = None;
            } else {
                *frames -= 1;
            }
        } else if self.lives > 0 && self.lives < MAX_LIVES && !self.empty_slots.is_empty() {
            let denom = EXTRA_LIFE_DENOMS[(self.lives - 1) as usize];
            if next_u16_in(&mut self.rng, 0, denom - 1) == 0 {
                let slot_idx =
                    next_u16_in(&mut self.rng, 0, (self.empty_slots.len() - 1) as u16) as usize;
                let pos = self.empty_slots[slot_idx];
                self.extra_life = Some((Rect::new(pos, EXTRA_LIFE_SIZE), EXTRA_LIFE_FRAMES));
            }
        }

        if self.bricks.is_empty() {
            if self.level >= 9 {
                return Some(SceneAction::Win);
            } else {
                self.level += 1;
                self.bricks = make_bricks(self.level);
                self.launched = false;
                self.show_level_badge = true;
                self.extra_life = None;
                sound_controller.play_sfx(SoundEffect::LevelUp);
                self.empty_slots.clear();
                self.ball_vel = vec2(1, -1);
                let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
                self.ball_pos.x = Fp::from(self.paddle_x + (paddle_w >> 1) - (BALL_SIZE.x >> 1));
                self.ball_pos.y = Fp::from(PADDLE_Y - BALL_SIZE.y);
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

        if let Some((rect, frames)) = self.extra_life {
            let visible = if frames <= EXTRA_LIFE_BLINK_FAST {
                frames & 8 != 0
            } else if frames <= EXTRA_LIFE_BLINK_SLOW {
                frames & 16 != 0
            } else {
                true
            };
            if visible {
                self.extra_life_obj.set_pos(rect.position).show(frame);
            }
        }

        self.ball_obj
            .set_pos(vec2(self.ball_pos.x.floor(), self.ball_pos.y.floor()))
            .show(frame);

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
