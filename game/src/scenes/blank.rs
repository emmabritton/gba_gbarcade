use crate::scenes::SceneAction;
use crate::sound_controller::SoundController;
use agb::display::tiled::{
    DynamicTile16, RegularBackground, RegularBackgroundSize, TileEffect, TileFormat,
};
use agb::display::{GraphicsFrame, Priority};
use agb::fixnum::vec2;
use agb::input::ButtonController;

//used as placeholder when swapping scenes
//as otherwise sprite vram sometimes fills
//up and crashes

pub struct BlankState {
    bg: RegularBackground,
}

impl BlankState {
    pub fn new() -> Self {
        let tile = DynamicTile16::new().fill_with(1);
        let mut bg = RegularBackground::new(
            Priority::P2,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        for y in 0..20 {
            for x in 0..30 {
                bg.set_tile_dynamic16(vec2(x, y), &tile, TileEffect::new(false, false, 14));
            }
        }
        Self { bg }
    }
}

impl BlankState {
    pub fn update(
        &mut self,
        _button_controller: &mut ButtonController,
        _sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        None
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
    }
}
