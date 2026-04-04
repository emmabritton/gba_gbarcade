use crate::scenes::aster::AsterState;
use crate::scenes::blank::BlankState;
use crate::scenes::bricks::BricksState;
use crate::scenes::config::ConfigState;
use crate::scenes::invaders::InvadersState;
use crate::scenes::lights_out::{LightGridSize, LightsOutState};
use crate::scenes::menu::MenuState;
use crate::scenes::pipes::{PipeDifficulty, PipesState};
use crate::scenes::sweeper::{SweeperGridSize, SweeperState};
use crate::sound_controller::SoundController;
use agb::display::GraphicsFrame;
use agb::input::ButtonController;

pub mod aster;
pub mod blank;
pub mod bricks;
pub mod config;
pub mod invaders;
pub mod lights_out;
pub mod menu;
pub mod pipes;
pub mod sweeper;

#[allow(clippy::large_enum_variant)]
pub enum Scene {
    Menu(MenuState),
    Pipes(PipesState),
    Bricks(BricksState),
    Aster(AsterState),
    Sweeper(SweeperState),
    Invaders(InvadersState),
    LightsOut(LightsOutState),
    Config(ConfigState),
    Blank(BlankState),
}

impl Scene {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        match self {
            Scene::Menu(state) => state.update(button_controller, sound_controller),
            Scene::Pipes(state) => state.update(button_controller, sound_controller),
            Scene::Bricks(state) => state.update(button_controller, sound_controller),
            Scene::Sweeper(state) => state.update(button_controller, sound_controller),
            Scene::Aster(state) => state.update(button_controller, sound_controller),
            Scene::LightsOut(state) => state.update(button_controller, sound_controller),
            Scene::Invaders(state) => state.update(button_controller, sound_controller),
            Scene::Config(state) => state.update(button_controller, sound_controller),
            Scene::Blank(state) => state.update(button_controller, sound_controller),
        }
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        match self {
            Scene::Menu(state) => state.show(frame),
            Scene::Pipes(state) => state.show(frame),
            Scene::Bricks(state) => state.show(frame),
            Scene::Sweeper(state) => state.show(frame),
            Scene::Aster(state) => state.show(frame),
            Scene::Invaders(state) => state.show(frame),
            Scene::LightsOut(state) => state.show(frame),
            Scene::Config(state) => state.show(frame),
            Scene::Blank(state) => state.show(frame),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SceneAction {
    Menu,
    PipesConfig,
    Pipes(PipeDifficulty),
    Bricks,
    Aster,
    SweeperConfig,
    Sweeper(SweeperGridSize),
    LightsConfig,
    Lights(LightGridSize),
    Invaders,
}
