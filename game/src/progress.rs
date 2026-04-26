use agb::external::portable_atomic::AtomicU32;
use core::sync::atomic::Ordering;

#[unsafe(link_section = ".ewram.achievements")]
static ACHIEVEMENTS: AtomicU32 = AtomicU32::new(0);

#[repr(usize)]
pub enum Achievement {
    UsedCheatAsteroids,
    UsedCheatBricks,
    UsedCheatInvaders,
    UsedCheatLightsOut,
    UsedCheatPipe,
    UsedCheatSweeper,
    BeatAsteroids,
    BeatInvaders,
    BeatInvadersWithFullLives,
    BeatPipeSmallEasy,
    BeatPipeSmallHard,
    BeatPipeLargeEasy,
    BeatPipeLargeHard,
    BeatSweeper8x8,
    BeatSweeper12x8,
    BeatSweeper16x8,
    BeatSweeper12x12,
    BeatSweeper16x16,
    BeatSweeper28x17,
    BeatLightsOut5x5,
    BeatLightsOut10x6,
    BeatLightsOut12x8,
    BeatBricks3,
    BeatBricks6,
    BeatBricks9,
    BeatBricks10,
    BeatBricks11,
    BeatBricks12,
    BeatBricks12WithoutLosingBall,
}

pub fn set_achievement(achievement: Achievement) {
    let bit = achievement as usize;
    ACHIEVEMENTS.fetch_or(1 << bit, Ordering::SeqCst);
}
