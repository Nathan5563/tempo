use super::{super::utils, super::prng};

pub type ZobristType = u64;
const NUM_CASTLING_STATES: usize = 16;
const NUM_ENPASSANT_STATES: usize = 17;
const PRNG_SEED: u64 = 42;

#[derive(Debug)]
pub struct ZobristRandoms
{
    active: [ZobristType; utils::NUM_COLORS],
    pieces: [[[ZobristType; utils::NUM_SQUARES]; utils::NUM_PIECE_TYPES]; utils::NUM_COLORS],
    castling: [ZobristType; NUM_CASTLING_STATES],
    enpassant: [ZobristType; NUM_ENPASSANT_STATES],
}

impl Default for ZobristRandoms
{
    fn default() -> Self
    {
        let mut rng = prng::Xoshiro256StarStar::from_seed(PRNG_SEED);
        let active = [(); utils::NUM_COLORS].map(|_| rng.next_u64());
        let pieces = [(); utils::NUM_COLORS].map(|_| {
            [(); utils::NUM_PIECE_TYPES].map(|_| {
                [(); utils::NUM_SQUARES].map(|_| rng.next_u64())
            })
        });
        let castling = [(); NUM_CASTLING_STATES].map(|_| rng.next_u64());
        let enpassant = [(); NUM_ENPASSANT_STATES].map(|_| rng.next_u64());
        ZobristRandoms { active, pieces, castling, enpassant }
    }
}
