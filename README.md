# GBArcade

GBArcade is a collection of simple games for the GameBoy Advance, written in Rust using the AGB framework.

### Games

- Minefield
- Brick Break
- Lights out
- Pipes
- Snake
- Invaders

## Screenshots

## Player Usage

Download gba file from [here](https://github.com/emmabritton/gbarcade/releases/latest) and run in an emulator (mGBA recommended)

### Dev Usage

First follow instructions at https://agbrs.dev/book/setup/getting_started.html

#### Run

`cargo run`

(runs in mGBA)

#### Test

`cargo test --package game`
`cargo test --package resources`  
`cargo test --package game_logic`

(runs in mGBA)

#### Make gba file

`agb-gbafix target/thumbv4t-none-eabi/release/gb_arcade -o gb_arcade.gba`

## Thanks/Tools

- agb
  - https://agbrs.dev/
  - Framework for running rust on GBA
- mGBA
  - https://mgba.io/
  - Testing
- aseprite
  - https://www.aseprite.org/
  - Creating backgrounds and sprites
- abyssbox
  - https://choptop84.github.io/abyssbox-app/
  - Creating music and sound effects
- audacity
  - https://www.audacityteam.org/
  - Editing/encoding music
