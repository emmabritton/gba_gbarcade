use agb::display::object::{Object, Sprite, Tag};
use agb::display::tile_data::TileData;
use agb::display::tiled::{RegularBackground, RegularBackgroundSize, TileFormat};
use agb::display::{GraphicsFrame, Priority};
use agb::fixnum::Vector2D;

pub fn background(data: &'static TileData, priority: Priority) -> RegularBackground {
    let mut background = RegularBackground::new(
        priority,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    background.fill_with(data);

    background
}

pub fn background_stack<const N: usize>(layers: [&'static TileData; N]) -> [RegularBackground; N] {
    assert!(N > 0, "at least 1 background required");
    assert!(N <= 4, "max 4 layers");

    let priorities = [Priority::P3, Priority::P2, Priority::P1, Priority::P0];

    core::array::from_fn(|i| background(layers[i], priorities[i]))
}

pub trait ShowSprite {
    fn show(&self, at: Vector2D<i32>, frame: &mut GraphicsFrame);
}

pub trait ShowTag {
    fn show(&self, idx: usize, at: Vector2D<i32>, frame: &mut GraphicsFrame);
}

impl ShowSprite for &'static Sprite {
    #[inline]
    fn show(&self, at: Vector2D<i32>, frame: &mut GraphicsFrame) {
        Object::new(*self).set_pos(at).show(frame);
    }
}

impl ShowTag for Tag {
    #[inline]
    fn show(&self, idx: usize, at: Vector2D<i32>, frame: &mut GraphicsFrame) {
        Object::new(self.sprite(idx)).set_pos(at).show(frame);
    }
}
