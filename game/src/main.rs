#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::display::Graphics;
use agb::display::object::Object;
use agb::fixnum::vec2;
use agb::input::ButtonController;
use agb::sound::mixer::{Frequency, Mixer};
use resources::prelude::*;

extern crate alloc;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    gba.save.init_sram();
    let mixer = gba.mixer.mixer(Frequency::Hz18157);
    let gfx = gba.graphics.get();
    let button_controller = ButtonController::new();

    run(mixer, gfx, button_controller)
}

fn run(mut mixer: Mixer, mut gfx: Graphics, mut button_controller: ButtonController) -> ! {
    loop {
        let mut frame = gfx.frame();
        button_controller.update();

        //game here

        //example
       Object::new(sprites::OK.sprite(0))
            .set_pos(vec2(48,48))
            .show(&mut frame);
        //end example

        mixer.frame();
        frame.commit();
    }
}
