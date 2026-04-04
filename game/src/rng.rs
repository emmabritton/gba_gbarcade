use agb::rng::RandomNumberGenerator;

#[inline]
pub fn next_u16(rng: &mut RandomNumberGenerator) -> u16 {
    next_i32(rng) as u16
}

//inclusive
#[inline]
pub fn next_u16_in(rng: &mut RandomNumberGenerator, min: u16, max: u16) -> u16 {
    if min >= max {
        return min;
    }

    let span = (max as u32).wrapping_sub(min as u32).wrapping_add(1);
    let random_val = next_u16(rng) as u32;

    let scaled = (random_val * span) >> 16;

    (scaled as u16) + min
}

#[inline]
pub fn next_i32(rng: &mut RandomNumberGenerator) -> i32 {
    rng.next_i32()
}
