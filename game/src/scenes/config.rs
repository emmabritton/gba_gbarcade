use crate::grid_background::GridBackground;
use crate::menu_cursor::MenuCursor;
use crate::printer::WhiteVariWidthText;
use crate::scenes::SceneAction;
use crate::scenes::lights_out::LightGridSize;
use crate::scenes::pipes::PipeDifficulty;
use crate::scenes::sweeper::SweeperGridSize;
use crate::sound_controller::SoundController;
use agb::display::object::{Object, Tag};
use agb::display::{GraphicsFrame, Priority, WIDTH};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use resources::{bg, sprites};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ConfigMode {
    Sweeper,
    Pipes,
    LightsOut,
}

pub enum OptionPx {
    Three(i32, [i32; 3]),
    Four([i32; 2], [i32; 2]),
    Six([i32; 2], [i32; 3]),
}

const PX_THREE: OptionPx = OptionPx::Three(70, [50, 110, 170]);
const PX_FOUR: OptionPx = OptionPx::Four([50, 100], [70, 150]);
const PX_SIX: OptionPx = OptionPx::Six([50, 100], [50, 110, 170]);

impl ConfigMode {
    pub fn create_cursor(self) -> MenuCursor {
        match self {
            ConfigMode::Sweeper => MenuCursor::new(3, 6),
            ConfigMode::Pipes => MenuCursor::new(2, 4),
            ConfigMode::LightsOut => MenuCursor::new(3, 3),
        }
    }

    pub fn create_action(self, idx: usize) -> SceneAction {
        match self {
            ConfigMode::Sweeper => match idx {
                0 => SceneAction::Sweeper(SweeperGridSize::EightEight),
                1 => SceneAction::Sweeper(SweeperGridSize::TwelveEight),
                2 => SceneAction::Sweeper(SweeperGridSize::SixteenEight),
                3 => SceneAction::Sweeper(SweeperGridSize::TwelveTwelve),
                4 => SceneAction::Sweeper(SweeperGridSize::SixteenSixteen),
                5 => SceneAction::Sweeper(SweeperGridSize::TwentyEightSeventeen),
                _ => panic!("invalid config: {self:?} {idx}"),
            },
            ConfigMode::Pipes => match idx {
                0 => SceneAction::Pipes(PipeDifficulty::SmallEasy),
                1 => SceneAction::Pipes(PipeDifficulty::SmallHard),
                2 => SceneAction::Pipes(PipeDifficulty::LargeEasy),
                3 => SceneAction::Pipes(PipeDifficulty::LargeHard),
                _ => panic!("invalid config: {self:?} {idx}"),
            },
            ConfigMode::LightsOut => match idx {
                0 => SceneAction::Lights(LightGridSize::FiveFive),
                1 => SceneAction::Lights(LightGridSize::TenSix),
                2 => SceneAction::Lights(LightGridSize::TwelveEight),
                _ => panic!("invalid config: {self:?} {idx}"),
            },
        }
    }

    pub fn options(self) -> &'static [&'static [&'static str; 3]; 2] {
        match self {
            ConfigMode::Sweeper => &[&["8x8", "12x8", "16x8"], &["12x12", "16x16", "28x17"]],
            ConfigMode::Pipes => &[
                &["Small\nEasy", "Small\nHard", ""],
                &["Large\nEasy", "Large\nHard", ""],
            ],
            ConfigMode::LightsOut => &[&["5x5", "10x6", "12x8"], &["", "", ""]],
        }
    }

    pub fn option_px(self) -> &'static OptionPx {
        match self {
            ConfigMode::Sweeper => &PX_SIX,
            ConfigMode::Pipes => &PX_FOUR,
            ConfigMode::LightsOut => &PX_THREE,
        }
    }

    pub fn logo(self) -> (&'static Tag, usize) {
        match self {
            ConfigMode::Sweeper => (&sprites::CFG_LOGO_SWEEPER, 2),
            ConfigMode::Pipes => (&sprites::CFG_LOGO_PIPES, 2),
            ConfigMode::LightsOut => (&sprites::CFG_LOGO_LIGHTS_OUT, 3),
        }
    }
}

pub struct ConfigState {
    grid_bg: GridBackground,
    cursor: MenuCursor,
    mode: ConfigMode,
}

impl ConfigState {
    pub fn new(mode: ConfigMode) -> Self {
        let grid_bg = GridBackground::new(Priority::P1, 10, &bg::bg_grid, 0, vec2(1, -1));
        let cursor = mode.create_cursor();
        Self {
            grid_bg,
            mode,
            cursor,
        }
    }
}

impl ConfigState {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        self.grid_bg.update();

        self.cursor.update(button_controller, sound_controller);

        if button_controller.is_just_pressed(Button::A) {
            return Some(self.mode.create_action(self.cursor.idx()));
        }

        if button_controller.is_just_pressed(Button::B) {
            return Some(SceneAction::Menu);
        }

        None
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.grid_bg.show(frame);

        let (tag, count) = self.mode.logo();
        let x = (WIDTH / 2) - ((count as i32 * 32) / 2);
        let y = 16;
        for i in 0..count {
            let spr_x = x + (i as i32 * 32);
            Object::new(tag.sprite(i))
                .set_pos(vec2(spr_x, y))
                .show(frame);
        }

        match self.mode.option_px() {
            OptionPx::Three(y, xs) => {
                for (i, x) in xs.iter().enumerate() {
                    show_button(
                        *x,
                        *y,
                        self.mode.options()[0][i],
                        i == self.cursor.idx(),
                        frame,
                    );
                }
            }
            OptionPx::Four(ys, xs) => {
                for (i, y) in ys.iter().enumerate() {
                    for (j, x) in xs.iter().enumerate() {
                        show_button(
                            *x,
                            *y,
                            self.mode.options()[i][j],
                            i * 2 + j == self.cursor.idx(),
                            frame,
                        );
                    }
                }
            }
            OptionPx::Six(ys, xs) => {
                for (i, y) in ys.iter().enumerate() {
                    for (j, x) in xs.iter().enumerate() {
                        show_button(
                            *x,
                            *y,
                            self.mode.options()[i][j],
                            i * 3 + j == self.cursor.idx(),
                            frame,
                        );
                    }
                }
            }
        }
    }
}

fn show_button(x: i32, y: i32, text: &str, show_cursor: bool, frame: &mut GraphicsFrame) {
    let pos = vec2(x, y);
    let printer = WhiteVariWidthText::new(text, 0);
    let text_x = 16 - (printer.width / 2) - 1;
    printer.show(vec2(text_x + x, y + 12), frame);
    if show_cursor {
        Object::new(sprites::CFG_CURSOR.sprite(0))
            .set_pos(pos)
            .show(frame);
    }
    Object::new(sprites::CFG_BUTTON.sprite(0))
        .set_pos(pos)
        .show(frame);
}
