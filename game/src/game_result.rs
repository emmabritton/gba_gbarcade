use crate::gfx::{ShowSprite, ShowTag};
use agb::display::GraphicsFrame;
use agb::display::object::Tag;
use agb::fixnum::{Vector2D, vec2};
use resources::sprites;

const WIN_INPUT_DELAY: u8 = 60;
const LOSE_INPUT_DELAY: u8 = 30;

const WIN_SPEED: u8 = 6;
const LOSE_SPEED: u8 = 16;

const TAG_OFFSET: Vector2D<i32> = vec2(24, 48);

#[derive(Debug)]
pub enum GameResult {
    Win {
        input_delay: u8,
        letter_idx: u8,
        frame_counter: u8,
    },
    Lose {
        input_delay: u8,
        dark: bool,
        frame_counter: u8,
    },
}

impl GameResult {
    pub fn new_win() -> GameResult {
        GameResult::Win {
            input_delay: WIN_INPUT_DELAY,
            letter_idx: 0,
            frame_counter: 0,
        }
    }

    pub fn new_lose() -> GameResult {
        GameResult::Lose {
            input_delay: LOSE_INPUT_DELAY,
            dark: false,
            frame_counter: 0,
        }
    }

    pub fn input_allowed(&self) -> bool {
        match self {
            GameResult::Win { input_delay, .. } => *input_delay == 0,
            GameResult::Lose { input_delay, .. } => *input_delay == 0,
        }
    }

    pub fn update(&self) -> GameResult {
        match self {
            GameResult::Win {
                input_delay,
                letter_idx,
                frame_counter,
            } => {
                let input_delay = (input_delay).saturating_sub(1);
                let (letter_idx, frame_counter) = if *frame_counter > WIN_SPEED {
                    let letter_idx = if *letter_idx >= 6 { 0 } else { letter_idx + 1 };
                    (letter_idx, 0)
                } else {
                    (*letter_idx, frame_counter + 1)
                };
                GameResult::Win {
                    input_delay,
                    letter_idx,
                    frame_counter,
                }
            }
            GameResult::Lose {
                input_delay,
                dark,
                frame_counter,
            } => {
                let input_delay = input_delay.saturating_sub(1);
                let (dark, frame_counter) = if *frame_counter > LOSE_SPEED {
                    (!dark, 0)
                } else {
                    (*dark, *frame_counter + 1)
                };
                GameResult::Lose {
                    input_delay,
                    dark,
                    frame_counter,
                }
            }
        }
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        match self {
            GameResult::Win { letter_idx, .. } => {
                let idx = *letter_idx as usize;
                let idx2 = (idx + 4) % 7;
                let letter_x_offsets = [29, 47, 65, 91, 115, 125, 153];
                let letter_y_offset = 23;
                sprites::WIN_HIGHLIGHT.show(
                    idx,
                    TAG_OFFSET + vec2(letter_x_offsets[idx], letter_y_offset),
                    frame,
                );
                sprites::WIN_HIGHLIGHT.show(
                    idx2,
                    TAG_OFFSET + vec2(letter_x_offsets[idx2], letter_y_offset),
                    frame,
                );
                draw_badge(&sprites::WIN_BADGE, frame);
            }
            GameResult::Lose { dark, .. } => {
                let tag = if *dark {
                    &sprites::LOSE_DARK
                } else {
                    &sprites::LOSE_LIGHT
                };
                tag.sprites().iter().enumerate().for_each(|(i, sprite)| {
                    let pos = TAG_OFFSET + vec2(25, 24) + vec2(i as i32 * 32, 0);
                    sprite.show(pos, frame);
                });
                draw_badge(&sprites::LOSE_BADGE, frame);
            }
        }
    }
}

fn draw_badge(tag: &Tag, frame: &mut GraphicsFrame) {
    tag.sprites().iter().enumerate().for_each(|(i, sprite)| {
        let pos = TAG_OFFSET + vec2(i as i32 * 64, 0);
        sprite.show(pos, frame);
    });
    for i in 0..4 {
        sprites::WIN_LOSE_SHADE.show(0, vec2(i * 64, -16), frame);
        sprites::WIN_LOSE_SHADE.show(0, vec2(i * 64, 112), frame);
    }
    sprites::WIN_LOSE_SHADE.show(0, vec2(-40, 48), frame);
    sprites::WIN_LOSE_SHADE.show(0, vec2(176, 48), frame);
}
