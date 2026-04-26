use crate::gfx::background;
use crate::progress::{Achievement, set_achievement};
use crate::rng::next_u16_in;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::object::Object;
use agb::display::tile_data::TileData;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{GraphicsFrame, Priority, WIDTH};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use agb::rng::RandomNumberGenerator;
use resources::{bg, sprites};

// Tiles are 8x8 px; each tile covers two half-columns with health 0–3 each.
const BRICK_TILESET: &TileData = &bg::bg_invaders_bricks;

// 4 bases, 3 tiles wide x 2 tiles tall (24x16 px).
// Columns: 3-5, 10-12, 17-19, 24-26 -> 16 px left/right margins, 32 px gaps.
// Tile row 15 = pixel y 120, between the invader grid and the player.
const NUM_BASES: usize = 4;
const BASE_COLS: usize = 3;
const BASE_ROWS: usize = 2;
const BASE_TILE_Y: usize = 15;
const BASE_TILE_X: [usize; NUM_BASES] = [3, 10, 17, 24];

const COLS: u32 = 11;
const ROWS: u32 = 6;
const MAX_INVADERS: u32 = COLS * ROWS;
const MAX_SHOTS: usize = 3;

const COL_STRIDE: i32 = 16;
const ROW_STRIDE: i32 = 10;
const INV_W: i32 = 16;
const INV_H: i32 = 8;
const SHOT_W: i32 = 8;
const SHOT_H: i32 = 8;

// Grid width = (COLS-1)*COL_STRIDE + INV_W = 10*18+16 = 196. Centred on 240 -> start at 22.
const GRID_INIT_X: i32 = 22;
const GRID_INIT_Y: i32 = 20;

const LEFT_WALL: i32 = 2;
const RIGHT_WALL: i32 = WIDTH - LEFT_WALL;

const ADVANCE_STEP: i32 = 2;
const DROP_AMOUNT: i32 = 2;

const DANGER_Y: i32 = 140;

const PLAYER_Y: i32 = 150;
const PLAYER_W: i32 = 16;
const PLAYER_H: i32 = 8;
const PLAYER_INIT_X: i32 = 112;
const PLAYER_SPEED: i32 = 1;
const PLAYER_LEFT_LIMIT: i32 = LEFT_WALL;
const PLAYER_RIGHT_LIMIT: i32 = RIGHT_WALL - PLAYER_W;

const PLAYER_SHOT_SPEED: i32 = 4;
const INV_SHOT_SPEED: i32 = 2;
const PLAYER_SHOT_Y_LIMIT: i32 = 9;
const UFO_SHOT_Y_LIMIT: i32 = 152;

const UFO_Y: i32 = 10;
const UFO_W: i32 = 16;
const UFO_H: i32 = 8;
const UFO_SCORE: u32 = 100;
const UFO_INTERVAL: u16 = 1000;
const UFO_SPEED: i32 = 1;

const LIFE_Y: i32 = 0;
const LIFE_STRIDE: i32 = 10;

const DEAD_FRAMES: u8 = 30;
const PLAYER_DEAD_FRAMES: u16 = 90;

// Top 2 rows -> type 0 (30 pts), middle 2 -> type 1 (20 pts), bottom 2 -> type 2 (10 pts).
const fn row_type(row: u32) -> usize {
    if row < 2 {
        0
    } else if row < 4 {
        1
    } else {
        2
    }
}

const fn type_score(t: usize) -> u32 {
    match t {
        0 => 30,
        1 => 20,
        _ => 10,
    }
}

/// Frames between grid advances; decreases as invaders are eliminated.
fn advance_period(alive: u32) -> u16 {
    let alive = alive.max(1) as u16;
    5 + alive.saturating_mul(25) / (MAX_INVADERS as u16)
}

// HEALTH_MAP[(left_health << 2) | right_health] -> tileset tile index.
// Health 3 = full, 0 = destroyed. Rows = left health 0..3, columns = right health 0..3.
//
//        right=0  right=1  right=2  right=3
// left=0:   15      10       9        8     (left destroyed)
// left=1:    5      12      14        7
// left=2:    4      13      11        6
// left=3:    3       2       1        0     (both full)
const HEALTH_MAP: [usize; 16] = [15, 10, 9, 8, 5, 12, 14, 7, 4, 13, 11, 6, 3, 2, 1, 0];

