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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn splitmix64_new_stores_seed_as_initial_state()
    {
        let rng = SplitMix64::new(0x0123_4567_89ab_cdef);

        assert_eq!(rng.0, 0x0123_4567_89ab_cdef);
    }

    #[test]
    fn splitmix64_matches_reference_sequence_from_zero_seed()
    {
        let mut rng = SplitMix64::new(0);
        let expected = [
            0xe220_a839_7b1d_cdaf,
            0x6e78_9e6a_a1b9_65f4,
            0x06c4_5d18_8009_454f,
            0xf88b_b8a8_724c_81ec,
            0x1b39_896a_51a8_749b,
            0x53cb_9f0c_747e_a2ea,
            0x2c82_9abe_1f45_32e1,
            0xc584_133a_c916_ab3c,
        ];

        for value in expected
        {
            assert_eq!(rng.next_u64(), value);
        }
        assert_eq!(rng.0, 0xf1bb_cdcb_fa53_e0a8);
    }

    #[test]
    fn splitmix64_wraps_state_before_mixing()
    {
        let mut rng = SplitMix64::new(u64::MAX);

        assert_eq!(rng.next_u64(), 0xe4d9_7177_1b65_2c20);
        assert_eq!(rng.0, 0x9e37_79b9_7f4a_7c14);
    }

    #[test]
    fn xoshiro_from_seed_expands_state_with_splitmix64()
    {
        let rng = Xoshiro256StarStar::from_seed(42);

        assert_eq!(
            rng.0,
            [
                0xbdd7_3226_2feb_6e95,
                0x28ef_e333_b266_f103,
                0x4752_6757_130f_9f52,
                0x581c_e1ff_0e4a_e394,
            ]
        );
    }

    #[test]
    fn xoshiro_matches_reference_sequence_from_seed_42()
    {
        let mut rng = Xoshiro256StarStar::from_seed(42);
        let expected = [
            0x1578_0b2e_0c2e_c716,
            0x6104_d986_6d11_3a7e,
            0xae17_5332_39e4_99a1,
            0xecb8_ad47_03b3_60a1,
            0xfde6_dc7f_e2ec_5e64,
            0xc50d_a531_0179_5238,
            0xb821_5485_5a65_ddb2,
            0xd99a_2743_ebe6_0087,
        ];

        for value in expected
        {
            assert_eq!(rng.next_u64(), value);
        }
    }

    #[test]
    fn xoshiro_next_u64_updates_internal_state_after_returning_result()
    {
        let mut rng = Xoshiro256StarStar::from_seed(0);

        assert_eq!(rng.next_u64(), 0x99ec_5f36_cb75_f2b4);
        assert_eq!(
            rng.0,
            [
                0x74d3_8efb_a8e8_29b7,
                0x8a9c_6b4b_5aad_ed14,
                0xd831_b653_30fc_88e0,
                0xbc83_12de_64d8_5a7e,
            ]
        );
    }

    #[test]
    fn xoshiro_same_seed_instances_are_deterministic_and_independent()
    {
        let mut left = Xoshiro256StarStar::from_seed(42);
        let mut right = Xoshiro256StarStar::from_seed(42);

        assert_eq!(left.next_u64(), right.next_u64());
        assert_eq!(left.next_u64(), right.next_u64());

        let left_third = left.next_u64();
        let left_fourth = left.next_u64();
        let right_third = right.next_u64();

        assert_eq!(right_third, left_third);
        assert_ne!(right_third, left_fourth);
        assert_eq!(right.next_u64(), left_fourth);
    }
}
