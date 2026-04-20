# GBArcade

GBArcade is a collection of simple games for the GameBoy Advance, written in Rust using the AGB framework.

### Games

- Asteroids
- Pipe Dream
- Brick Break
- Minesweeper
- Space Invaders
- Lights out

## Screenshots

![Screenshot of main menu](https://raw.githubusercontent.com/emmabritton/gba_gbarcade/refs/heads/main/.github/screenshots/ss_menu.png)
![Screenshot of asteroids](https://raw.githubusercontent.com/emmabritton/gba_gbarcade/refs/heads/main/.github/screenshots/ss_aster.png)
![Screenshot of minesweeper](https://raw.githubusercontent.com/emmabritton/gba_gbarcade/refs/heads/main/.github/screenshots/ss_sweeper.png)
![Screenshot of breakout](https://raw.githubusercontent.com/emmabritton/gba_gbarcade/refs/heads/main/.github/screenshots/ss_breakout.png)

## Player Usage

Download gba file from [here](https://github.com/emmabritton/gba_gbarcade/releases/latest) and run in an emulator (mGBA recommended)

Doesn't support saving.

## Example Cartridge

https://shop.insidegadgets.com/product/gba-4mb-rom-only-flash-cart/

### Dev Usage

First follow instructions at https://agbrs.dev/book/setup/getting_started.html

#### Run

`cargo run`

(runs in mGBA)

#### Test

`cargo test --package game`
`cargo test --package resources`  

(runs in mGBA)

#### Make gba file

`agb-gbafix target/thumbv4t-none-eabi/release/gbarcade -o gbarcade.gba`

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
- Bfxr
  - https://www.bfxr.net/
  - Creating sound effects
- audacity
  - https://www.audacityteam.org/
  - Editing/encoding music/sound effects
