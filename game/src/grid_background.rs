use agb::display::tile_data::TileData;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{GraphicsFrame, Priority};
use agb::fixnum::{Vector2D, vec2};

pub struct GridBackground {
    bg: RegularBackground,
    scroll_offset_timer: i32,
    scroll_speed: i32,
    offset: Vector2D<i32>,
}

impl GridBackground {
    pub fn new(
        priority: Priority,
        scroll_speed: i32,
        tile_data: &TileData,
        tile_idx: usize,
        offset: Vector2D<i32>,
    ) -> Self {
        let mut bg = RegularBackground::new(
            priority,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        for x in 0..32 {
            for y in 0..32 {
                bg.set_tile(
                    vec2(x, y),
                    &tile_data.tiles,
                    tile_data.tile_settings[tile_idx],
                );
            }
        }
        Self {
            bg,
            scroll_speed,
            scroll_offset_timer: 0,
            offset,
        }
    }
}

impl GridBackground {
    pub fn update(&mut self) {
        self.scroll_offset_timer -= 1;
        if self.scroll_offset_timer <= 0 {
            self.scroll_offset_timer = self.scroll_speed;
            let mut new_pos = self.bg.scroll_pos() + self.offset;
            if new_pos.x > 8 {
                //set to 0 instead of SCROLL_SPEED otherwise it stays on this scroll_pos for too long
                self.scroll_offset_timer = 0;
                new_pos = vec2(0, 0);
            }
            self.bg.set_scroll_pos(new_pos);
        }
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
    }
}
