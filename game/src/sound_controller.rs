use agb::fixnum::num;
use agb::sound::mixer::{Mixer, SoundChannel};
use resources::*;

pub struct SoundController<'gba> {
    mixer: Mixer<'gba>,
}

impl<'gba> SoundController<'gba> {
    pub fn new(mixer: Mixer<'gba>) -> Self {
        Self { mixer }
    }
}

impl<'gba> SoundController<'gba> {
    pub fn frame(&mut self) {
        self.mixer.frame();
    }

    pub fn play_sfx(&mut self, effect: SoundEffect) {
        let sound_data = match effect {
            SoundEffect::Cursor => SFX_CLICK,
            SoundEffect::InvalidCursor => SFX_INVALID,
            SoundEffect::Select => SFX_SELECT,
            SoundEffect::Place => SFX_PLACE,
            SoundEffect::BrickBreak => SFX_BRICK_BREAK,
            SoundEffect::BrickBounce => SFX_BRICK_BOUNCE,
            SoundEffect::BrickDamage => SFX_BRICK_DAMAGE,
            SoundEffect::BrickFloor => SFX_BRICK_FLOOR,
            SoundEffect::InvaderDeath => SFX_INVADER_DEATH,
            SoundEffect::InvaderUfo => SFX_INVADER_UFO_MOVE,
            SoundEffect::InvaderPlayerFire => SFX_INVADER_PLAYER_SHOOT,
            SoundEffect::InvaderPlayerDeath => SFX_INVADER_PLAYER_DEAD,
            SoundEffect::InvaderMove1 => SFX_INVADER_PLAYER_MOVE_1,
            SoundEffect::InvaderMove2 => SFX_INVADER_PLAYER_MOVE_2,
            SoundEffect::InvaderCrumble => SFX_INVADER_CRUMBLE,
            SoundEffect::Explosion => SFX_EXPLOSION,
            SoundEffect::SweeperSelect => SFX_SWEEPER_SELECT,
            SoundEffect::SweeperCursor => SFX_SWEEPER_CURSOR,
            SoundEffect::Water => SFX_WATER,
            SoundEffect::Win => SFX_WIN,
            SoundEffect::Lose => SFX_LOSE,
        };

        let mut channel = SoundChannel::new(sound_data);
        channel.stereo();
        if effect == SoundEffect::Lose {
            channel.volume(num!(3));
        }
        self.mixer.play_sound(channel);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SoundEffect {
    Cursor,
    InvalidCursor,
    Select,
    Place,
    BrickBreak,
    BrickBounce,
    BrickDamage,
    BrickFloor,
    InvaderDeath,
    InvaderUfo,
    InvaderPlayerFire,
    InvaderPlayerDeath,
    InvaderMove1,
    InvaderMove2,
    InvaderCrumble,
    Explosion,
    SweeperSelect,
    SweeperCursor,
    Water,
    Win,
    Lose,
}
