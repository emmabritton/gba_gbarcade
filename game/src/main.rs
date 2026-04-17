#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

mod direction;
mod game_result;
mod gfx;
pub mod grid_background;
mod menu_cursor;
mod printer;
mod rng;
mod scenes;
mod sound_controller;

use crate::scenes::aster::AsterState;
use crate::scenes::blank::BlankState;
use crate::scenes::bricks::BricksState;
use crate::scenes::config::{ConfigMode, ConfigState};
use crate::scenes::invaders::InvadersState;
use crate::scenes::lights_out::LightsOutState;
use crate::scenes::menu::MenuState;
use crate::scenes::pipes::PipesState;
use crate::scenes::sweeper::SweeperState;
use crate::scenes::{Scene, SceneAction};
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
    let mut scene = Scene::Menu(MenuState::new());
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
                SceneAction::Menu => Scene::Menu(MenuState::new()),
                SceneAction::Pipes(diff) => Scene::Pipes(PipesState::new(rng_seed, diff)),
                SceneAction::Bricks => Scene::Bricks(BricksState::new(rng_seed)),
                SceneAction::Aster => Scene::Aster(AsterState::new(rng_seed)),
                SceneAction::Sweeper(diff) => Scene::Sweeper(SweeperState::new(rng_seed, diff)),
                SceneAction::Invaders => Scene::Invaders(InvadersState::new(rng_seed)),
                SceneAction::Lights(diff) => Scene::LightsOut(LightsOutState::new(diff)),
                SceneAction::SweeperConfig => {
                    Scene::Config(ConfigState::new(ConfigMode::Sweeper))
                }
                SceneAction::PipesConfig => Scene::Config(ConfigState::new(ConfigMode::Pipes)),
                SceneAction::LightsConfig => {
                    Scene::Config(ConfigState::new(ConfigMode::LightsOut))
                }
            };
        } else {
            pending_action = scene.update(&mut button_controller, &mut sound_controller);
            if pending_action.is_some() {
                //unload current scene so memory can be reclaimed before
                //creating new scene on next frame
                scene = Scene::Blank(BlankState::new());
            }
        }

        scene.show(&mut frame);
        sound_controller.frame();
        frame.commit();
        rng_seed[0] += 1;
        rng_seed[2] ^= rng_seed[0].wrapping_mul(0x9e3779b9);
    }
}
