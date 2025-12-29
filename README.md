# ???

A gba game?

## Screenshots

## Player Usage

Download gba file from [here](https://github.com/emmabritton/???/releases/latest) and run in an emulator (mGBA recommended)

### Dev Usage

First follow instructions at https://agbrs.dev/book/setup/getting_started.html

#### Run

cargo run --package game

(runs in mGBA)

#### Test

cargo test

(runs in mGBA)

#### Make gba file

agb-gbafix target/thumbv4t-none-eabi/release/??? -o ???.gba

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
