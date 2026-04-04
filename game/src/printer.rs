use agb::display::GraphicsFrame;
use agb::display::object::{Object, Sprite};
use agb::fixnum::{Vector2D, vec2};
use alloc::vec::Vec;

pub struct WhiteVariWidthText {
    sprites: Vec<&'static Sprite>,
    offsets: Vec<Vector2D<i32>>,
    pub width: i32,
}

impl WhiteVariWidthText {
    pub fn new(text: &str, extra_line_height: i32) -> Self {
        let mut sprites = Vec::with_capacity(text.len());
        let mut offsets = Vec::with_capacity(text.len());

        let mut width = 0;
        let mut x = 0;
        let mut y = 0;

        for c in text.chars() {
            if c == ' ' {
                x += char_width(c);
                continue;
            }
            if c == '\n' {
                if x > width {
                    width = x;
                }
                x = 0;
                y += 8 + extra_line_height;
                continue;
            }

            let sprite = white_char_to_sprite(c);

            x -= char_offset(c);

            sprites.push(sprite);
            offsets.push(vec2(x, y));

            x += char_width(c) + 1;
        }
        if x > width {
            width = x;
        }

        Self {
            sprites,
            offsets,
            width,
        }
    }

    pub fn show(&self, pos: Vector2D<i32>, frame: &mut GraphicsFrame) {
        for (&sprite, &offset) in self.sprites.iter().zip(self.offsets.iter()) {
            Object::new(sprite).set_pos(pos + offset).show(frame);
        }
    }
}

pub const fn white_char_to_sprite(c: char) -> &'static Sprite {
    match c {
        '0'..='9' => {
            let index = c as usize - '0' as usize;
            crate::sprites::NUMBERS_WHITE.sprite(index)
        }
        'A'..='Z' => {
            let index = c as usize - 'A' as usize;
            crate::sprites::UPPER_WHITE.sprite(index)
        }
        'a'..='z' => {
            let index = c as usize - 'a' as usize;
            crate::sprites::LOWER_WHITE.sprite(index)
        }
        '!' => crate::sprites::SYM_EXCLAIM_WHITE.sprite(0),
        '?' => crate::sprites::SYM_QUESTION_WHITE.sprite(0),
        '.' => crate::sprites::SYM_PERIOD_WHITE.sprite(0),
        ',' => crate::sprites::SYM_COMMA_WHITE.sprite(0),
        ':' => crate::sprites::SYM_COLON_WHITE.sprite(0),
        '#' => crate::sprites::SYM_HASH_WHITE.sprite(0),
        '(' => crate::sprites::SYM_PAREN_L_WHITE.sprite(0),
        ')' => crate::sprites::SYM_PAREN_R_WHITE.sprite(0),
        '%' => crate::sprites::SYM_PERCENT_WHITE.sprite(0),
        '-' => crate::sprites::SYM_DASH_WHITE.sprite(0),
        '+' => crate::sprites::SYM_PLUS_WHITE.sprite(0),
        ' ' => crate::sprites::SPACE_WHITE.sprite(0),
        _ => crate::sprites::UNKNOWN_WHITE.sprite(0),
    }
}

/// Returns the width in pixels of the given character with starting padding
pub const fn char_width(c: char) -> i32 {
    match c {
        '.' | '!' | ':' | 'i' | ',' | 'l' => 2,
        '(' | ')' | 'j' => 3,
        'T' | 'c' | 'e'..='g' | 'r'..='s' | 'v' | 'x'..='z' => 4,
        '0'..='9'
        | 'A'..='H'
        | 'J'..='L'
        | 'I'
        | 'P'
        | 'R'
        | 'S'
        | 'U'..='V'
        | 'Z'
        | '?'
        | 'b'
        | 'd'
        | 'h'
        | 'k'
        | 'n'..='q'
        | 't'..='u'
        | '-' => 5,
        'X'..='Y' | '#' | '%' | 'a' | 'm' | 'w' | '+' => 6,
        'Q' | 'M'..='O' | 'W' => 7,
        ' ' => 4,
        _ => 7,
    }
}

/// Returns the start in pixels of the given character with starting padding
pub const fn char_offset(c: char) -> i32 {
    match c {
        'Q'
        | 'M'
        | 'N'
        | 'O'
        | 'W'
        | 'X'
        | 'Y'
        | 'Z'
        | '.'
        | ','
        | '('
        | ')'
        | '?'
        | '!'
        | ':'
        | '#'
        | '%'
        | 'a'..='z'
        | ' ' => 0,
        '1' | '3' | 'I' => 2,
        _ => 1,
    }
}
