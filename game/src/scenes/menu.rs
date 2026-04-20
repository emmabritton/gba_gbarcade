use crate::gfx::{ShowSprite, ShowTag};
use crate::grid_background::GridBackground;
use crate::menu_cursor::MenuCursor;
use crate::scenes::SceneAction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::display::object::{Sprite, Tag};
use agb::display::{GraphicsFrame, Priority};
use agb::fixnum::vec2;
use agb::input::{Button, ButtonController};
use resources::bg;
use resources::sprites::*;

const SCROLL_SPEED: i32 = 10;
const SPRITE_BUTTON_LOGOS: [[&Sprite; 3]; 2] = [
    [
        MENU_BUTTON_ASTER.sprite(0),
        MENU_BUTTON_PIPES.sprite(0),
        MENU_BUTTON_BRICKBREAK.sprite(0),
    ],
    [
        MENU_BUTTON_MINEFIELD.sprite(0),
        MENU_BUTTON_INVADERS.sprite(0),
        MENU_BUTTON_LIGHTS.sprite(0),
    ],
];
const BUTTON_ACTION: [[SceneAction; 3]; 2] = [
    [
        SceneAction::Aster,
        SceneAction::PipesConfig,
        SceneAction::Bricks,
    ],
    [
        SceneAction::SweeperConfig,
        SceneAction::Invaders,
        SceneAction::LightsConfig,
    ],
];
const SPRITE_HIGHLIGHT: &Sprite = MENU_BUTTON_HIGHLIGHT.sprite(0);
const TAG_TITLE: &Tag = &MENU_LOGO;
const BUTTON_COLS: [i32; 3] = [16, 88, 160];
const BUTTON_ROWS: [i32; 2] = [48, 104];
const VISIBLE_COUNTDOWN: u8 = 4;

struct PendingAction {
    action: SceneAction,
    frame_counter: u8,
    is_visible: bool,
    visible_countdown: u8,
}

impl PendingAction {
    pub fn new(action: SceneAction) -> PendingAction {
        Self {
            action,
            frame_counter: 36,
            is_visible: true,
            visible_countdown: VISIBLE_COUNTDOWN,
        }
    }
}

pub struct MenuState {
    grid_bg: GridBackground,
    cursor: MenuCursor,
    pending_action: Option<PendingAction>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            grid_bg: GridBackground::new(Priority::P1, SCROLL_SPEED, &bg::bg_grid, 0, vec2(1, 1)),
            cursor: MenuCursor::new(
                BUTTON_COLS.len() as u8,
                (BUTTON_ROWS.len() * BUTTON_COLS.len()) as u8,
            ),
            pending_action: None,
        }
    }
}

impl MenuState {
    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> Option<SceneAction> {
        self.grid_bg.update();

        if let Some(pending) = &mut self.pending_action {
            if pending.frame_counter > 0 {
                pending.frame_counter -= 1;
                if pending.visible_countdown > 0 {
                    pending.visible_countdown -= 1;
                } else {
                    pending.is_visible = !pending.is_visible;
                    pending.visible_countdown = VISIBLE_COUNTDOWN;
                }
            } else {
                return Some(pending.action);
            }
        } else {
            self.cursor.update(button_controller, sound_controller);

            if button_controller.is_just_pressed(Button::A) {
                sound_controller.play_sfx(SoundEffect::Select);
                let (col, row) = self.cursor.pos_usize();
                self.pending_action = Some(PendingAction::new(BUTTON_ACTION[row][col]))
            }
        }
        None
    }

    fn is_cursor_visible(&self) -> bool {
        self.pending_action.is_none()
            || self
                .pending_action
                .as_ref()
                .map(|pa| pa.is_visible)
                .unwrap_or(false)
    }

    pub fn show(&mut self, frame: &mut GraphicsFrame) {
        self.grid_bg.show(frame);
        let title_start_x = 58;
        let title_y = 8;
        let title_tile_width = 32;
        for x in 0..4 {
            TAG_TITLE.show(
                x,
                vec2(title_start_x + (x as i32 * title_tile_width), title_y),
                frame,
            );
        }

        let (col, row) = self.cursor.pos_usize();
        let pos = vec2(BUTTON_COLS[col], BUTTON_ROWS[row]);
        if self.is_cursor_visible() {
            SPRITE_HIGHLIGHT.show(pos, frame);
        }

        for (y, row) in SPRITE_BUTTON_LOGOS.iter().enumerate() {
            for (x, sprite) in row.iter().enumerate() {
                let pos = vec2(BUTTON_COLS[x], BUTTON_ROWS[y]);
                sprite.show(pos, frame);
            }
        }
    }
}
