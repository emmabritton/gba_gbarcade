use crate::gfx::{ShowSprite, background};
use crate::progress::{Achievement, set_achievement};
use crate::rng::next_u16_in;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{GraphicsFrame, Priority};
use agb::display::{HEIGHT, WIDTH};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use agb::rng::RandomNumberGenerator;
use resources::{bg, sprites};

const TILE_BLANK: usize = 0;
const TILE_REVEALED: usize = 1;
const TILE_MINE: usize = 2;
const TILE_MINE_BOOM: usize = 3;
const TILE_FLAG: usize = 4;
const TILE_NUM_1: usize = 5;
const TILE_INVALID_FLAG: usize = 13;
const TILE_FOUND_MINE: usize = 14;
const TILE_VALID_FLAG: usize = 15;

const TILE_PX: i32 = 8;
const MAX_CELLS: usize = 28 * 17;

const LOSE_RESULT_DELAY: u8 = 90;
const WIN_RESULT_DELAY: u8 = 70;

#[derive(Copy, Clone)]
struct Cell {
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent: u8,
}

impl Cell {
    const fn empty() -> Self {
        Cell {
            is_mine: false,
            is_revealed: false,
            is_flagged: false,
            adjacent: 0,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum State {
    FirstMove,
    Playing,
    CountingDownToWin(u8),
    CountingDownToLose(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SweeperGridSize {
    EightEight,
    TwelveEight,
    SixteenEight,
    TwelveTwelve,
    SixteenSixteen,
    TwentyEightSeventeen,
}

impl SweeperGridSize {
    pub fn width(&self) -> u8 {
        match self {
            SweeperGridSize::EightEight => 8,
            SweeperGridSize::TwelveEight => 12,
            SweeperGridSize::SixteenEight => 16,
            SweeperGridSize::TwelveTwelve => 12,
            SweeperGridSize::SixteenSixteen => 16,
            SweeperGridSize::TwentyEightSeventeen => 28,
        }
    }

    pub fn height(&self) -> u8 {
        match self {
            SweeperGridSize::EightEight => 8,
            SweeperGridSize::TwelveEight => 8,
            SweeperGridSize::SixteenEight => 8,
            SweeperGridSize::TwelveTwelve => 12,
            SweeperGridSize::SixteenSixteen => 16,
            SweeperGridSize::TwentyEightSeventeen => 17,
        }
    }

    fn mine_count(&self) -> u16 {
        match self {
            SweeperGridSize::EightEight => 10,
            SweeperGridSize::TwelveEight => 13,
            SweeperGridSize::SixteenEight => 17,
            SweeperGridSize::TwelveTwelve => 20,
            SweeperGridSize::SixteenSixteen => 40,
            SweeperGridSize::TwentyEightSeventeen => 99,
        }
    }
}

const MOVE_TIMER_DURATION: u8 = 10;

fn neighbors_of_wh(idx: usize, w: usize, h: usize) -> ([usize; 8], usize) {
    let row = idx / w;
    let col = idx % w;
    let mut buf = [0; 8];
    let mut count = 0;
    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nc >= 0 && (nr as usize) < h && (nc as usize) < w {
                buf[count] = nr as usize * w + nc as usize;
                count += 1;
            }
        }
    }
    (buf, count)
}

fn compute_adjacent(
    mines: &[bool; MAX_CELLS],
    adj_out: &mut [u8; MAX_CELLS],
    total: usize,
    width: usize,
    height: usize,
) {
    for i in 0..total {
        if !mines[i] {
            let (ns, nc) = neighbors_of_wh(i, width, height);
            let mut count = 0;
            for j in 0..nc {
                if mines[ns[j]] {
                    count += 1;
                }
            }
            adj_out[i] = count;
        } else {
            adj_out[i] = 0;
        }
    }
}

pub struct SweeperState {
    cells: [Cell; MAX_CELLS],
    flood_stack: [u16; MAX_CELLS],
    width: u8,
    height: u8,
    mine_count: u16,
    revealed_count: u16,
    safe_count: u16,
    state: State,
    boom_idx: u16,
    cursor_x: u8,
    cursor_y: u8,
    rng: RandomNumberGenerator,
    tile_layer: RegularBackground,
    bg_black: RegularBackground,
    move_input_timer: u8,
}

impl SweeperState {
    pub fn new(seed: [u32; 4], grid_size: SweeperGridSize) -> Self {
        let rng = RandomNumberGenerator::new_with_seed(seed);
        let w = grid_size.width();
        let h = grid_size.height();
        let mines = grid_size.mine_count();
        let tile_layer = RegularBackground::new(
            Priority::P1,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let bg_black = background(&bg::bg_minesweeper, Priority::P2);
        let mut state = Self {
            bg_black,
            tile_layer,
            cells: [Cell::empty(); MAX_CELLS],
            flood_stack: [0; MAX_CELLS],
            width: w,
            height: h,
            mine_count: mines,
            revealed_count: 0,
            safe_count: w as u16 * h as u16 - mines,
            state: State::FirstMove,
            boom_idx: 0,
            cursor_x: w / 2,
            cursor_y: h / 2,
            rng,
            move_input_timer: 0,
        };
        state.update_tiles();
        state
    }

    fn cursor_idx(&self) -> usize {
        self.cursor_y as usize * self.width as usize + self.cursor_x as usize
    }

    fn grid_origin(&self) -> (i32, i32) {
        let w = self.width as i32 * TILE_PX;
        let h = self.height as i32 * TILE_PX;
        let (x, y) = ((WIDTH - w) / 2, ((HEIGHT - h) / 2) + TILE_PX);
        (x / TILE_PX, y / TILE_PX)
    }

    fn neighbors_of(&self, idx: usize) -> ([usize; 8], usize) {
        neighbors_of_wh(idx, self.width as usize, self.height as usize)
    }

    // Place mines after the first reveal, guaranteeing the clicked cell and all its neighbors are mine free
    // Then generates 5 candidates and picks the one where the
    // most cells just outside the safe zone have adj==0 (flood fill eligible), which
    // tends to produce larger openings and reduce forced guessing
    // can take a moment on the largest grid
    fn place_mines(&mut self, safe_idx: usize) {
        let total = self.width as usize * self.height as usize;
        let w = self.width as usize;
        let h = self.height as usize;

        let mut safe = [false; MAX_CELLS];
        safe[safe_idx] = true;
        let (ns0, nc0) = neighbors_of_wh(safe_idx, w, h);
        for i in 0..nc0 {
            safe[ns0[i]] = true;
        }

        // Build the ring of cells just outside the safe zone
        // These are the first cells that could flood fill, more of them having adj==0 means a bigger opening
        let mut ring = [0; 48];
        let mut ring_len = 0;
        for i in 0..total {
            if !safe[i] {
                continue;
            }
            let (ns, nc) = neighbors_of_wh(i, w, h);
            for n in ns.iter().take(nc) {
                if safe[*n] {
                    continue;
                }
                let mut already = false;
                for ring_element in ring.iter().take(ring_len) {
                    if *ring_element == *n as u16 {
                        already = true;
                        break;
                    }
                }
                if !already && ring_len < ring.len() {
                    ring[ring_len] = *n as u16;
                    ring_len += 1;
                }
            }
        }

        let mut best_mines = [false; MAX_CELLS];
        let mut best_score = 0;

        for attempt in 0..5 {
            let mut mine_map = [false; MAX_CELLS];
            let mut placed = 0;
            while placed < self.mine_count {
                let idx = next_u16_in(&mut self.rng, 0, (total - 1) as u16) as usize;
                if !safe[idx] && !mine_map[idx] {
                    mine_map[idx] = true;
                    placed += 1;
                }
            }

            // Score: count ring cells that would have adj==0, so no mine neighbors
            let mut score = 0;
            for ring_element in ring {
                let i = ring_element as usize;
                if mine_map[i] {
                    continue;
                }
                let (ns, nc) = neighbors_of_wh(i, w, h);
                let mut mine_count = 0;
                for j in 0..nc {
                    if mine_map[ns[j]] {
                        mine_count += 1;
                    }
                }
                if mine_count == 0 {
                    score += 1;
                }
            }

            if attempt == 0 || score > best_score {
                best_score = score;
                best_mines = mine_map;
            }
        }

        let mut adj = [0; MAX_CELLS];
        compute_adjacent(&best_mines, &mut adj, total, w, h);

        for i in 0..total {
            self.cells[i].is_mine = best_mines[i];
            self.cells[i].adjacent = adj[i];
        }
    }

    // Reveal starting at start_idx
    // Flood fills through zero-adjacent cells
    // Return true if a mine was hit
    fn reveal_from(&mut self, start_idx: usize) -> bool {
        let cell = self.cells[start_idx];
        if cell.is_flagged || cell.is_revealed {
            return false;
        }
        if cell.is_mine {
            self.cells[start_idx].is_revealed = true;
            self.boom_idx = start_idx as u16;
            return true;
        }

        let mut stack_len = 1;
        self.flood_stack[0] = start_idx as u16;

        while stack_len > 0 {
            stack_len -= 1;
            let idx = self.flood_stack[stack_len] as usize;
            if self.cells[idx].is_revealed || self.cells[idx].is_flagged || self.cells[idx].is_mine
            {
                continue;
            }
            self.cells[idx].is_revealed = true;
            self.revealed_count += 1;
            if self.cells[idx].adjacent == 0 {
                let (ns, nc) = self.neighbors_of(idx);
                for n in ns.iter().take(nc) {
                    if !self.cells[*n].is_revealed
                        && !self.cells[*n].is_flagged
                        && stack_len < MAX_CELLS
                    {
                        self.flood_stack[stack_len] = *n as u16;
                        stack_len += 1;
                    }
                }
            }
        }

        false
    }

    fn update_tiles(&mut self) {
        let (tile_ox, tile_oy) = self.grid_origin();
        let w = self.width as usize;
        let h = self.height as usize;
        let won = matches!(self.state, State::CountingDownToWin(_));
        let lost = matches!(self.state, State::CountingDownToLose(_));
        let game_over = won || lost;

        for row in 0..h {
            for col in 0..w {
                let idx = row * w + col;
                let cell = self.cells[idx];
                let tile_idx = if (won || lost) && cell.is_mine && cell.is_flagged {
                    TILE_VALID_FLAG
                } else if won && cell.is_mine {
                    TILE_FOUND_MINE
                } else if lost && cell.is_mine && !cell.is_flagged {
                    if idx == self.boom_idx as usize {
                        TILE_MINE_BOOM
                    } else {
                        TILE_MINE
                    }
                } else if game_over && cell.is_flagged && !cell.is_mine {
                    TILE_INVALID_FLAG
                } else if cell.is_revealed {
                    if cell.adjacent > 0 {
                        TILE_NUM_1 + cell.adjacent as usize - 1
                    } else {
                        TILE_REVEALED
                    }
                } else if cell.is_flagged {
                    TILE_FLAG
                } else {
                    TILE_BLANK
                };
                self.tile_layer.set_tile(
                    vec2(tile_ox + col as i32, tile_oy + row as i32),
                    &bg::sweeper.tiles,
                    bg::sweeper.tile_settings[tile_idx],
                );
            }
        }
    }
}

impl SweeperState {
    pub fn cheat(&mut self) {
        set_achievement(Achievement::UsedCheatSweeper);
        if self.state != State::Playing {
            return;
        }
        let total = self.width as usize * self.height as usize;
        let mut candidates = [0u16; MAX_CELLS];
        let mut count = 0;
        for i in 0..total {
            let cell = self.cells[i];
            if cell.is_mine && !cell.is_flagged && !cell.is_revealed {
                candidates[count] = i as u16;
                count += 1;
            }
        }
        if count == 0 {
            return;
        }
        let pick = next_u16_in(&mut self.rng, 0, (count - 1) as u16) as usize;
        self.cells[candidates[pick] as usize].is_flagged = true;
        self.update_tiles();
    }

    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        match self.state {
            State::FirstMove => {}
            State::Playing => {}
            State::CountingDownToWin(frames_remaining) => {
                if frames_remaining > 0 {
                    self.state = State::CountingDownToWin(frames_remaining - 1);
                } else {
                    match (self.width, self.height) {
                        (8, 8) => set_achievement(Achievement::BeatSweeper8x8),
                        (12, 8) => set_achievement(Achievement::BeatSweeper12x8),
                        (16, 8) => set_achievement(Achievement::BeatSweeper16x8),
                        (12, 12) => set_achievement(Achievement::BeatSweeper12x12),
                        (16, 16) => set_achievement(Achievement::BeatSweeper16x16),
                        (28, 17) => set_achievement(Achievement::BeatSweeper28x17),
                        _ => {}
                    }
                    return Some(SceneAction::Win);
                }
            }
            State::CountingDownToLose(frames_remaining) => {
                if frames_remaining > 0 {
                    self.state = State::CountingDownToLose(frames_remaining - 1);
                } else {
                    return Some(SceneAction::Lose);
                }
            }
        }

        if self.move_input_timer > 0 {
            self.move_input_timer -= 1;
        } else {
            if button_controller.is_pressed(Button::Up) && self.cursor_y > 0 {
                sound_controller.play_sfx(SoundEffect::SweeperCursor);
                self.cursor_y -= 1;
                self.move_input_timer = MOVE_TIMER_DURATION;
            }
            if button_controller.is_pressed(Button::Down) && self.cursor_y + 1 < self.height {
                sound_controller.play_sfx(SoundEffect::SweeperCursor);
                self.cursor_y += 1;
                self.move_input_timer = MOVE_TIMER_DURATION;
            }
            if button_controller.is_pressed(Button::Left) && self.cursor_x > 0 {
                sound_controller.play_sfx(SoundEffect::SweeperCursor);
                self.cursor_x -= 1;
                self.move_input_timer = MOVE_TIMER_DURATION;
            }
            if button_controller.is_pressed(Button::Right) && self.cursor_x + 1 < self.width {
                sound_controller.play_sfx(SoundEffect::SweeperCursor);
                self.cursor_x += 1;
                self.move_input_timer = MOVE_TIMER_DURATION;
            }
        }
        if button_controller.vector() == vec2(0, 0) {
            self.move_input_timer = 0;
        }

        let idx = self.cursor_idx();

        if button_controller.is_just_pressed(Button::A) {
            sound_controller.play_sfx(SoundEffect::SweeperSelect);
            match self.state {
                State::FirstMove => {
                    self.place_mines(idx);
                    self.state = State::Playing;
                    if self.reveal_from(idx) {
                        panic!("First click contained mine");
                    } else if self.revealed_count >= self.safe_count {
                        self.state = State::CountingDownToWin(WIN_RESULT_DELAY);
                    }
                    self.update_tiles();
                }
                State::Playing => {
                    if self.reveal_from(idx) {
                        sound_controller.play_sfx(SoundEffect::Explosion);
                        self.state = State::CountingDownToLose(LOSE_RESULT_DELAY);
                    } else if self.revealed_count >= self.safe_count {
                        self.state = State::CountingDownToWin(WIN_RESULT_DELAY);
                    }
                    self.update_tiles();
                }
                State::CountingDownToLose(_) => {}
                State::CountingDownToWin(_) => {}
            }
        }

        if button_controller.is_just_pressed(Button::B) {
            sound_controller.play_sfx(SoundEffect::SweeperSelect);
            let cell = &mut self.cells[idx];
            if !cell.is_revealed {
                cell.is_flagged = !cell.is_flagged;
                self.update_tiles();
            }
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame, is_running: bool) {
        self.tile_layer.show(frame);
        self.bg_black.show(frame);

        if is_running {
            let (ox, oy) = self.grid_origin();
            let x = (ox + self.cursor_x as i32) * TILE_PX;
            let y = (oy + self.cursor_y as i32) * TILE_PX;
            sprites::MINESWEEPER_SELECT
                .sprite(0)
                .show(vec2(x, y), frame);
        }
    }
}
