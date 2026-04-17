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
}

impl BlankState {
    pub fn new() -> Self {
       Self {}
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
        
    }
}