/// Returns `(base, row, col, half)` if pixel point (x, y) falls inside a base tile.
/// `half` is 0 for the left 4 px of the tile, 1 for the right 4 px.
fn find_base_tile(x: i32, y: i32) -> Option<(usize, usize, usize, usize)> {
    if x < 0 || y < 0 {
        return None;
    }
    let tile_x = (x / 8) as usize;
    let tile_y = (y / 8) as usize;
    let half = ((x & 7) / 4) as usize;

    if !(BASE_TILE_Y..BASE_TILE_Y + BASE_ROWS).contains(&tile_y) {
        return None;
    }
    let row = tile_y - BASE_TILE_Y;

    for (b, bx) in BASE_TILE_X.iter().enumerate().take(NUM_BASES) {
        if tile_x >= *bx && tile_x < bx + BASE_COLS {
            return Some((b, row, tile_x - bx, half));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn overlaps(ax: i32, ay: i32, aw: i32, ah: i32, bx: i32, by: i32, bw: i32, bh: i32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn inv_pixel_pos(grid_x: i32, grid_y: i32, col: u32, row: u32) -> (i32, i32) {
    (
        grid_x + col as i32 * COL_STRIDE,
        grid_y + row as i32 * ROW_STRIDE,
    )
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ShotKind {
    Normal, // 2x4 px actual area (player and invader shots)
    Ufo,    // 3x7 px actual area
}

#[derive(Copy, Clone)]
struct Shot {
    x: i32,
    y: i32,
    anim: usize,
    active: bool,
    kind: ShotKind,
}

impl Shot {
    const NONE: Self = Self {
        x: 0,
        y: 0,
        anim: 0,
        active: false,
        kind: ShotKind::Normal,
    };

    /// Actual collision rectangle `(x, y, w, h)` within the 8x8 sprite region.
    const fn hitbox(self) -> (i32, i32, i32, i32) {
        match self.kind {
            ShotKind::Normal => (self.x + 3, self.y + 2, 2, 4),
            ShotKind::Ufo => (self.x + 2, self.y, 3, 7),
        }
    }

    /// Horizontal centre of the actual content (for base-tile half lookup).
    const fn hit_center_x(self) -> i32 {
        match self.kind {
            ShotKind::Normal => self.x + 4, // centre of 2 px at offset 3
            ShotKind::Ufo => self.x + 3,    // centre of 3 px at offset 2
        }
    }

    /// Top edge of content (leading edge for upward player shot).
    const fn hit_top(self) -> i32 {
        match self.kind {
            ShotKind::Normal => self.y + 2,
            ShotKind::Ufo => self.y,
        }
    }

    /// Bottom edge of content (leading edge for downward invader/UFO shot).
    const fn hit_bottom(self) -> i32 {
        match self.kind {
            ShotKind::Normal => self.y + 6, // 2 + 4
            ShotKind::Ufo => self.y + 7,    // 0 + 7
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum State {
    Playing,
    PlayerDead,
}

pub struct InvadersState {
    background: RegularBackground,
    rng: RandomNumberGenerator,

    alive_mask: [u64; 2],
    alive_count: u32,

    grid_x: i32,
    grid_y: i32,
    dir: i32,
    advance_timer: u16,
    pending_drop: bool,
    anim_frame: usize,

    player_x: i32,
    player_shot: Shot,

    inv_shots: [Shot; MAX_SHOTS],
    shot_timer: u16,
    move_shots_timer: u8,    // counts down from 3; fire when 0
    inv_shot_anim_timer: u8, // counts down from 8; step anim when 0
    ufo_move_timer: u8,      // counts down from 2; move UFO when 0

    ufo_active: bool,
    ufo_x: i32,
    ufo_frame: usize,
    ufo_frame_timer: u8, // counts down from 4; advance ufo_frame when 0
    ufo_timer: u16,
    ufo_dead_timer: u8,
    ufo_dead_x: i32,
    ufo_fire_x: i32, // x position at which the UFO fires this crossing
    ufo_fired: bool,

    dead_inv: u32,
    dead_timer: u8,
    dead_row: u32,
    dead_col: u32,

    ufo_sfx_timer: u8,

    lives: u8,
    score: u32,
    state: State,
    state_timer: u16,

    brick_background: RegularBackground,
    // [base][row][col][half]: 0=left half, 1=right half. Health 0..=3.
    base_health: [[[[u8; 2]; BASE_COLS]; BASE_ROWS]; NUM_BASES],
    // true = tile was last damaged from below (player shot) -> render with vflip.
    base_flip: [[[bool; BASE_COLS]; BASE_ROWS]; NUM_BASES],
    move_sound: bool,
}

impl InvadersState {
    pub fn new(seed: [u32; 4]) -> Self {
        let rng = RandomNumberGenerator::new_with_seed(seed);
        let background = background(&bg::bg_invaders, Priority::P1);
        let brick_background = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let base_health = [[[[3u8; 2]; BASE_COLS]; BASE_ROWS]; NUM_BASES];
        let base_flip = [[[false; BASE_COLS]; BASE_ROWS]; NUM_BASES];
        let mut state = Self {
            background,
            rng,
            brick_background,
            base_health,
            ufo_sfx_timer: 0,
            alive_mask: [u64::MAX; 2],
            alive_count: MAX_INVADERS,
            grid_x: GRID_INIT_X,
            grid_y: GRID_INIT_Y,
            dir: 1,
            advance_timer: advance_period(MAX_INVADERS),
            pending_drop: false,
            anim_frame: 0,
            player_x: PLAYER_INIT_X,
            player_shot: Shot::NONE,
            inv_shots: [Shot::NONE; MAX_SHOTS],
            shot_timer: 60,
            move_shots_timer: 3,
            inv_shot_anim_timer: 8,
            ufo_move_timer: 2,
            ufo_active: false,
            ufo_x: 0,
            ufo_frame: 0,
            ufo_frame_timer: 4,
            ufo_timer: UFO_INTERVAL,
            ufo_dead_timer: 0,
            ufo_dead_x: 0,
            ufo_fire_x: 0,
            ufo_fired: false,
            dead_inv: MAX_INVADERS,
            dead_timer: 0,
            dead_row: 0,
            dead_col: 0,
            lives: 3,
            score: 0,
            state: State::Playing,
            state_timer: 0,
            base_flip,
            move_sound: false,
        };
        state.paint_all_base_tiles();
        state
    }
}

impl InvadersState {
    #[inline]
    fn is_alive(&self, idx: u32) -> bool {
        if idx < 64 {
            (self.alive_mask[0] & (1 << idx)) != 0
        } else {
            (self.alive_mask[1] & (1 << (idx - 64))) != 0
        }
    }

    #[inline]
    fn set_alive(&mut self, idx: u32, alive: bool) {
        let (word, bit) = if idx < 64 {
            (&mut self.alive_mask[0], idx)
        } else {
            (&mut self.alive_mask[1], idx - 64)
        };
        if alive {
            *word |= 1 << bit;
        } else {
            *word &= !(1 << bit);
        }
    }

    pub fn cheat(&mut self) {
        self.lives = self.lives.saturating_add(1).min(8);
        set_achievement(Achievement::UsedCheatInvaders);
    }

    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        if self.state == State::PlayerDead {
            if self.state_timer > 0 {
                self.state_timer -= 1;
            } else if self.lives == 0 {
                return Some(SceneAction::Lose);
            } else {
                self.lives -= 1;
                self.reset_player();
                self.state = State::Playing;
            }
            return None;
        }

        if button_controller.is_pressed(Button::Left) {
            self.player_x = (self.player_x - PLAYER_SPEED).max(PLAYER_LEFT_LIMIT);
        }
        if button_controller.is_pressed(Button::Right) {
            self.player_x = (self.player_x + PLAYER_SPEED).min(PLAYER_RIGHT_LIMIT);
        }

        if button_controller.is_just_pressed(Button::A) && !self.player_shot.active {
            sound_controller.play_sfx(SoundEffect::InvaderPlayerFire);
            self.player_shot = Shot {
                x: self.player_x + PLAYER_W / 2 - SHOT_W / 2,
                y: PLAYER_Y - SHOT_H,
                anim: 0,
                active: true,
                kind: ShotKind::Normal,
            };
        }

        self.move_shots_timer = self.move_shots_timer.saturating_sub(1);
        let move_shots = self.move_shots_timer == 0;
        if move_shots {
            self.move_shots_timer = 3;
        }

        self.inv_shot_anim_timer = self.inv_shot_anim_timer.saturating_sub(1);
        let step_shot_anim = self.inv_shot_anim_timer == 0;
        if step_shot_anim {
            self.inv_shot_anim_timer = 8;
        }

        if self.player_shot.active {
            if move_shots {
                self.player_shot.y -= PLAYER_SHOT_SPEED;
            }
            if self.player_shot.y < PLAYER_SHOT_Y_LIMIT {
                self.player_shot.active = false;
            }
        }

        for shot in &mut self.inv_shots {
            if shot.active {
                if move_shots {
                    shot.y += INV_SHOT_SPEED;
                }
                if step_shot_anim {
                    shot.anim += 1;
                    let cap = match shot.kind {
                        ShotKind::Normal => 2,
                        ShotKind::Ufo => 3,
                    };
                    if shot.anim >= cap {
                        shot.anim = 0;
                    }
                }
                if shot.y > UFO_SHOT_Y_LIMIT {
                    shot.active = false;
                }
            }
        }

        if self.advance_timer > 0 {
            self.advance_timer -= 1;
        } else {
            self.do_advance();
            if self.move_sound {
                self.move_sound = false;
                sound_controller.play_sfx(SoundEffect::InvaderMove1);
            } else {
                self.move_sound = true;
                sound_controller.play_sfx(SoundEffect::InvaderMove2);
            }
        }

        self.shot_timer = self.shot_timer.saturating_sub(1);
        if self.shot_timer == 0 {
            self.try_spawn_invader_shot();
            let alive_u = self.alive_count.max(1) as u16;
            self.shot_timer = next_u16_in(&mut self.rng, 30, (45 + alive_u).min(120));
        }

        if self.ufo_active {
            self.ufo_sfx_timer = self.ufo_sfx_timer.saturating_sub(1);
            if self.ufo_sfx_timer == 0 {
                sound_controller.play_sfx(SoundEffect::InvaderUfo);
                self.ufo_sfx_timer = 10;
            }
            self.ufo_move_timer = self.ufo_move_timer.saturating_sub(1);
            if self.ufo_move_timer == 0 {
                self.ufo_move_timer = 2;
                self.ufo_x -= UFO_SPEED;
            }
            self.ufo_frame_timer = self.ufo_frame_timer.saturating_sub(1);
            if self.ufo_frame_timer == 0 {
                self.ufo_frame_timer = 4;
                self.ufo_frame += 1;
                if self.ufo_frame >= 4 {
                    self.ufo_frame = 0;
                }
            }
            if !self.ufo_fired && self.ufo_x <= self.ufo_fire_x {
                self.ufo_fired = true;
                self.try_spawn_invader_shot_at(self.ufo_x + UFO_W / 2, UFO_Y + UFO_H);
            }
            if self.ufo_x < -UFO_W {
                self.ufo_active = false;
                self.ufo_timer = UFO_INTERVAL;
            }
        } else if self.ufo_dead_timer > 0 {
            self.ufo_dead_timer = self.ufo_dead_timer.saturating_sub(1);
        } else {
            self.ufo_timer = self.ufo_timer.saturating_sub(1);
            if self.ufo_timer == 0 {
                self.ufo_active = true;
                self.ufo_x = RIGHT_WALL;
                self.ufo_frame = 0;
                self.ufo_frame_timer = 0;
                self.ufo_fired = false;
                self.ufo_fire_x =
                    next_u16_in(&mut self.rng, LEFT_WALL as u16, RIGHT_WALL as u16) as i32;
            }
        }

        if self.dead_timer > 0 {
            self.dead_timer -= 1;
            if self.dead_timer == 0 {
                self.dead_inv = MAX_INVADERS;
            }
        }

        //player shot vs invader shot
        if self.player_shot.active {
            let (mut phx, phy, mut phw, phh) = self.player_shot.hitbox();
            phx -= 1;
            phw += 1;
            for inv_shot in &mut self.inv_shots {
                if inv_shot.active {
                    let (ihx, ihy, ihw, ihh) = inv_shot.hitbox();
                    if overlaps(phx, phy, phw, phh, ihx, ihy, ihw, ihh) {
                        self.player_shot.active = false;
                        inv_shot.active = false;
                        break;
                    }
                }
            }
        }

        //player shot vs base
        if self.player_shot.active {
            let cx = self.player_shot.hit_center_x();
            let cy = self.player_shot.hit_top(); // leading edge moving upward
            if let Some((b, r, c, h)) = find_base_tile(cx, cy)
                && self.damage_base(b, r, c, h, true)
            {
                sound_controller.play_sfx(SoundEffect::InvaderCrumble);
                self.player_shot.active = false;
            }
        }

        //player shot vs invader
        if self.player_shot.active {
            let (phx, phy, phw, phh) = self.player_shot.hitbox();
            let col = (phx - self.grid_x) / COL_STRIDE;
            let row = (phy - self.grid_y) / ROW_STRIDE;
            if col >= 0 && col < COLS as i32 && row >= 0 && row < ROWS as i32 {
                let col = col as u32;
                let row = row as u32;
                let idx = row * COLS + col;
                if self.is_alive(idx) {
                    let (ix, iy) = inv_pixel_pos(self.grid_x, self.grid_y, col, row);
                    if overlaps(phx, phy, phw, phh, ix, iy, INV_W, INV_H) {
                        self.set_alive(idx, false);
                        self.alive_count = self.alive_count.saturating_sub(1);
                        self.score += type_score(row_type(row));
                        self.player_shot.active = false;
                        self.dead_inv = idx;
                        self.dead_row = row;
                        self.dead_col = col;
                        self.dead_timer = DEAD_FRAMES;
                        sound_controller.play_sfx(SoundEffect::InvaderDeath);
                        let new_period = advance_period(self.alive_count);
                        if new_period < self.advance_timer {
                            self.advance_timer = new_period;
                        }
                        if self.alive_count == 0 {
                            set_achievement(Achievement::BeatInvaders);
                            if self.lives == 3 {
                                set_achievement(Achievement::BeatInvadersWithFullLives);
                            }
                            return Some(SceneAction::Win);
                        }
                    }
                }
            }
        }

        // player shot vs ufo
        if self.player_shot.active && self.ufo_active {
            let (phx, phy, phw, phh) = self.player_shot.hitbox();
            if overlaps(phx, phy, phw, phh, self.ufo_x, UFO_Y, UFO_W, UFO_H) {
                sound_controller.play_sfx(SoundEffect::InvaderDeath);
                self.player_shot.active = false;
                self.ufo_active = false;
                self.score += UFO_SCORE;
                self.ufo_dead_x = self.ufo_x;
                self.ufo_dead_timer = DEAD_FRAMES;
                self.ufo_timer = UFO_INTERVAL;
            }
        }

        // invader shot vs base
        let mut base_hits = [None::<(usize, usize, usize, usize)>; MAX_SHOTS];
        for (i, shot) in self.inv_shots.iter().enumerate() {
            if shot.active {
                base_hits[i] = find_base_tile(shot.hit_center_x(), shot.hit_bottom());
            }
        }
        for (i, hit) in base_hits.iter().enumerate() {
            if let Some((b, r, c, h)) = *hit
                && self.damage_base(b, r, c, h, false)
            {
                sound_controller.play_sfx(SoundEffect::InvaderCrumble);
                self.inv_shots[i].active = false;
            }
        }

        // invader shot vs player
        for shot in &mut self.inv_shots {
            let (shx, shy, shw, shh) = shot.hitbox();
            if shot.active
                && overlaps(
                    shx,
                    shy,
                    shw,
                    shh,
                    self.player_x,
                    PLAYER_Y,
                    PLAYER_W,
                    PLAYER_H,
                )
            {
                sound_controller.play_sfx(SoundEffect::InvaderPlayerDeath);
                shot.active = false;
                self.state = State::PlayerDead;
                self.state_timer = PLAYER_DEAD_FRAMES;
                return None;
            }
        }

        'danger: for row in (0..ROWS).rev() {
            for col in 0..COLS {
                if self.is_alive(row * COLS + col) {
                    let iy = self.grid_y + row as i32 * ROW_STRIDE;
                    if iy + INV_H >= DANGER_Y {
                        return Some(SceneAction::Lose);
                    }
                    break 'danger;
                }
            }
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        self.background.show(frame);
        self.brick_background.show(frame);

        for row in 0..ROWS {
            for col in 0..COLS {
                let idx = row * COLS + col;
                if !self.is_alive(idx) {
                    continue;
                }
                let (x, y) = inv_pixel_pos(self.grid_x, self.grid_y, col, row);
                let spr = match row_type(row) {
                    0 => sprites::INVADER_1.sprite(self.anim_frame),
                    1 => sprites::INVADER_2.sprite(self.anim_frame),
                    _ => sprites::INVADER_3.sprite(self.anim_frame),
                };
                Object::new(spr).set_pos(vec2(x, y)).show(frame);
            }
        }

        if self.dead_timer > 0 && self.dead_inv < MAX_INVADERS {
            let (x, y) = inv_pixel_pos(self.grid_x, self.grid_y, self.dead_col, self.dead_row);
            let spr = match row_type(self.dead_row) {
                0 => sprites::INVADER_1_DEAD.sprite(0),
                1 => sprites::INVADER_2_DEAD.sprite(0),
                _ => sprites::INVADER_3_DEAD.sprite(0),
            };
            Object::new(spr).set_pos(vec2(x, y)).show(frame);
        }

        let tank_spr = if self.state == State::PlayerDead {
            sprites::INVADER_TANK_DEAD.sprite(0)
        } else {
            sprites::INVADERS_TANK.sprite(0)
        };
        Object::new(tank_spr)
            .set_pos(vec2(self.player_x, PLAYER_Y))
            .show(frame);

        if self.player_shot.active {
            Object::new(sprites::INVADER_SHOT_TANK.sprite(0))
                .set_pos(vec2(self.player_shot.x, self.player_shot.y))
                .show(frame);
        }

        for shot in &self.inv_shots {
            if shot.active {
                let spr = match shot.kind {
                    ShotKind::Normal => sprites::INVADER_SHOT_INVADER.sprite(shot.anim),
                    ShotKind::Ufo => sprites::INVADER_SHOT_UFO.sprite(shot.anim),
                };
                Object::new(spr).set_pos(vec2(shot.x, shot.y)).show(frame);
            }
        }

        if self.ufo_active {
            Object::new(sprites::INVADERS_UFO.sprite(self.ufo_frame))
                .set_pos(vec2(self.ufo_x, UFO_Y))
                .show(frame);
        } else if self.ufo_dead_timer > 0 {
            Object::new(sprites::INVADER_UFO_DEAD.sprite(0))
                .set_pos(vec2(self.ufo_dead_x, UFO_Y))
                .show(frame);
        }

        let start_x = WIDTH - (self.lives as i32 * LIFE_STRIDE);
        for i in 0..self.lives as i32 {
            Object::new(sprites::INVADER_LIFE.sprite(0))
                .set_pos(vec2(start_x + i * LIFE_STRIDE, LIFE_Y))
                .show(frame);
        }
    }
}

impl InvadersState {
    fn reset_player(&mut self) {
        self.player_x = PLAYER_INIT_X;
        self.player_shot = Shot::NONE;
        self.inv_shots = [Shot::NONE; MAX_SHOTS];
    }

    fn do_advance(&mut self) {
        if self.pending_drop {
            self.grid_y += DROP_AMOUNT;
            self.pending_drop = false;
            self.crush_bases_with_invaders();
            self.advance_timer = advance_period(self.alive_count);
            return;
        }

        self.grid_x += ADVANCE_STEP * self.dir;
        self.anim_frame ^= 1;

        let (left_col, right_col) = self.alive_x_extent();
        let right_edge = self.grid_x + right_col as i32 * COL_STRIDE + INV_W;
        let left_edge = self.grid_x + left_col as i32 * COL_STRIDE;

        if self.dir > 0 && right_edge >= RIGHT_WALL {
            self.dir = -1;
            self.pending_drop = true;
        } else if self.dir < 0 && left_edge <= LEFT_WALL {
            self.dir = 1;
            self.pending_drop = true;
        }

        self.crush_bases_with_invaders();
        self.advance_timer = advance_period(self.alive_count);
    }

    fn alive_x_extent(&self) -> (u32, u32) {
        if self.alive_count == 0 {
            return (0, 0);
        }
        let mut left = COLS - 1;
        let mut right = 0;
        for row in 0..ROWS {
            for col in 0..COLS {
                if self.is_alive(row * COLS + col) {
                    if col < left {
                        left = col;
                    }
                    if col > right {
                        right = col;
                    }
                }
            }
        }
        (left, right)
    }

    fn crush_bases_with_invaders(&mut self) {
        let base_y_min = (BASE_TILE_Y * 8) as i32;
        let base_y_max = ((BASE_TILE_Y + BASE_ROWS) * 8) as i32;

        for row in 0..ROWS {
            let iy = self.grid_y + row as i32 * ROW_STRIDE;
            if iy + INV_H <= base_y_min || iy >= base_y_max {
                continue;
            }
            for col in 0..COLS {
                if !self.is_alive(row * COLS + col) {
                    continue;
                }
                let ix = self.grid_x + col as i32 * COL_STRIDE;
                for (b, tile_x) in BASE_TILE_X.iter().enumerate().take(NUM_BASES) {
                    for bc in 0..BASE_COLS {
                        let bx = (tile_x + bc) as i32 * 8;
                        for br in 0..BASE_ROWS {
                            let by = (BASE_TILE_Y + br) as i32 * 8;
                            if overlaps(ix, iy, INV_W, INV_H, bx, by, 8, 8)
                                && (self.base_health[b][br][bc][0] > 0
                                    || self.base_health[b][br][bc][1] > 0)
                            {
                                self.base_health[b][br][bc][0] = 0;
                                self.base_health[b][br][bc][1] = 0;
                                self.update_base_tile(b, br, bc);
                            }
                        }
                    }
                }
            }
        }
    }

    fn paint_all_base_tiles(&mut self) {
        for b in 0..NUM_BASES {
            for r in 0..BASE_ROWS {
                for c in 0..BASE_COLS {
                    self.update_base_tile(b, r, c);
                }
            }
        }
    }

    /// damages one half-column of a base tile. Returns true if the tile had
    /// health remaining and should use up the shot
    fn damage_base(
        &mut self,
        base: usize,
        row: usize,
        col: usize,
        half: usize,
        from_below: bool,
    ) -> bool {
        let h = self.base_health[base][row][col][half];
        if h == 0 {
            return false;
        }
        self.base_health[base][row][col][half] = h - 1;
        self.base_flip[base][row][col] = from_below;
        self.update_base_tile(base, row, col);
        true
    }

    fn update_base_tile(&mut self, base: usize, row: usize, col: usize) {
        let left = self.base_health[base][row][col][0];
        let right = self.base_health[base][row][col][1];
        let lookup_idx = (left << 2) | right;
        let tile_idx = HEALTH_MAP[lookup_idx as usize];
        let vflip = self.base_flip[base][row][col];
        let tx = (BASE_TILE_X[base] + col) as i32;
        let ty = (BASE_TILE_Y + row) as i32;
        self.brick_background.set_tile(
            vec2(tx, ty),
            &BRICK_TILESET.tiles,
            BRICK_TILESET.tile_settings[tile_idx].vflip(vflip),
        );
    }

    fn try_spawn_invader_shot(&mut self) {
        let Some(slot) = self.inv_shots.iter().position(|s| !s.active) else {
            return;
        };

        let mut candidates: [(u32, u32); COLS as usize] = [(0, 0); COLS as usize];
        let mut count: u16 = 0;
        for col in 0..COLS {
            for row in (0..ROWS).rev() {
                if self.is_alive(row * COLS + col) {
                    candidates[count as usize] = (col, row);
                    count += 1;
                    break;
                }
            }
        }
        if count == 0 {
            return;
        }

        let (col, row) = candidates[next_u16_in(&mut self.rng, 0, count - 1) as usize];
        let x = self.grid_x + col as i32 * COL_STRIDE + INV_W / 2 - SHOT_W / 2;
        let y = self.grid_y + row as i32 * ROW_STRIDE + INV_H;
        self.inv_shots[slot] = Shot {
            x,
            y,
            anim: 0,
            active: true,
            kind: ShotKind::Normal,
        };
    }

    fn try_spawn_invader_shot_at(&mut self, x: i32, y: i32) {
        let Some(slot) = self.inv_shots.iter().position(|s| !s.active) else {
            return;
        };
        self.inv_shots[slot] = Shot {
            x: x - SHOT_W / 2,
            y,
            anim: 0,
            active: true,
            kind: ShotKind::Ufo,
        };
    }
}
