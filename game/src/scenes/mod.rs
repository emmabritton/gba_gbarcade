use crate::scenes::aster::AsterState;
use crate::scenes::blank::BlankState;
use crate::scenes::bricks::BricksState;
use crate::scenes::config::{ConfigMode, ConfigState};
use crate::scenes::game_host_state::GameHostState;
use crate::scenes::invaders::InvadersState;
use crate::scenes::lights_out::{LightGridSize, LightsOutState};
use crate::scenes::menu::MenuState;
use crate::scenes::pipes::{PipeDifficulty, PipesState};
use crate::scenes::sweeper::{SweeperGridSize, SweeperState};
use crate::sound_controller::SoundController;
use agb::display::GraphicsFrame;
use agb::input::{Button, ButtonController};

const RETURN_TO_MENU_TIMER: u8 = 16;

pub mod aster;
pub mod blank;
pub mod bricks;
pub mod config;
mod game_host_state;
pub mod invaders;
pub mod lights_out;
pub mod menu;
pub mod pipes;
pub mod sweeper;

#[allow(clippy::large_enum_variant)]
enum SceneKind {
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

impl SceneKind {
    fn is_game(&self) -> bool {
        matches!(
            self,
            SceneKind::Pipes(_)
                | SceneKind::Bricks(_)
                | SceneKind::Aster(_)
                | SceneKind::Sweeper(_)
                | SceneKind::Invaders(_)
                | SceneKind::LightsOut(_)
        )
    }

    fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        match self {
            SceneKind::Menu(s) => s.update(button_controller, sound_controller),
            SceneKind::Pipes(s) => s.update(button_controller, sound_controller),
            SceneKind::Bricks(s) => s.update(button_controller, sound_controller),
            SceneKind::Sweeper(s) => s.update(button_controller, sound_controller),
            SceneKind::Aster(s) => s.update(button_controller, sound_controller),
            SceneKind::LightsOut(s) => s.update(button_controller, sound_controller),
            SceneKind::Invaders(s) => s.update(button_controller, sound_controller),
            SceneKind::Config(s) => s.update(button_controller, sound_controller),
            SceneKind::Blank(s) => s.update(button_controller, sound_controller),
        }
    }

    fn show(&mut self, frame: &mut GraphicsFrame, is_running: bool) {
        match self {
            SceneKind::Menu(s) => s.show(frame),
            SceneKind::Pipes(s) => s.show(frame, is_running),
            SceneKind::Bricks(s) => s.show(frame),
            SceneKind::Sweeper(s) => s.show(frame, is_running),
            SceneKind::Aster(s) => s.show(frame, is_running),
            SceneKind::Invaders(s) => s.show(frame),
            SceneKind::LightsOut(s) => s.show(frame, is_running),
            SceneKind::Config(s) => s.show(frame),
            SceneKind::Blank(s) => s.show(frame),
        }
    }
}

pub struct SceneHost {
    kind: SceneKind,
    game_result: GameHostState,
    select_held_timer: u8,
}

impl SceneHost {
    pub fn menu() -> Self {
        Self::new(SceneKind::Menu(MenuState::new()))
    }
    pub fn blank() -> Self {
        Self::new(SceneKind::Blank(BlankState::new()))
    }
    pub fn pipes(seed: [u32; 4], diff: PipeDifficulty) -> Self {
        Self::new(SceneKind::Pipes(PipesState::new(seed, diff)))
    }
    pub fn bricks(seed: [u32; 4]) -> Self {
        Self::new(SceneKind::Bricks(BricksState::new(seed)))
    }
    pub fn aster(seed: [u32; 4]) -> Self {
        Self::new(SceneKind::Aster(AsterState::new(seed)))
    }
    pub fn sweeper(seed: [u32; 4], size: SweeperGridSize) -> Self {
        Self::new(SceneKind::Sweeper(SweeperState::new(seed, size)))
    }
    pub fn invaders(seed: [u32; 4]) -> Self {
        Self::new(SceneKind::Invaders(InvadersState::new(seed)))
    }
    pub fn lights(size: LightGridSize) -> Self {
        Self::new(SceneKind::LightsOut(LightsOutState::new(size)))
    }
    pub fn config(mode: ConfigMode) -> Self {
        Self::new(SceneKind::Config(ConfigState::new(mode)))
    }

    fn new(kind: SceneKind) -> Self {
        Self {
            kind,
            game_result: GameHostState::Running,
            select_held_timer: 0,
        }
    }
}

impl SceneHost {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        self.game_result = self.game_result.update(sound_controller);

        if matches!(
            self.game_result,
            GameHostState::Win { .. } | GameHostState::Lose { .. }
        ) {
            if self.game_result.input_allowed()
                && (button_controller.is_just_pressed(Button::A)
                    || button_controller.is_just_pressed(Button::B))
            {
                return Some(SceneAction::Menu);
            }
            return None;
        }

        if self.kind.is_game() {
            if self.game_result == GameHostState::Paused {
                if button_controller.is_just_pressed(Button::Start) {
                    self.game_result = GameHostState::Running;
                } else if button_controller.is_pressed(Button::Select) {
                    self.select_held_timer += 1;
                    if self.select_held_timer > RETURN_TO_MENU_TIMER {
                        return Some(SceneAction::Menu);
                    }
                } else {
                    self.select_held_timer = 0;
                }
                return None;
            } else {
                self.select_held_timer = 0;
            }
            if button_controller.is_just_pressed(Button::Start) {
                self.game_result = GameHostState::Paused;
                return None;
            }
        } else {
            self.select_held_timer = 0;
        }

        match self.kind.update(button_controller, sound_controller) {
            Some(SceneAction::Win) => {
                self.game_result = GameHostState::new_win();
                None
            }
            Some(SceneAction::Lose) => {
                self.game_result = GameHostState::new_lose();
                None
            }
            other => other,
        }
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        self.game_result.show(frame);
        self.kind
            .show(frame, self.game_result == GameHostState::Running);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SceneAction {
    Menu,
    Win,
    Lose,
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
