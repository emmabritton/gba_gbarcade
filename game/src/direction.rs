use agb::input::{Button, ButtonController};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_recent_input(buttons: &ButtonController) -> Option<Direction> {
        if buttons.is_just_pressed(Button::Up) {
            Some(Direction::Up)
        } else if buttons.is_just_pressed(Button::Down) {
            Some(Direction::Down)
        } else if buttons.is_just_pressed(Button::Left) {
            Some(Direction::Left)
        } else if buttons.is_just_pressed(Button::Right) {
            Some(Direction::Right)
        } else {
            None
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}
