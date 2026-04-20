use crate::direction::Direction;
use crate::gfx::background_stack;
use crate::printer::WhiteVariWidthText;
use crate::rng::next_u16_in;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::object::{Object, Sprite, Tag};
use agb::display::tile_data::TileData;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat, TileSetting};
use agb::display::{GraphicsFrame, Layer, Priority};
use agb::fixnum::{num, vec2};
use agb::input::{Button, ButtonController};
use agb::rng::RandomNumberGenerator;
use alloc::format;
use resources::{bg, sprites};

// Pipe tile base indices (meta tile system: each pipe = 2x2 GBA tiles)
// Given base index `b`, the 4 tiles are: b, b+1, b+TILE_STRIDE, b+TILE_STRIDE+1
const TILE_STRIDE: usize = 36;

const PIPE_HORI_EMPTY: usize = 0;
const PIPE_HORI_FULL: usize = 2;
const PIPE_VERT_EMPTY: usize = 4;
const PIPE_VERT_FULL: usize = 6;
const PIPE_CORNER_BR_EMPTY: usize = 8; // connects bottom + right
const PIPE_CORNER_BR_FULL: usize = 10;
const PIPE_CROSS_EMPTY: usize = 12;
const PIPE_CROSS_VERT_FULL: usize = 14;
const PIPE_CROSS_HORI_FULL: usize = 18;
const PIPE_CROSS_BOTH_FULL: usize = 16;
const PIPE_START_LEFT_EMPTY: usize = 20; // left side on grid edge; water exits right
const PIPE_START_LEFT_FULL: usize = 22;
const PIPE_END_LEFT_EMPTY: usize = 24; // left side on grid edge; water enters right
const PIPE_END_LEFT_FULL: usize = 26;
const PIPE_START_TOP_EMPTY: usize = 28; // top side on grid edge; water exits bottom
const PIPE_START_TOP_FULL: usize = 30;
const PIPE_END_TOP_EMPTY: usize = 32; // top side on grid edge; water enters bottom
const PIPE_END_TOP_FULL: usize = 34;
const PIPE_CORNER_BL_EMPTY: usize = 72; // connects bottom + left
const PIPE_CORNER_BL_FULL: usize = 74;
const PIPE_CORNER_TR_EMPTY: usize = 76; // connects top + right
const PIPE_CORNER_TR_FULL: usize = 78;
const PIPE_CORNER_TL_EMPTY: usize = 80; // connects top + left
const PIPE_CORNER_TL_FULL: usize = 82;
const PIPE_START_RIGHT_EMPTY: usize = 98; // right side on grid edge; water exits left
const PIPE_START_RIGHT_FULL: usize = 96;
const PIPE_END_RIGHT_EMPTY: usize = 94; // right side on grid edge; water enters left
const PIPE_END_RIGHT_FULL: usize = 92;
const PIPE_START_BOTTOM_EMPTY: usize = 100; // bottom side on grid edge; water exits top
const PIPE_START_BOTTOM_FULL: usize = 102;
const PIPE_END_BOTTOM_EMPTY: usize = 104; // bottom side on grid edge; water enters top
const PIPE_END_BOTTOM_FULL: usize = 106;

//block that can't be connected with, starts on grid, can't be placed
const PIPE_INVALID: usize = 84;

// Fill animation tags
const FILL_HORI: &Tag = &sprites::PIPE_FILL_HORI; // 8 frames, left -> right
const FILL_VERT: &Tag = &sprites::PIPE_FILL_VERT; // 8 frames, top -> bottom
const FILL_VERT_CROSS: &Tag = &sprites::PIPE_FILL_VERT_CROSS; // 8 frames, top -> bottom for cross
const FILL_CORNER_1: &Tag = &sprites::PIPE_FILL_CORNER_1; // 10 frames: none=bottom -> right, h=bottom -> left, v=top -> right, vh=top -> left
const FILL_CORNER_2: &Tag = &sprites::PIPE_FILL_CORNER_2; // 10 frames: none=right -> bottom, h=left -> bottom, v=right -> top, vh=left -> top
const FILL_HORI_START_END: &Tag = &sprites::PIPE_FILL_SE_HORI; //7 frames: none=right->left, h=left->right
const FILL_VERT_START_END: &Tag = &sprites::PIPE_FILL_SE_VERT; //7 frames: none=top->bottom, h=bottom->top

const CURSOR: &Sprite = sprites::PIPE_CURSOR.sprite(0);

const MAX_W: usize = 12;
const MAX_H: usize = 8;
const QUEUE_LEN: usize = 7;
// Bag: 1 vert + 1 horz + 1 cross + 2 corners + 3 random
const BAG_SIZE: usize = 8;

const MOVE_TIMER_DURATION: u8 = 10;

//when a cross pipe is filled
const SCORE_DELTA_CROSS: i32 = 500;
//when the end is reached
const SCORE_DELTA_END: i32 = 1000;
//when a pipe is placed
const SCORE_DELTA_PLACE: i32 = 50;
//per empty pipe at end or when replacing pipe
const SCORE_DELTA_EMPTY: i32 = -100;
//per tick of flow
const SCORE_DELTA_FLOW: i32 = 10;

const SCORE_CROSS: &Sprite = sprites::PIPE_SCORE_LOOP.sprite(0);
const SCORE_END: &Sprite = sprites::PIPE_SCORE_END.sprite(0);
const SCORE_PLACE: &Sprite = sprites::PIPE_SCORE_PLACE.sprite(0);
const SCORE_EMPTY: &Sprite = sprites::PIPE_SCORE_UNUSED_PIPE.sprite(0);
const SCORE_FLOW: &Sprite = sprites::PIPE_SCORE_FLOW_TICK.sprite(0);

