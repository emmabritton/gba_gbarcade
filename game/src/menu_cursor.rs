use crate::direction::Direction;
use crate::sound_controller::{SoundController, SoundEffect};
use agb::input::ButtonController;

pub struct MenuCursor {
    idx: u8,
    max_x: u8,
    total: u8,
}

impl MenuCursor {
    pub fn new(column_count: u8, count: u8) -> Self {
        assert!(
            column_count > 0,
            "ListCursor created with {}, {}",
            column_count,
            count
        );
        Self {
            idx: 0,
            max_x: column_count,
            total: count,
        }
    }
}

impl MenuCursor {
    #[inline]
    pub fn idx(&self) -> usize {
        self.idx as usize
    }

    #[inline]
    pub fn pos_usize(&self) -> (usize, usize) {
        (
            (self.idx % self.max_x) as usize,
            (self.idx / self.max_x) as usize,
        )
    }

    pub fn update(
        &mut self,
        button_controller: &mut ButtonController,
        sound_controller: &mut SoundController,
    ) -> bool {
        if let Some(dir) = Direction::from_recent_input(button_controller) {
            let col = self.idx % self.max_x;

            match dir {
                Direction::Up => {
                    if self.idx >= self.max_x {
                        self.idx -= self.max_x;
                        sound_controller.play_sfx(SoundEffect::Cursor);
                    } else {
                        sound_controller.play_sfx(SoundEffect::InvalidCursor);
                        return false;
                    }
                }
                Direction::Down => {
                    if self.idx + self.max_x < self.total {
                        self.idx += self.max_x;
                        sound_controller.play_sfx(SoundEffect::Cursor);
                    } else {
                        sound_controller.play_sfx(SoundEffect::InvalidCursor);
                        return false;
                    }
                }
                Direction::Left => {
                    if col > 0 {
                        self.idx -= 1;
                        sound_controller.play_sfx(SoundEffect::Cursor);
                    } else {
                        sound_controller.play_sfx(SoundEffect::InvalidCursor);
                        return false;
                    }
                }
                Direction::Right => {
                    if (col < self.max_x - 1) && (self.idx + 1 < self.total) {
                        self.idx += 1;
                        sound_controller.play_sfx(SoundEffect::Cursor);
                    } else {
                        sound_controller.play_sfx(SoundEffect::InvalidCursor);
                        return false;
                    }
                }
            }

            true
        } else {
            false
        }
    }
}
