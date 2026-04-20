#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod direction;
mod gfx;
pub mod grid_background;
mod menu_cursor;
mod printer;
mod rng;
mod scenes;
mod sound_controller;

use crate::scenes::config::ConfigMode;
use crate::scenes::{SceneHost, SceneAction};
use crate::sound_controller::SoundController;
use agb::display::tiled::VRAM_MANAGER;
use agb::display::{Graphics, Rgb15};
use agb::input::{Button, ButtonController};
use agb::sound::mixer::{Frequency, Mixer};
use resources::bg;
use resources::prelude::*;

extern crate alloc;

const TILE_SIZE: i32 = 8;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mixer = gba.mixer.mixer(Frequency::Hz32768);
    let gfx = gba.graphics.get();
    let button_controller = ButtonController::new();

    run(mixer, gfx, button_controller)
}

fn run(mixer: Mixer, mut gfx: Graphics, mut button_controller: ButtonController) -> ! {
    VRAM_MANAGER.set_background_palettes(bg::PALETTES);
    VRAM_MANAGER.set_background_palette_colour(14, 1, Rgb15::BLACK);

    let mut rng_seed: [u32; 4] = [1; 4];
    let mut sound_controller = SoundController::new(mixer);
    let mut scene = SceneHost::menu();
    let mut pending_action: Option<SceneAction> = None;

    loop {
        let mut frame = gfx.frame();
        button_controller.update();
        rng_seed[1] = rng_seed[1].rotate_left(1);
        rng_seed[1] |= button_controller.is_pressed(Button::Left) as u32;
        rng_seed[1] |= (button_controller.is_pressed(Button::Right) as u32) << 1;
        rng_seed[1] |= (button_controller.is_pressed(Button::Up) as u32) << 2;
        rng_seed[1] |= (button_controller.is_pressed(Button::Down) as u32) << 3;
        rng_seed[1] |= (button_controller.is_pressed(Button::A) as u32) << 4;
        rng_seed[1] |= (button_controller.is_pressed(Button::B) as u32) << 5;
        rng_seed[1] |= (button_controller.is_pressed(Button::L) as u32) << 6;
        rng_seed[1] |= (button_controller.is_pressed(Button::R) as u32) << 7;

        if let Some(action) = pending_action.take() {
            scene = match action {
                SceneAction::Menu => SceneHost::menu(),
                SceneAction::Pipes(diff) => SceneHost::pipes(rng_seed, diff),
                SceneAction::Bricks => SceneHost::bricks(rng_seed),
                SceneAction::Aster => SceneHost::aster(rng_seed),
                SceneAction::Sweeper(size) => SceneHost::sweeper(rng_seed, size),
                SceneAction::Invaders => SceneHost::invaders(rng_seed),
                SceneAction::Lights(size) => SceneHost::lights(size),
                SceneAction::SweeperConfig => SceneHost::config(ConfigMode::Sweeper),
                SceneAction::PipesConfig => SceneHost::config(ConfigMode::Pipes),
                SceneAction::LightsConfig => SceneHost::config(ConfigMode::LightsOut),
                SceneAction::Win | SceneAction::Lose => unreachable!(),
            };
        } else {
            pending_action = scene.update(&mut button_controller, &mut sound_controller);
            if pending_action.is_some() {
                //unload current scene so memory can be reclaimed before
                //creating new scene on next frame
                scene = SceneHost::blank();
            }
        }

        scene.show(&mut frame);
        sound_controller.frame();
        frame.commit();
        rng_seed[0] += 1;
        rng_seed[2] ^= rng_seed[0].wrapping_mul(0x9e3779b9);
    }
}
