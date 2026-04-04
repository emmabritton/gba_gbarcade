use crate::game_result::GameResult;
use crate::gfx::background_stack;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use crate::TILE_SIZE;
use agb::display::object::Object;
use agb::display::tiled::RegularBackground;
use agb::display::{GraphicsFrame, WIDTH};
use agb::fixnum::{vec2, Rect, Vector2D};
use agb::input::{Button, ButtonController};
use agb::println;
use alloc::vec;
use alloc::vec::Vec;
use resources::sprites::{
    BRICK_BALL, BRICK_BLUE, BRICK_GREEN, BRICK_ORANGE, BRICK_PADDLE_L, BRICK_PADDLE_M,
    BRICK_PADDLE_R, BRICK_RED, BRICK_YELLOW,
};
use resources::{bg, sprites};

const PADDLE_Y: i32 = 149;
const BOUNDS: Rect<i32> = Rect {
    position: vec2(2, 11),
    size: vec2(236, 147),
};
const FLIP_OFFSET: Vector2D<i32> = vec2(16, 0);
const BALL_SIZE: Vector2D<i32> = vec2(4, 4);
const BRICK_OFFSET: Vector2D<i32> = vec2(3, 3);

const LIFE_Y: i32 = 4;
const LIFE_X0: i32 = WIDTH - (LIFE_STRIDE * 3);
const LIFE_STRIDE: i32 = 8;

const SERVE_OFFSET: Vector2D<i32> = vec2(80, 120);

#[derive(Debug)]
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
    ball_pos: Vector2D<i32>,
    ball_vel: Vector2D<i32>,
    launched: bool,
    bricks: Vec<Brick>,
    backgrounds: [RegularBackground; 1],
    brick_objs: [Object; 5],
    flipped_brick_objs: [Object; 5],
    ball_obj: Object,
    paddle_cap_objs: [Object; 2],
    lives: u8,
    game_result: Option<GameResult>,
}

fn make_bricks() -> Vec<Brick> {
    let mut bricks = vec![];

    for y in 0..5 {
        for x in 0..10 {
            let kind = match y {
                0 => BrickKind::Red,
                1 => BrickKind::Orange,
                2 => BrickKind::Yellow,
                3 => BrickKind::Green,
                4 => BrickKind::Blue,
                _ => unreachable!(),
            };
            let rect = Rect::new(
                vec2(
                    BOUNDS.position.x + x * 40 + BRICK_OFFSET.x,
                    BOUNDS.position.y + y * 12 + BRICK_OFFSET.y,
                ),
                vec2(32, 10),
            );
            bricks.push(Brick { kind, rect });
        }
    }

    bricks
}

impl BricksState {
    pub fn new() -> Self {
        let backgrounds = background_stack([&bg::bg_brick_break]);

        let ball_obj = Object::new(BRICK_BALL.sprite(0));

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
            paddle_x + paddle_w / 2 - BALL_SIZE.x / 2,
            PADDLE_Y - BALL_SIZE.y,
        );

        Self {
            ball_obj,
            paddle_len: 1,
            paddle_x,
            ball_pos,
            ball_vel: vec2(1, -1),
            launched: false,
            bricks: make_bricks(),
            backgrounds,
            brick_objs,
            flipped_brick_objs,
            paddle_cap_objs,
            lives: 3,
            game_result: None,
        }
    }
}

