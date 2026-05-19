// Rust port of xoshiro256** 1.0 by David Blackman and Sebastiano Vigna.
// Original C source: https://prng.di.unimi.it/xoshiro256starstar.c
// The license of the original work is as below:

/*  Written in 2018 by David Blackman and Sebastiano Vigna (vigna@acm.org)

To the extent possible under law, the author has dedicated all copyright
and related and neighboring rights to this software to the public domain
worldwide.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR
IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE. */

#[derive(Debug, Clone, Copy)]
pub struct Xoshiro256StarStar([u64; 4]);

impl Xoshiro256StarStar
{
    pub fn from_seed(seed: u64) -> Self
    {
        let mut sm = SplitMix64::new(seed);

        Self([
            sm.next_u64(),
            sm.next_u64(),
            sm.next_u64(),
            sm.next_u64(),
        ])
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64
    {
        let result = self.0[1]
            .wrapping_mul(5)
            .rotate_left(7)
            .wrapping_mul(9);

        let t = self.0[1] << 17;

        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];

        self.0[2] ^= t;
        self.0[3] = self.0[3].rotate_left(45);

        result
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SplitMix64(u64);

impl SplitMix64
{
    pub fn new(seed: u64) -> Self
    {
        Self(seed)
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64
    {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);

        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