const SCORE_POPUP_DURATION: u8 = 16;
const MAX_SCORE_POPUPS: usize = 20;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PipeDifficulty {
    SmallEasy,
    SmallHard,
    LargeEasy,
    LargeHard,
}

impl PipeDifficulty {
    const fn bg(self) -> &'static TileData {
        match self {
            PipeDifficulty::SmallEasy | PipeDifficulty::SmallHard => &bg::bg_pipes_sml,
            PipeDifficulty::LargeEasy | PipeDifficulty::LargeHard => &bg::bg_pipes_lrg,
        }
    }

    const fn width(self) -> usize {
        match self {
            PipeDifficulty::SmallEasy | PipeDifficulty::SmallHard => 8,
            PipeDifficulty::LargeEasy | PipeDifficulty::LargeHard => 12,
        }
    }

    const fn height(self) -> usize {
        match self {
            PipeDifficulty::SmallEasy | PipeDifficulty::SmallHard => 6,
            PipeDifficulty::LargeEasy | PipeDifficulty::LargeHard => 8,
        }
    }

    const fn invalid_pipes_allowed(self) -> bool {
        matches!(self, PipeDifficulty::SmallHard | PipeDifficulty::LargeHard)
    }

    const fn frames_per_fill_step(self) -> u8 {
        match self {
            PipeDifficulty::SmallEasy | PipeDifficulty::LargeEasy => 26,
            PipeDifficulty::SmallHard | PipeDifficulty::LargeHard => 12,
        }
    }

    const fn fill_delay(self) -> u16 {
        (match self {
            PipeDifficulty::SmallEasy => 25,
            PipeDifficulty::LargeEasy => 30,
            PipeDifficulty::SmallHard => 10,
            PipeDifficulty::LargeHard => 15,
        }) * 60
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum PipeKind {
    Empty,
    Horizontal,
    Vertical,
    CornerBR, // connects bottom + right
    CornerBL, // connects bottom + left
    CornerTR, // connects top + right
    CornerTL, // connects top + left
    Cross,
    Invalid,
    // Start pipes: open side is on the grid edge, water flows inward
    StartLeft,   // left side on edge, enter from Left  ->  exit Right
    StartRight,  // right side on edge, enter from Right  ->  exit Left
    StartTop,    // top side on edge, enter from Up  ->  exit Down
    StartBottom, // bottom side on edge, enter from Down  ->  exit Up
    // End pipes: open side is on the grid edge, water flows outward = win
    EndLeft,   // left side on edge, enter from Right  ->  exit Left (off-grid)
    EndRight,  // right side on edge, enter from Left  ->  exit Right (off-grid)
    EndTop,    // top side on edge, enter from Down  ->  exit Up (off-grid)
    EndBottom, // bottom side on edge, enter from Up  ->  exit Down (off-grid)
}

impl PipeKind {
    fn empty_tile(self) -> Option<usize> {
        Some(match self {
            PipeKind::Horizontal => PIPE_HORI_EMPTY,
            PipeKind::Vertical => PIPE_VERT_EMPTY,
            PipeKind::CornerBR => PIPE_CORNER_BR_EMPTY,
            PipeKind::CornerBL => PIPE_CORNER_BL_EMPTY,
            PipeKind::CornerTR => PIPE_CORNER_TR_EMPTY,
            PipeKind::CornerTL => PIPE_CORNER_TL_EMPTY,
            PipeKind::Cross => PIPE_CROSS_EMPTY,
            PipeKind::Invalid => PIPE_INVALID,
            PipeKind::StartLeft => PIPE_START_LEFT_EMPTY,
            PipeKind::StartRight => PIPE_START_RIGHT_EMPTY,
            PipeKind::StartTop => PIPE_START_TOP_EMPTY,
            PipeKind::StartBottom => PIPE_START_BOTTOM_EMPTY,
            PipeKind::EndLeft => PIPE_END_LEFT_EMPTY,
            PipeKind::EndRight => PIPE_END_RIGHT_EMPTY,
            PipeKind::EndTop => PIPE_END_TOP_EMPTY,
            PipeKind::EndBottom => PIPE_END_BOTTOM_EMPTY,
            PipeKind::Empty => return None,
        })
    }

    fn full_tile(self) -> Option<usize> {
        Some(match self {
            PipeKind::Horizontal => PIPE_HORI_FULL,
            PipeKind::Vertical => PIPE_VERT_FULL,
            PipeKind::CornerBR => PIPE_CORNER_BR_FULL,
            PipeKind::CornerBL => PIPE_CORNER_BL_FULL,
            PipeKind::CornerTR => PIPE_CORNER_TR_FULL,
            PipeKind::CornerTL => PIPE_CORNER_TL_FULL,
            PipeKind::Cross => PIPE_CROSS_BOTH_FULL,
            PipeKind::StartLeft => PIPE_START_LEFT_FULL,
            PipeKind::StartRight => PIPE_START_RIGHT_FULL,
            PipeKind::StartTop => PIPE_START_TOP_FULL,
            PipeKind::StartBottom => PIPE_START_BOTTOM_FULL,
            PipeKind::EndLeft => PIPE_END_LEFT_FULL,
            PipeKind::EndRight => PIPE_END_RIGHT_FULL,
            PipeKind::EndTop => PIPE_END_TOP_FULL,
            PipeKind::EndBottom => PIPE_END_BOTTOM_FULL,
            PipeKind::Invalid | PipeKind::Empty => return None,
        })
    }

    fn cross_tile(self, horiz_full: bool, vert_full: bool) -> usize {
        match (horiz_full, vert_full) {
            (false, false) => PIPE_CROSS_EMPTY,
            (true, false) => PIPE_CROSS_HORI_FULL,
            (false, true) => PIPE_CROSS_VERT_FULL,
            (true, true) => PIPE_CROSS_BOTH_FULL,
        }
    }

    /// Returns the exit direction when water enters from `from`, or None if incompatible.
    fn pipe_exit(self, from: Direction) -> Option<Direction> {
        use Direction::*;
        match (self, from) {
            (PipeKind::Horizontal | PipeKind::StartLeft | PipeKind::EndRight, Left) => Some(Right),
            (PipeKind::Horizontal | PipeKind::StartRight | PipeKind::EndLeft, Right) => Some(Left),
            (PipeKind::Vertical | PipeKind::StartTop | PipeKind::EndBottom, Up) => Some(Down),
            (PipeKind::Vertical | PipeKind::StartBottom | PipeKind::EndTop, Down) => Some(Up),
            (PipeKind::CornerBR, Down) => Some(Right),
            (PipeKind::CornerBR, Right) => Some(Down),
            (PipeKind::CornerBL, Down) => Some(Left),
            (PipeKind::CornerBL, Left) => Some(Down),
            (PipeKind::CornerTR, Up) => Some(Right),
            (PipeKind::CornerTR, Right) => Some(Up),
            (PipeKind::CornerTL, Up) => Some(Left),
            (PipeKind::CornerTL, Left) => Some(Up),
            (PipeKind::Cross, _) => Some(from.opposite()),
            _ => None,
        }
    }

    fn is_end(self) -> bool {
        matches!(
            self,
            PipeKind::EndLeft | PipeKind::EndRight | PipeKind::EndTop | PipeKind::EndBottom
        )
    }

    fn is_start(self) -> bool {
        matches!(
            self,
            PipeKind::StartLeft | PipeKind::StartRight | PipeKind::StartTop | PipeKind::StartBottom
        )
    }

    fn start_enter_dir(self) -> Direction {
        match self {
            PipeKind::StartLeft => Direction::Left,
            PipeKind::StartRight => Direction::Right,
            PipeKind::StartTop => Direction::Up,
            PipeKind::StartBottom => Direction::Down,
            _ => Direction::Left,
        }
    }

    fn anim_max_frames(self) -> u8 {
        match self {
            PipeKind::CornerBR | PipeKind::CornerBL | PipeKind::CornerTR | PipeKind::CornerTL => 10,
            PipeKind::EndRight
            | PipeKind::EndLeft
            | PipeKind::EndTop
            | PipeKind::EndBottom
            | PipeKind::StartRight
            | PipeKind::StartLeft
            | PipeKind::StartTop
            | PipeKind::StartBottom => 7,
            _ => 8,
        }
    }
}

#[derive(Copy, Clone)]
struct Cell {
    kind: PipeKind,
    horiz_full: bool,
    vert_full: bool,
}

impl Cell {
    const fn empty() -> Self {
        Self {
            kind: PipeKind::Empty,
            horiz_full: false,
            vert_full: false,
        }
    }

    fn display_tile(self) -> Option<usize> {
        match self.kind {
            PipeKind::Empty => None,
            PipeKind::Cross => Some(self.kind.cross_tile(self.horiz_full, self.vert_full)),
            _ => {
                if self.horiz_full || self.vert_full {
                    self.kind.full_tile()
                } else {
                    self.kind.empty_tile()
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct ScorePopup {
    sprite: &'static Sprite,
    x: i16,
    y: i16,
    frames_left: u8,
}

#[derive(Debug)]
enum FlowPhase {
    WaitingForFirstPipe(u16), //countdown to use
    Countdown(u16),           //remaining frames
    Flowing {
        x: u8,
        y: u8,
        enter_from: Direction,
        anim_frame: u8,
        anim_timer: u8,
    },
    WinCounting {
        empty_scan_pos: u16,
    },
}

pub struct PipesState {
    backgrounds: [RegularBackground; 2],
    tile_layer: RegularBackground,
    grid: [Cell; MAX_W * MAX_H],
    grid_w: u8,
    grid_h: u8,
    grid_tile_x: i32,
    grid_tile_y: i32,
    cursor_x: u8,
    cursor_y: u8,
    queue: [PipeKind; QUEUE_LEN],
    bag: [PipeKind; BAG_SIZE],
    bag_pos: u8,
    difficulty: PipeDifficulty,
    rng: RandomNumberGenerator,
    flow: FlowPhase,
    score: i32,
    score_popups: [Option<ScorePopup>; MAX_SCORE_POPUPS],
    move_input_timer: u8,
    place_tile_layer: RegularBackground,
}

impl PipesState {
    pub fn new(seed: [u32; 4], difficulty: PipeDifficulty) -> Self {
        let mut rng = RandomNumberGenerator::new_with_seed(seed);
        let w = difficulty.width() as u8;
        let h = difficulty.height() as u8;

        let grid_tiles_w = w as i32 * 2;
        let grid_tile_x = 2 + (28 - grid_tiles_w) / 2; // 28 = 30 total - 2 queue cols
        let grid_tile_y = (20 - h as i32 * 2) / 2; // always 8 rows tall

        let backgrounds = background_stack([&bg::bg_pipes, difficulty.bg()]);
        let tile_layer = RegularBackground::new(
            Priority::P1,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let place_tile_layer = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        let mut grid = [Cell::empty(); MAX_W * MAX_H];

        let start_pos = place_edge_pipe(&mut rng, w, h, &mut grid, true, None);
        let end_pos = place_edge_pipe(&mut rng, w, h, &mut grid, false, Some(start_pos));

        if difficulty.invalid_pipes_allowed() {
            let count = (w as u16 * h as u16 / 8).max(1);
            for _ in 0..count {
                let idx = next_u16_in(&mut rng, 0, w as u16 * h as u16 - 1) as usize;
                let x = idx % w as usize;
                let y = idx / w as usize;
                if grid[idx].kind == PipeKind::Empty && not_neighbours((x, y), start_pos, end_pos) {
                    grid[idx].kind = PipeKind::Invalid;
                }
            }
        }

        let mut bag = [PipeKind::Horizontal; BAG_SIZE];
        fill_bag(&mut rng, &mut bag);
        let mut bag_pos = 0u8;

        let mut queue = [PipeKind::Horizontal; QUEUE_LEN];
        for q in &mut queue {
            if bag_pos as usize >= BAG_SIZE {
                fill_bag(&mut rng, &mut bag);
                bag_pos = 0;
            }
            *q = bag[bag_pos as usize];
            bag_pos += 1;
        }

        let countdown = difficulty.fill_delay();

        let mut state = Self {
            place_tile_layer,
            backgrounds,
            tile_layer,
            grid,
            grid_w: w,
            grid_h: h,
            grid_tile_x,
            grid_tile_y,
            cursor_x: w / 2,
            cursor_y: h / 2,
            queue,
            bag,
            bag_pos,
            difficulty,
            rng,
            score: 0,
            score_popups: [None; MAX_SCORE_POPUPS],
            flow: FlowPhase::WaitingForFirstPipe(countdown),
            move_input_timer: 0,
        };
        state.init_tiles();
        state
    }

    fn draw_from_bag(&mut self) -> PipeKind {
        if self.bag_pos as usize >= BAG_SIZE {
            fill_bag(&mut self.rng, &mut self.bag);
            self.bag_pos = 0;
        }
        let piece = self.bag[self.bag_pos as usize];
        self.bag_pos += 1;
        piece
    }

    fn tile_coords(&self, x: usize, y: usize) -> (i32, i32) {
        (
            self.grid_tile_x + x as i32 * 2,
            self.grid_tile_y + y as i32 * 2,
        )
    }

    fn pixel_coords(&self, x: usize, y: usize) -> (i32, i32) {
        (
            self.grid_tile_x * 8 + x as i32 * 16,
            self.grid_tile_y * 8 + y as i32 * 16,
        )
    }

    fn set_cell_tile(&mut self, x: usize, y: usize) {
        let cell = self.grid[y * self.grid_w as usize + x];
        if let Some(tile_idx) = cell.display_tile() {
            let (tx, ty) = self.tile_coords(x, y);
            set_meta_tile(&mut self.tile_layer, tx, ty, tile_idx);
        }
    }

    fn init_tiles(&mut self) {
        let w = self.grid_w as usize;
        let h = self.grid_h as usize;
        for y in 0..h {
            for x in 0..w {
                self.set_cell_tile(x, y);
            }
        }
        self.redraw_queue();
        self.set_pipe_hint();
    }

    fn redraw_queue(&mut self) {
        for i in 0..QUEUE_LEN {
            if let Some(tile_idx) = self.queue[i].empty_tile() {
                let mut y = 3 + i as i32 * 2;
                if i > 0 {
                    y += 1;
                }
                set_meta_tile(&mut self.tile_layer, 1, y, tile_idx);
            }
        }
    }

    fn apply_score(&mut self, delta: i32) {
        self.score += delta;
    }

    fn spawn_score_popup(&mut self, sprite: &'static Sprite, x: usize, y: usize) {
        let (px, py) = self.pixel_coords(x, y);
        let popup = ScorePopup {
            sprite,
            x: px as i16,
            y: py as i16,
            frames_left: SCORE_POPUP_DURATION,
        };
        for slot in &mut self.score_popups {
            if slot.is_none() {
                *slot = Some(popup);
                return;
            }
        }
        if let Some(oldest) = self
            .score_popups
            .iter_mut()
            .min_by_key(|s| s.map_or(u8::MAX, |p| p.frames_left))
        {
            *oldest = Some(popup);
        }
    }

    fn clear_pipe_hint(&mut self) {
        let (tx, ty) = self.tile_coords(self.cursor_x as usize, self.cursor_y as usize);
        self.place_tile_layer
            .set_tile(vec2(tx, ty), &bg::bg_pipe_parts.tiles, TileSetting::BLANK);
        self.place_tile_layer.set_tile(
            vec2(tx + 1, ty),
            &bg::bg_pipe_parts.tiles,
            TileSetting::BLANK,
        );
        self.place_tile_layer.set_tile(
            vec2(tx, ty + 1),
            &bg::bg_pipe_parts.tiles,
            TileSetting::BLANK,
        );
        self.place_tile_layer.set_tile(
            vec2(tx + 1, ty + 1),
            &bg::bg_pipe_parts.tiles,
            TileSetting::BLANK,
        );
        let cx = self.cursor_x as usize;
        let cy = self.cursor_y as usize;
        self.set_cell_tile(cx, cy);
    }

    fn is_cursor_cell_flowing(&self) -> bool {
        matches!(self.flow, FlowPhase::Flowing { x, y, .. } if x == self.cursor_x && y == self.cursor_y)
    }

    fn set_pipe_hint(&mut self) {
        let idx = self.cursor_y as usize * self.grid_w as usize + self.cursor_x as usize;
        let cell = self.grid[idx];
        let is_empty = cell.kind == PipeKind::Empty;
        let can_replace = matches!(
            cell.kind,
            PipeKind::Horizontal
                | PipeKind::Vertical
                | PipeKind::CornerBR
                | PipeKind::CornerBL
                | PipeKind::CornerTR
                | PipeKind::CornerTL
                | PipeKind::Cross
        ) && !cell.horiz_full
            && !cell.vert_full
            && !self.is_cursor_cell_flowing();
        if (is_empty || can_replace)
            && let Some(tile_idx) = self.queue[0].empty_tile()
        {
            let (tx, ty) = self.tile_coords(self.cursor_x as usize, self.cursor_y as usize);
            set_meta_tile(&mut self.place_tile_layer, tx, ty, tile_idx);
            if can_replace {
                self.tile_layer.set_tile(
                    vec2(tx, ty),
                    &bg::bg_pipe_parts.tiles,
                    TileSetting::BLANK,
                );
                self.tile_layer.set_tile(
                    vec2(tx + 1, ty),
                    &bg::bg_pipe_parts.tiles,
                    TileSetting::BLANK,
                );
                self.tile_layer.set_tile(
                    vec2(tx, ty + 1),
                    &bg::bg_pipe_parts.tiles,
                    TileSetting::BLANK,
                );
                self.tile_layer.set_tile(
                    vec2(tx + 1, ty + 1),
                    &bg::bg_pipe_parts.tiles,
                    TileSetting::BLANK,
                );
            }
        }
    }

    fn place_pipe(&mut self, sound_controller: &mut SoundController) -> bool {
        let idx = self.cursor_y as usize * self.grid_w as usize + self.cursor_x as usize;
        let cell = self.grid[idx];
        let is_empty = cell.kind == PipeKind::Empty;
        let can_replace = matches!(
            cell.kind,
            PipeKind::Horizontal
                | PipeKind::Vertical
                | PipeKind::CornerBR
                | PipeKind::CornerBL
                | PipeKind::CornerTR
                | PipeKind::CornerTL
                | PipeKind::Cross
        ) && !cell.horiz_full
            && !cell.vert_full
            && !self.is_cursor_cell_flowing();
        if !is_empty && !can_replace {
            return false;
        }
        let kind = self.queue[0];
        for i in 0..QUEUE_LEN - 1 {
            self.queue[i] = self.queue[i + 1];
        }
        self.queue[QUEUE_LEN - 1] = self.draw_from_bag();
        self.grid[idx].kind = kind;
        let cx = self.cursor_x as usize;
        let cy = self.cursor_y as usize;
        self.set_cell_tile(cx, cy);
        self.redraw_queue();
        self.set_pipe_hint();
        if is_empty {
            sound_controller.play_sfx(SoundEffect::Place);
            self.apply_score(SCORE_DELTA_PLACE);
            self.spawn_score_popup(SCORE_PLACE, cx, cy);
        } else {
            sound_controller.play_sfx(SoundEffect::InvaderCrumble);
            self.apply_score(SCORE_DELTA_EMPTY);
            self.spawn_score_popup(SCORE_EMPTY, cx, cy);
        }
        true
    }

    fn start_flow(&mut self) -> Option<SceneAction> {
        let w = self.grid_w as usize;
        let h = self.grid_h as usize;
        for y in 0..h {
            for x in 0..w {
                let kind = self.grid[y * w + x].kind;
                if kind.is_start() {
                    self.flow = FlowPhase::Flowing {
                        x: x as u8,
                        y: y as u8,
                        enter_from: kind.start_enter_dir(),
                        anim_frame: 0,
                        anim_timer: 0,
                    };
                    return None;
                }
            }
        }
        self.set_as_loss()
    }

    fn set_as_loss(&mut self) -> Option<SceneAction> {
        self.clear_pipe_hint();
        Some(SceneAction::Lose)
    }

    fn advance_flow(
        &mut self,
        x: u8,
        y: u8,
        enter_from: Direction,
        anim_frame: u8,
        anim_timer: u8,
    ) -> Option<SceneAction> {
        let new_timer = anim_timer + 1;
        if new_timer < self.difficulty.frames_per_fill_step() {
            if let FlowPhase::Flowing {
                anim_timer: ref mut t,
                ..
            } = self.flow
            {
                *t = new_timer;
            }
            return None;
        }

        let kind = self.grid[y as usize * self.grid_w as usize + x as usize].kind;
        let new_frame = anim_frame + 1;

        self.apply_score(SCORE_DELTA_FLOW);
        self.spawn_score_popup(SCORE_FLOW, x as usize, y as usize);

        if new_frame < kind.anim_max_frames() {
            self.flow = FlowPhase::Flowing {
                x,
                y,
                enter_from,
                anim_frame: new_frame,
                anim_timer: 0,
            };
            return None;
        }

        let idx = y as usize * self.grid_w as usize + x as usize;
        let is_horiz = matches!(enter_from, Direction::Left | Direction::Right);
        if is_horiz {
            self.grid[idx].horiz_full = true;
        } else {
            self.grid[idx].vert_full = true;
        }
        self.set_cell_tile(x as usize, y as usize);
        if kind == PipeKind::Cross && self.grid[idx].horiz_full && self.grid[idx].vert_full {
            self.apply_score(SCORE_DELTA_CROSS);
            self.spawn_score_popup(SCORE_CROSS, x as usize, y as usize);
        }

        let exit_dir = match kind.pipe_exit(enter_from) {
            Some(d) => d,
            None => {
                return self.set_as_loss();
            }
        };

        let (nx, ny) = dir_step(x as i32, y as i32, exit_dir);

        if nx < 0 || ny < 0 || nx >= self.grid_w as i32 || ny >= self.grid_h as i32 {
            if kind.is_end() {
                self.apply_score(SCORE_DELTA_END);
                self.spawn_score_popup(SCORE_END, x as usize, y as usize);
                self.clear_pipe_hint();
                self.flow = FlowPhase::WinCounting { empty_scan_pos: 0 };
            } else {
                return self.set_as_loss();
            }
            return None;
        }

        let next_idx = ny as usize * self.grid_w as usize + nx as usize;
        let next_cell = self.grid[next_idx];
        let next_enter = exit_dir.opposite();
        let next_is_horiz = matches!(next_enter, Direction::Left | Direction::Right);

        let can_enter = next_cell.kind.pipe_exit(next_enter).is_some();
        let already_filled = if next_cell.kind == PipeKind::Cross {
            (next_is_horiz && next_cell.horiz_full) || (!next_is_horiz && next_cell.vert_full)
        } else {
            next_cell.horiz_full || next_cell.vert_full
        };

        if !can_enter || already_filled {
            return self.set_as_loss();
        }

        self.flow = FlowPhase::Flowing {
            x: nx as u8,
            y: ny as u8,
            enter_from: next_enter,
            anim_frame: 0,
            anim_timer: 0,
        };

        None
    }
}

fn not_neighbours(center: (usize, usize), start: (usize, usize), end: (usize, usize)) -> bool {
    let dx_s = center.0.wrapping_sub(start.0).wrapping_add(1);
    let dy_s = center.1.wrapping_sub(start.1).wrapping_add(1);
    if dx_s <= 2 && dy_s <= 2 {
        return false;
    }

    let dx_e = center.0.wrapping_sub(end.0).wrapping_add(1);
    let dy_e = center.1.wrapping_sub(end.1).wrapping_add(1);
    if dx_e <= 2 && dy_e <= 2 {
        return false;
    }

    true
}

impl PipesState {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        for slot in &mut self.score_popups {
            if let Some(popup) = slot {
                popup.y -= 1;
                if popup.frames_left == 0 {
                    *slot = None;
                } else {
                    popup.frames_left -= 1;
                }
            }
        }

        if !matches!(self.flow, FlowPhase::WinCounting { .. }) {
            let w = self.grid_w;
            let h = self.grid_h;
            if self.move_input_timer > 0 {
                self.move_input_timer -= 1;
            } else {
                if button_controller.is_pressed(Button::Up) && self.cursor_y > 0 {
                    self.clear_pipe_hint();
                    self.cursor_y -= 1;
                    sound_controller.play_sfx(SoundEffect::SweeperCursor);
                    self.move_input_timer = MOVE_TIMER_DURATION;
                    self.set_pipe_hint();
                }
                if button_controller.is_pressed(Button::Down) && self.cursor_y + 1 < h {
                    self.clear_pipe_hint();
                    self.cursor_y += 1;
                    sound_controller.play_sfx(SoundEffect::SweeperCursor);
                    self.move_input_timer = MOVE_TIMER_DURATION;
                    self.set_pipe_hint();
                }
                if button_controller.is_pressed(Button::Left) && self.cursor_x > 0 {
                    self.clear_pipe_hint();
                    self.cursor_x -= 1;
                    sound_controller.play_sfx(SoundEffect::SweeperCursor);
                    self.move_input_timer = MOVE_TIMER_DURATION;
                    self.set_pipe_hint();
                }
                if button_controller.is_pressed(Button::Right) && self.cursor_x + 1 < w {
                    self.clear_pipe_hint();
                    self.cursor_x += 1;
                    sound_controller.play_sfx(SoundEffect::SweeperCursor);
                    self.move_input_timer = MOVE_TIMER_DURATION;
                    self.set_pipe_hint();
                }
            }
            if button_controller.is_just_pressed(Button::A) {
                if self.place_pipe(sound_controller) {
                    if let FlowPhase::WaitingForFirstPipe(countdown) = self.flow {
                        self.flow = FlowPhase::Countdown(countdown);
                    }
                } else {
                    sound_controller.play_sfx(SoundEffect::InvalidCursor);
                }
            }
        }

        match &self.flow {
            FlowPhase::WaitingForFirstPipe(_) => {}
            FlowPhase::WinCounting { empty_scan_pos } => {
                let grid_w = self.grid_w as usize;
                let grid_size = grid_w * self.grid_h as usize;
                let start = *empty_scan_pos as usize;
                if start < grid_size {
                    if let Some(i) = (start..grid_size).find(|&i| {
                        let c = self.grid[i];
                        c.kind != PipeKind::Empty && !c.horiz_full && !c.vert_full
                    }) {
                        self.apply_score(SCORE_DELTA_EMPTY);
                        self.spawn_score_popup(SCORE_EMPTY, i % grid_w, i / grid_w);
                        self.flow = FlowPhase::WinCounting {
                            empty_scan_pos: (i + 1) as u16,
                        };
                    } else {
                        self.flow = FlowPhase::WinCounting {
                            empty_scan_pos: grid_size as u16,
                        };
                    }
                } else {
                    if self.score > 0 {
                        self.clear_pipe_hint();
                        return Some(SceneAction::Win);
                    } else {
                        return self.set_as_loss();
                    }
                }
            }
            FlowPhase::Countdown(frames) => {
                if *frames == 0 {
                    sound_controller.play_sfx(SoundEffect::Water);
                    if let Some(result) = self.start_flow() {
                        return Some(result);
                    }
                } else {
                    self.flow = FlowPhase::Countdown(frames - 1);
                }
            }
            FlowPhase::Flowing {
                x,
                y,
                enter_from,
                anim_frame,
                anim_timer,
            } => {
                if let Some(result) =
                    self.advance_flow(*x, *y, *enter_from, *anim_frame, *anim_timer)
                {
                    return Some(result);
                }
                if button_controller.is_pressed(Button::R)
                    && let FlowPhase::Flowing {
                        x,
                        y,
                        enter_from,
                        anim_frame,
                        anim_timer,
                    } = self.flow
                {
                    for _ in 0..3 {
                        if let Some(result) =
                            self.advance_flow(x, y, enter_from, anim_frame, anim_timer)
                        {
                            return Some(result);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame, is_running: bool) {
        self.backgrounds[0].show(frame);
        self.backgrounds[1].show(frame);
        let tile_id = self.tile_layer.show(frame);
        let place_id = self.place_tile_layer.show(frame);

        frame
            .blend()
            .alpha(num!(0.5), num!(0.5))
            .enable_background(Layer::Top, place_id)
            .enable_background(Layer::Bottom, tile_id);

        if is_running && !matches!(self.flow, FlowPhase::WinCounting { .. }) {
            let (px, py) = self.pixel_coords(self.cursor_x as usize, self.cursor_y as usize);
            Object::new(CURSOR).set_pos(vec2(px, py)).show(frame);
        }

        WhiteVariWidthText::new(&format!("Score: {: >5}", self.score), 0).show(vec2(180, 1), frame);

        if let FlowPhase::Flowing {
            x,
            y,
            enter_from,
            anim_frame,
            ..
        } = self.flow
        {
            let kind = self.grid[y as usize * self.grid_w as usize + x as usize].kind;
            let (px, py) = self.pixel_coords(x as usize, y as usize);
            let (tag, flip_h, flip_v) = fill_anim_for(kind, enter_from);
            Object::new(tag.sprite(anim_frame as usize))
                .set_hflip(flip_h)
                .set_vflip(flip_v)
                .set_pos(vec2(px, py))
                .show(frame);
        }

        for popup in self.score_popups.iter().flatten() {
            Object::new(popup.sprite)
                .set_pos(vec2(popup.x as i32, popup.y as i32))
                .show(frame);
        }
    }
}

fn place_edge_pipe(
    rng: &mut RandomNumberGenerator,
    w: u8,
    h: u8,
    grid: &mut [Cell],
    is_start: bool,
    min_dist_from: Option<(usize, usize)>,
) -> (usize, usize) {
    loop {
        let (x, y, kind) = match next_u16_in(rng, 0, 3) {
            0 => {
                let y = next_u16_in(rng, 0, h as u16 - 1) as usize;
                let kind = if is_start {
                    PipeKind::StartLeft
                } else {
                    PipeKind::EndLeft
                };
                (0, y, kind)
            }
            1 => {
                let y = next_u16_in(rng, 0, h as u16 - 1) as usize;
                let kind = if is_start {
                    PipeKind::StartRight
                } else {
                    PipeKind::EndRight
                };
                (w as usize - 1, y, kind)
            }
            2 => {
                let x = next_u16_in(rng, 0, w as u16 - 1) as usize;
                let kind = if is_start {
                    PipeKind::StartTop
                } else {
                    PipeKind::EndTop
                };
                (x, 0, kind)
            }
            _ => {
                let x = next_u16_in(rng, 0, w as u16 - 1) as usize;
                let kind = if is_start {
                    PipeKind::StartBottom
                } else {
                    PipeKind::EndBottom
                };
                (x, h as usize - 1, kind)
            }
        };
        if grid[y * w as usize + x].kind != PipeKind::Empty {
            continue;
        }
        if let Some((ox, oy)) = min_dist_from
            && ox.abs_diff(x) + oy.abs_diff(y) < 5
        {
            continue;
        }
        grid[y * w as usize + x] = Cell {
            kind,
            horiz_full: false,
            vert_full: false,
        };
        return (x, y);
    }
}

fn set_meta_tile(tile_layer: &mut RegularBackground, tx: i32, ty: i32, base_idx: usize) {
    tile_layer.set_tile(
        vec2(tx, ty),
        &bg::bg_pipe_parts.tiles,
        bg::bg_pipe_parts.tile_settings[base_idx],
    );
    tile_layer.set_tile(
        vec2(tx + 1, ty),
        &bg::bg_pipe_parts.tiles,
        bg::bg_pipe_parts.tile_settings[base_idx + 1],
    );
    tile_layer.set_tile(
        vec2(tx, ty + 1),
        &bg::bg_pipe_parts.tiles,
        bg::bg_pipe_parts.tile_settings[base_idx + TILE_STRIDE],
    );
    tile_layer.set_tile(
        vec2(tx + 1, ty + 1),
        &bg::bg_pipe_parts.tiles,
        bg::bg_pipe_parts.tile_settings[base_idx + TILE_STRIDE + 1],
    );
}

fn dir_step(x: i32, y: i32, dir: Direction) -> (i32, i32) {
    match dir {
        Direction::Left => (x - 1, y),
        Direction::Right => (x + 1, y),
        Direction::Up => (x, y - 1),
        Direction::Down => (x, y + 1),
    }
}

fn random_corner(rng: &mut RandomNumberGenerator) -> PipeKind {
    match next_u16_in(rng, 0, 3) {
        0 => PipeKind::CornerBR,
        1 => PipeKind::CornerBL,
        2 => PipeKind::CornerTR,
        _ => PipeKind::CornerTL,
    }
}

fn random_corner_not(rng: &mut RandomNumberGenerator, exclude: PipeKind) -> PipeKind {
    let all = [
        PipeKind::CornerBR,
        PipeKind::CornerBL,
        PipeKind::CornerTR,
        PipeKind::CornerTL,
    ];
    let mut available = [PipeKind::CornerBR; 3];
    let mut count = 0;
    for &c in &all {
        if c != exclude {
            available[count] = c;
            count += 1;
        }
    }
    available[next_u16_in(rng, 0, 2) as usize]
}

fn random_placeable_not(
    rng: &mut RandomNumberGenerator,
    excl1: PipeKind,
    excl2: PipeKind,
) -> PipeKind {
    let all = [
        PipeKind::Horizontal,
        PipeKind::Vertical,
        PipeKind::CornerBR,
        PipeKind::CornerBL,
        PipeKind::CornerTR,
        PipeKind::CornerTL,
        PipeKind::Cross,
    ];
    let mut available = [PipeKind::Horizontal; 5];
    let mut count = 0;
    for &p in &all {
        if p != excl1 && p != excl2 {
            available[count] = p;
            count += 1;
        }
    }
    available[next_u16_in(rng, 0, 4) as usize]
}

/// Fill a bag with 1 vert, 1 horz, 1 cross, 2 distinct random corners, and 3 random pieces
/// (none matching either corner type), then shuffle with Fisher-Yates.
fn fill_bag(rng: &mut RandomNumberGenerator, bag: &mut [PipeKind; BAG_SIZE]) {
    bag[0] = PipeKind::Vertical;
    bag[1] = PipeKind::Horizontal;
    bag[2] = PipeKind::Cross;
    bag[3] = random_corner(rng);
    bag[4] = random_corner_not(rng, bag[3]);
    bag[5] = random_placeable_not(rng, bag[3], bag[4]);
    bag[6] = random_placeable_not(rng, bag[3], bag[4]);
    bag[7] = random_placeable_not(rng, bag[3], bag[4]);
    // Fisher-Yates shuffle
    for i in (1..BAG_SIZE).rev() {
        let j = next_u16_in(rng, 0, i as u16) as usize;
        bag.swap(i, j);
    }
}

/// Returns (tag, flip_h, flip_v) for the fill animation of a pipe cell.
///
/// FILL_CORNER_1 variants (water enters from bottom or top):
///   none = bottom -> right, h = bottom -> left, v = top -> right, vh = top -> left
/// FILL_CORNER_2 variants (water enters from right or left):
///   none = right -> bottom, h = left -> bottom, v = right -> top, vh = left -> top
fn fill_anim_for(kind: PipeKind, enter_from: Direction) -> (&'static Tag, bool, bool) {
    use Direction::*;
    match (kind, enter_from) {
        (PipeKind::StartLeft, Left) => (FILL_HORI_START_END, true, false),
        (PipeKind::StartRight, Right) => (FILL_HORI_START_END, false, false),
        (PipeKind::StartTop, Up) => (FILL_VERT_START_END, false, false),
        (PipeKind::StartBottom, _) => (FILL_VERT_START_END, false, true),
        (PipeKind::Horizontal | PipeKind::EndRight | PipeKind::Cross, Left) => {
            (FILL_HORI, false, false)
        }
        // Horizontal flow: right -> left
        (PipeKind::Horizontal | PipeKind::EndLeft | PipeKind::Cross, Right) => {
            (FILL_HORI, true, false)
        }
        // Vertical flow: top -> bottom
        (PipeKind::Vertical | PipeKind::EndBottom, Up) => (FILL_VERT, false, false),
        // Vertical flow: bottom -> top
        (PipeKind::Vertical | PipeKind::EndTop, Down) => (FILL_VERT, false, true),
        // Cross vertical uses its own animation
        (PipeKind::Cross, Up) => (FILL_VERT_CROSS, false, false),
        (PipeKind::Cross, Down) => (FILL_VERT_CROSS, false, true),
        // CornerBR (bottom+right): enter Down -> exit Right, enter Right -> exit Down
        (PipeKind::CornerBR, Down) => (FILL_CORNER_1, false, false),
        (PipeKind::CornerBR, Right) => (FILL_CORNER_2, false, false),
        // CornerBL (bottom+left): enter Down -> exit Left, enter Left -> exit Down
        (PipeKind::CornerBL, Down) => (FILL_CORNER_1, true, false),
        (PipeKind::CornerBL, Left) => (FILL_CORNER_2, true, false),
        // CornerTR (top+right): enter Up -> exit Right, enter Right -> exit Up
        (PipeKind::CornerTR, Up) => (FILL_CORNER_1, false, true),
        (PipeKind::CornerTR, Right) => (FILL_CORNER_2, false, true),
        // CornerTL (top+left): enter Up -> exit Left, enter Left -> exit Up
        (PipeKind::CornerTL, Up) => (FILL_CORNER_1, true, true),
        (PipeKind::CornerTL, Left) => (FILL_CORNER_2, true, true),
        // Fallback (shouldn't occur in valid gameplay)
        _ => (FILL_HORI, false, false),
    }
}