impl BricksState {
    fn ball_rect(&self) -> Rect<i32> {
        Rect::new(self.ball_pos, BALL_SIZE)
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
        if let Some(result) = self.game_result.take() {
            let result = result.update();
            let input = result.input_allowed();
            self.game_result = Some(result);
            if input
                && (button_controller.is_just_pressed(Button::A)
                    || button_controller.is_just_pressed(Button::B)
                    || button_controller.is_just_pressed(Button::Start))
            {
                return Some(SceneAction::Menu);
            }
            return None;
        }

        let max_x = BOUNDS.bottom_right().x - ((self.paddle_len as i32 + 2) * TILE_SIZE);
        if button_controller.is_pressed(Button::Left) {
            self.paddle_x = self.paddle_x.saturating_sub(2);
        }
        if button_controller.is_pressed(Button::Right) {
            self.paddle_x = self.paddle_x.saturating_add(2);
        }
        self.paddle_x = self.paddle_x.clamp(BOUNDS.position.x, max_x);

        if !self.launched {
            let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
            self.ball_pos.x = self.paddle_x + paddle_w / 2 - BALL_SIZE.x / 2;
            self.ball_pos.y = PADDLE_Y - BALL_SIZE.y;
            if button_controller.is_just_pressed(Button::A) {
                self.launched = true;
            }
            return None;
        }

        let mut has_bounced = false;
        let mut hit_brick = false;
        let prev_pos = self.ball_pos;

        self.ball_pos += self.ball_vel;

        let ball_rect = self.ball_rect();
        let paddle_rect = self.paddle_rect();
        if self.ball_vel.y > 0 && paddle_rect.touches(ball_rect) {
            self.ball_pos.y = paddle_rect.top_left().y - BALL_SIZE.y;

            let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
            let paddle_center_x = self.paddle_x + paddle_w / 2;
            let ball_center_x = self.ball_pos.x + BALL_SIZE.x / 2;
            let rel = ball_center_x - paddle_center_x;

            let mut vx = (rel * 2) / (paddle_w / 2).max(1);
            if vx == 0 {
                vx = if rel < 0 { -1 } else { 1 };
            }
            vx = vx.clamp(-2, 2);

            self.ball_vel.x = vx;
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
                self.ball_pos.x = brick_rect.top_left().x - BALL_SIZE.x;
            } else if prev_rect.top_left().x >= brick_rect.bottom_right().x {
                invert_x = true;
                self.ball_pos.x = brick_rect.bottom_right().x;
            }

            if prev_rect.bottom_right().y <= brick_rect.top_left().y {
                invert_y = true;
                self.ball_pos.y = brick_rect.top_left().y - BALL_SIZE.y;
            } else if prev_rect.top_left().y >= brick_rect.bottom_right().y {
                invert_y = true;
                self.ball_pos.y = brick_rect.bottom_right().y;
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
                        self.ball_pos.x -= overlap_x;
                    } else {
                        self.ball_pos.x += overlap_x;
                    }
                } else {
                    invert_y = true;
                    if ball_mid.y < brick_mid.y {
                        self.ball_pos.y -= overlap_y;
                    } else {
                        self.ball_pos.y += overlap_y;
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
                self.bricks.remove(idx);
                sound_controller.play_sfx(SoundEffect::BrickBreak);
            }
            hit_brick = true;
        }

        let left = BOUNDS.top_left().x;
        let right = BOUNDS.bottom_right().x;
        let top = BOUNDS.top_left().y;
        let bottom = BOUNDS.bottom_right().y;

        if self.ball_pos.x < left {
            self.ball_pos.x = left;
            self.ball_vel.x *= -1;
            has_bounced = true;
        } else if self.ball_pos.x + BALL_SIZE.x > right {
            self.ball_pos.x = right - BALL_SIZE.x;
            self.ball_vel.x *= -1;
            has_bounced = true;
        }

        if self.ball_pos.y < top {
            self.ball_pos.y = top;
            self.ball_vel.y *= -1;
            has_bounced = true;
        } else if self.ball_pos.y + BALL_SIZE.y > bottom {
            sound_controller.play_sfx(SoundEffect::BrickFloor);
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                sound_controller.play_sfx(SoundEffect::Lose);
                self.game_result = Some(GameResult::new_lose());
            }
            self.launched = false;
            let paddle_w = (self.paddle_len as i32 + 2) * TILE_SIZE;
            self.ball_pos.x = self.paddle_x + paddle_w / 2 - BALL_SIZE.x / 2;
            self.ball_pos.y = PADDLE_Y - BALL_SIZE.y;
        }

        if has_bounced && !hit_brick {
            sound_controller.play_sfx(SoundEffect::BrickBounce);
        }

        if self.bricks.is_empty() {
            sound_controller.play_sfx(SoundEffect::Win);
            self.game_result = Some(GameResult::new_win());
        }

        if button_controller.is_just_pressed(Button::Start) {
            println!("{:?}", self.bricks);
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        self.backgrounds[0].show(frame);

        if let Some(result) = &self.game_result {
            result.show(frame);
        }

        if !self.launched {
            sprites::SERVE_LABEL
                .sprites()
                .iter()
                .enumerate()
                .for_each(|(i, s)| {
                    let pos = SERVE_OFFSET + vec2(16 * i as i32, 0);
                    Object::new(s).set_pos(pos).show(frame);
                });
        }

        for brick in &self.bricks {
            self.brick_objs[brick.kind.idx()]
                .set_pos(brick.rect.position)
                .show(frame);
            self.flipped_brick_objs[brick.kind.idx()]
                .set_pos(brick.rect.position + FLIP_OFFSET)
                .show(frame);
        }

        self.ball_obj.set_pos(self.ball_pos).show(frame);

        let mut x = self.paddle_x;
        self.paddle_cap_objs[0]
            .set_pos(vec2(x, PADDLE_Y))
            .show(frame);
        for _ in 0..self.paddle_len {
            x += TILE_SIZE;
            Object::new(BRICK_PADDLE_M.sprite(0))
                .set_pos(vec2(x, PADDLE_Y))
                .show(frame);
        }
        self.paddle_cap_objs[1]
            .set_pos(vec2(x + TILE_SIZE, PADDLE_Y))
            .show(frame);

        for i in 0..self.lives as i32 {
            Object::new(BRICK_BALL.sprite(0))
                .set_pos(vec2(LIFE_X0 + i * LIFE_STRIDE, LIFE_Y))
                .show(frame);
        }
    }
}
