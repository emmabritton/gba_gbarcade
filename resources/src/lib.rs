#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

#[cfg(test)]
#[allow(clippy::empty_loop)]
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    loop {}
}

extern crate alloc;

use agb::sound::mixer::SoundData;
use agb::{include_aseprite, include_background_gfx, include_wav};

include_aseprite!(
    pub mod sprites,
    "gfx/menu_buttons.aseprite",
    "gfx/gbarcade_logo.aseprite",
    "gfx/font.aseprite",
    "gfx/font_white.aseprite",
    "gfx/bricks_16x16.aseprite",
    "gfx/bricks_8x8.aseprite",
    "gfx/lights.aseprite",
    "gfx/minesweeper.aseprite",
    "gfx/invaders.aseprite",
    "gfx/invaders_shots.aseprite",
    "gfx/pipes_sprites.aseprite",
    "gfx/pipe_fills.aseprite",
    "gfx/pipes_scores.aseprite",
    "gfx/cfg_sprites.aseprite",
    "gfx/cfg_logos.aseprite",
    "gfx/brick_serve.aseprite",
    "gfx/win_shade.aseprite",
    "gfx/win.aseprite",
    "gfx/win_highlight.aseprite",
    "gfx/lose.aseprite",
    "gfx/lose_back.aseprite",
    "gfx/aster_8.aseprite",
    "gfx/aster_16.aseprite",
    "gfx/aster_32.aseprite",
    "gfx/aster_64.aseprite",
    "gfx/bricks_level.aseprite",
    "gfx/bricks_32x16.aseprite",
);

include_background_gfx!(
    pub mod bg,
    "222222",
    sweeper => deduplicate "gfx/minesweeper_tiles.aseprite",
    bg_grid => "gfx/bg_grid_Cell.aseprite",
    bg_invaders => deduplicate "gfx/bg_invaders.aseprite",
    bg_invaders_bricks => deduplicate "gfx/bg_invader_bricks.aseprite",
    bg_minesweeper => deduplicate "gfx/bg_minesweeper.aseprite",
    bg_lights_out => deduplicate "gfx/bg_lights_out.aseprite",
    bg_brick_break => deduplicate "gfx/bg_brick_break.aseprite",
    bg_pipes => deduplicate "gfx/bg_pipes.aseprite",
    bg_aster => deduplicate "gfx/bg_aster.aseprite",
    bg_aster_fore => deduplicate "gfx/bg_aster_fore.aseprite",
    bg_pipe_parts => deduplicate "gfx/bg_pipe_parts.aseprite",
    bg_light_out_cell => deduplicate "gfx/bg_light_out_cell.aseprite",
    bg_pipes_sml => deduplicate "gfx/bg_pipe_sml.aseprite",
    bg_pipes_lrg => deduplicate "gfx/bg_pipe_lrg.aseprite",
);

pub static SFX_BRICK_DAMAGE: SoundData = include_wav!("sfx/brick_damage.wav");
pub static SFX_BRICK_BOUNCE: SoundData = include_wav!("sfx/brick_bounce.wav");
pub static SFX_BRICK_BREAK: SoundData = include_wav!("sfx/brick_break.wav");
pub static SFX_BRICK_FLOOR: SoundData = include_wav!("sfx/brick_ball_floor.wav");
pub static SFX_INVADER_UFO_MOVE: SoundData = include_wav!("sfx/ufo2.wav");
pub static SFX_INVADER_PLAYER_DEAD: SoundData = include_wav!("sfx/player_dead.wav");
pub static SFX_INVADER_PLAYER_SHOOT: SoundData = include_wav!("sfx/player_shoot.wav");
pub static SFX_INVADER_PLAYER_MOVE_1: SoundData = include_wav!("sfx/invader_move_1.wav");
pub static SFX_INVADER_PLAYER_MOVE_2: SoundData = include_wav!("sfx/invader_move_2.wav");
pub static SFX_INVADER_DEATH: SoundData = include_wav!("sfx/invader_dead.wav");
pub static SFX_INVADER_CRUMBLE: SoundData = include_wav!("sfx/crumble.wav");
pub static SFX_CLICK: SoundData = include_wav!("sfx/click.wav");
pub static SFX_SELECT: SoundData = include_wav!("sfx/select.wav");
pub static SFX_INVALID: SoundData = include_wav!("sfx/invalid.wav");
pub static SFX_EXPLOSION: SoundData = include_wav!("sfx/explosion.wav");
pub static SFX_SWEEPER_SELECT: SoundData = include_wav!("sfx/sweeper_select.wav");
pub static SFX_SWEEPER_CURSOR: SoundData = include_wav!("sfx/sweeper_click.wav");
pub static SFX_PLACE: SoundData = include_wav!("sfx/place.wav");
pub static SFX_WATER: SoundData = include_wav!("sfx/water.wav");
pub static SFX_WIN: SoundData = include_wav!("sfx/win.wav");
pub static SFX_LOSE: SoundData = include_wav!("sfx/lost.wav");
pub static SFX_LEVEL_UP: SoundData = include_wav!("sfx/level_up.wav");
pub static SFX_EXTRA_LIFE: SoundData = include_wav!("sfx/extra_life.wav");

pub mod prelude {
    pub use crate::bg;
    pub use crate::bg_idx;
    pub use crate::sprites;
}

pub mod bg_idx {
    pub const BLACK: usize = 0;
    pub const WHITE: usize = 0;
    pub const BROWN_DARK: usize = 8;
    pub const BROWN_LIGHT: usize = 9;
}
