use crate::gfx::{ShowSprite, background};
use crate::menu_cursor::MenuCursor;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{GraphicsFrame, HEIGHT, Priority, WIDTH};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use resources::{bg, sprites};

const CELL_ON: usize = 0;
const CELL_OFF: usize = 2;

const LIGHT_SIZE: i32 = 16;
const TILE_PX: i32 = 8;
const MAX_GRID_W: usize = 12;
const MAX_GRID_H: usize = 8;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LightGridSize {
    FiveFive,
    TenSix,
    TwelveEight,
}

impl LightGridSize {
    pub fn width(&self) -> u8 {
        match self {
            LightGridSize::FiveFive => 5,
            LightGridSize::TenSix => 10,
            LightGridSize::TwelveEight => 12,
        }
    }

    pub fn height(&self) -> u8 {
        match self {
            LightGridSize::FiveFive => 5,
            LightGridSize::TenSix => 6,
            LightGridSize::TwelveEight => 8,
        }
    }
}

pub struct LightsOutState {
    cursor: MenuCursor,
    grid: [bool; MAX_GRID_W * MAX_GRID_H],
    grid_width: u8,
    grid_height: u8,
    won: bool,
    bg_black: RegularBackground,
    tile_layer: RegularBackground,
}

impl LightsOutState {
    pub fn new(grid_size: LightGridSize) -> Self {
        let w = grid_size.width();
        let h = grid_size.height();
        let bg_black = background(&bg::bg_lights_out, Priority::P2);
        let tile_layer = RegularBackground::new(
            Priority::P1,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        Self {
            cursor: MenuCursor::new(w, w * h),
            grid: [true; MAX_GRID_W * MAX_GRID_H],
            grid_width: w,
            grid_height: h,
            won: false,
            bg_black,
            tile_layer,
        }
    }

    fn toggle(&mut self, idx: usize) {
        let w = self.grid_width as usize;
        let h = self.grid_height as usize;
        let row = idx / w;
        let col = idx % w;

        self.flip(row, col);
        if row > 0 {
            self.flip(row - 1, col);
        }
        if row + 1 < h {
            self.flip(row + 1, col);
        }
        if col > 0 {
            self.flip(row, col - 1);
        }
        if col + 1 < w {
            self.flip(row, col + 1);
        }
    }

    fn flip(&mut self, row: usize, col: usize) {
        let idx = row * self.grid_width as usize + col;
        self.grid[idx] = !self.grid[idx];
    }

    fn is_solved(&self) -> bool {
        let total = (self.grid_width * self.grid_height) as usize;
        self.grid[..total].iter().all(|&on| !on)
    }

    fn grid_origin(&self) -> (i32, i32) {
        let total_w = self.grid_width as i32 * LIGHT_SIZE;
        let total_h = self.grid_height as i32 * LIGHT_SIZE;
        ((WIDTH - total_w) / 2, (HEIGHT - total_h) / 2 + TILE_PX)
    }

    fn update_tiles(&mut self) {
        let (ox, oy) = self.grid_origin();
        let tile_ox = ox / TILE_PX;
        let tile_oy = oy / TILE_PX;
        let tiles_per_cell = LIGHT_SIZE / TILE_PX;
        let w = self.grid_width as usize;
        let h = self.grid_height as usize;

        for row in 0..h {
            for col in 0..w {
                let idx = row * w + col;
                let tile_idx = if self.grid[idx] { CELL_ON } else { CELL_OFF };
                let tx = tile_ox + col as i32 * tiles_per_cell;
                let ty = tile_oy + row as i32 * tiles_per_cell;
                self.tile_layer.set_tile(
                    vec2(tx, ty),
                    &bg::bg_light_out_cell.tiles,
                    bg::bg_light_out_cell.tile_settings[tile_idx],
                );
                self.tile_layer.set_tile(
                    vec2(tx + 1, ty),
                    &bg::bg_light_out_cell.tiles,
                    bg::bg_light_out_cell.tile_settings[tile_idx + 1],
                );
                self.tile_layer.set_tile(
                    vec2(tx, ty + 1),
                    &bg::bg_light_out_cell.tiles,
                    bg::bg_light_out_cell.tile_settings[tile_idx + 4],
                );
                self.tile_layer.set_tile(
                    vec2(tx + 1, ty + 1),
                    &bg::bg_light_out_cell.tiles,
                    bg::bg_light_out_cell.tile_settings[tile_idx + 5],
                );
            }
        }
    }
}

impl LightsOutState {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        self.cursor.update(button_controller, sound_controller);

        if button_controller.is_just_pressed(Button::A) {
            let idx = self.cursor.idx();
            self.toggle(idx);
            sound_controller.play_sfx(SoundEffect::Place);
            if self.is_solved() {
                return Some(SceneAction::Win)
            }
        }

        None
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame, is_running: bool) {
        self.update_tiles();
        self.tile_layer.show(frame);
        self.bg_black.show(frame);

        if is_running {
            let (ox, oy) = self.grid_origin();
            let (col, row) = self.cursor.pos_usize();
            let x = ox + col as i32 * LIGHT_SIZE;
            let y = oy + row as i32 * LIGHT_SIZE;
            sprites::LIGHT_SELECT.sprite(0).show(vec2(x, y), frame);
        }
    }
}
