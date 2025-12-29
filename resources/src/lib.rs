#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb::include_aseprite;

include_aseprite!(
    pub mod sprites,
    "gfx/tst.aseprite",
);

pub mod prelude {
    pub use crate::sprites;
}