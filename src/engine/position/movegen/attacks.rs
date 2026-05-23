#![allow(long_running_const_eval)]

use super::super::{super::utils, bitboard::BitBoard};
use self::magics::{
    BISHOP_MAGICS,
    BISHOP_MASKS,
    BISHOP_SHIFTS,
    ROOK_MAGICS,
    ROOK_MASKS,
    ROOK_SHIFTS,
};

mod magics;

const ROOK_TABLE_SIZE: usize = 4096;
const BISHOP_TABLE_SIZE: usize = 512;

static PAWN_ATTACKS: [[u64; utils::NUM_SQUARES]; utils::NUM_COLORS] = init_pawn_attacks();
static KNIGHT_ATTACKS: [u64; utils::NUM_SQUARES] = init_knight_attacks();
static KING_ATTACKS: [u64; utils::NUM_SQUARES] = init_king_attacks();
static ROOK_ATTACKS: [[u64; ROOK_TABLE_SIZE]; utils::NUM_SQUARES] = init_rook_attacks();
static BISHOP_ATTACKS: [[u64; BISHOP_TABLE_SIZE]; utils::NUM_SQUARES] = init_bishop_attacks();

pub(super) fn pawn_attacks(
    square: utils::Square,
    color: utils::Color,
) -> BitBoard
{
    BitBoard::from_bits(PAWN_ATTACKS[color as usize][square as usize])
}

pub(super) fn knight_attacks(square: utils::Square) -> BitBoard
{
    BitBoard::from_bits(KNIGHT_ATTACKS[square as usize])
}

pub(super) fn bishop_attacks(
    square: utils::Square,
    occupied: BitBoard,
) -> BitBoard
{
    let index = bishop_index(square as usize, occupied.bits());
    BitBoard::from_bits(BISHOP_ATTACKS[square as usize][index])
}

pub(super) fn rook_attacks(
    square: utils::Square,
    occupied: BitBoard,
) -> BitBoard
{
    let index = rook_index(square as usize, occupied.bits());
    BitBoard::from_bits(ROOK_ATTACKS[square as usize][index])
}

pub(super) fn queen_attacks(
    square: utils::Square,
    occupied: BitBoard,
) -> BitBoard
{
    bishop_attacks(square, occupied) | rook_attacks(square, occupied)
}

pub(super) fn king_attacks(square: utils::Square) -> BitBoard
{
    BitBoard::from_bits(KING_ATTACKS[square as usize])
}

const fn init_pawn_attacks() -> [[u64; utils::NUM_SQUARES]; utils::NUM_COLORS]
{
    let mut attacks = [[0; utils::NUM_SQUARES]; utils::NUM_COLORS];
    let mut square = 0;

    while square < utils::NUM_SQUARES
    {
        let rank = rank(square);
        let file = file(square);

        if rank < 7
        {
            if file > 0
            {
                attacks[utils::Color::White as usize][square] |= bit(square + 7);
            }
            if file < 7
            {
                attacks[utils::Color::White as usize][square] |= bit(square + 9);
            }
        }

        if rank > 0
        {
            if file > 0
            {
                attacks[utils::Color::Black as usize][square] |= bit(square - 9);
            }
            if file < 7
            {
                attacks[utils::Color::Black as usize][square] |= bit(square - 7);
            }
        }

        square += 1;
    }

    attacks
}

const fn init_knight_attacks() -> [u64; utils::NUM_SQUARES]
{
    let mut attacks = [0; utils::NUM_SQUARES];
    let mut square = 0;

    while square < utils::NUM_SQUARES
    {
        attacks[square] = leaper_attacks(
            square,
            [
                (2, 1),
                (1, 2),
                (-1, 2),
                (-2, 1),
                (-2, -1),
                (-1, -2),
                (1, -2),
                (2, -1),
            ],
        );
        square += 1;
    }

    attacks
}

const fn init_king_attacks() -> [u64; utils::NUM_SQUARES]
{
    let mut attacks = [0; utils::NUM_SQUARES];
    let mut square = 0;

    while square < utils::NUM_SQUARES
    {
        attacks[square] = leaper_attacks(
            square,
            [
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ],
        );
        square += 1;
    }

    attacks
}

const fn init_rook_attacks() -> [[u64; ROOK_TABLE_SIZE]; utils::NUM_SQUARES]
{
    let mut attacks = [[0; ROOK_TABLE_SIZE]; utils::NUM_SQUARES];
    let mut square = 0;

    while square < utils::NUM_SQUARES
    {
        let mask = ROOK_MASKS[square];
        let mut subset = 0;

        loop
        {
            let index = rook_magic_index(square, subset);
            attacks[square][index] = rook_slow(square, subset);

            subset = subset.wrapping_sub(mask) & mask;
            if subset == 0
            {
                break;
            }
        }

        square += 1;
    }

    attacks
}

const fn init_bishop_attacks() -> [[u64; BISHOP_TABLE_SIZE]; utils::NUM_SQUARES]
{
    let mut attacks = [[0; BISHOP_TABLE_SIZE]; utils::NUM_SQUARES];
    let mut square = 0;

    while square < utils::NUM_SQUARES
    {
        let mask = BISHOP_MASKS[square];
        let mut subset = 0;

        loop
        {
            let index = bishop_magic_index(square, subset);
            attacks[square][index] = bishop_slow(square, subset);

            subset = subset.wrapping_sub(mask) & mask;
            if subset == 0
            {
                break;
            }
        }

        square += 1;
    }

    attacks
}

const fn leaper_attacks(square: usize, offsets: [(i32, i32); 8]) -> u64
{
    let mut attacks = 0;
    let mut index = 0;
    let from_rank = rank(square) as i32;
    let from_file = file(square) as i32;

    while index < offsets.len()
    {
        let to_rank = from_rank + offsets[index].0;
        let to_file = from_file + offsets[index].1;

        if is_square(to_rank, to_file)
        {
            attacks |= bit((to_rank as usize * 8) + to_file as usize);
        }

        index += 1;
    }

    attacks
}

const fn bishop_slow(square: usize, occupied: u64) -> u64
{
    ray_attacks(square, occupied, 1, 1)
        | ray_attacks(square, occupied, 1, -1)
        | ray_attacks(square, occupied, -1, 1)
        | ray_attacks(square, occupied, -1, -1)
}

const fn rook_slow(square: usize, occupied: u64) -> u64
{
    ray_attacks(square, occupied, 1, 0)
        | ray_attacks(square, occupied, -1, 0)
        | ray_attacks(square, occupied, 0, 1)
        | ray_attacks(square, occupied, 0, -1)
}

const fn ray_attacks(
    square: usize,
    occupied: u64,
    rank_delta: i32,
    file_delta: i32,
) -> u64
{
    let mut attacks = 0;
    let mut to_rank = rank(square) as i32 + rank_delta;
    let mut to_file = file(square) as i32 + file_delta;

    while is_square(to_rank, to_file)
    {
        let to = (to_rank as usize * 8) + to_file as usize;
        attacks |= bit(to);

        if (occupied & bit(to)) != 0
        {
            break;
        }

        to_rank += rank_delta;
        to_file += file_delta;
    }

    attacks
}

const fn rook_index(square: usize, occupied: u64) -> usize
{
    rook_magic_index(square, occupied & ROOK_MASKS[square])
}

const fn bishop_index(square: usize, occupied: u64) -> usize
{
    bishop_magic_index(square, occupied & BISHOP_MASKS[square])
}

const fn rook_magic_index(square: usize, occupancy: u64) -> usize
{
    (occupancy.wrapping_mul(ROOK_MAGICS[square]) >> ROOK_SHIFTS[square]) as usize
}

const fn bishop_magic_index(square: usize, occupancy: u64) -> usize
{
    (occupancy.wrapping_mul(BISHOP_MAGICS[square]) >> BISHOP_SHIFTS[square]) as usize
}

const fn bit(square: usize) -> u64
{
    1u64 << square
}

const fn rank(square: usize) -> usize
{
    square / 8
}

const fn file(square: usize) -> usize
{
    square % 8
}

const fn is_square(rank: i32, file: i32) -> bool
{
    rank >= 0 && rank < 8 && file >= 0 && file < 8
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn assert_attack_eq(
        actual: u64,
        expected: u64,
        square: utils::Square,
        subset: u64,
    )
    {
        assert_eq!(actual, expected, "{:?} {:#018x}", square, subset);
    }

    #[test]
    fn leaper_attack_tables_cover_expected_squares()
    {
        assert_eq!(
            knight_attacks(utils::Square::D4),
            BitBoard::from_bits(
                bit(utils::Square::C2 as usize)
                    | bit(utils::Square::E2 as usize)
                    | bit(utils::Square::B3 as usize)
                    | bit(utils::Square::F3 as usize)
                    | bit(utils::Square::B5 as usize)
                    | bit(utils::Square::F5 as usize)
                    | bit(utils::Square::C6 as usize)
                    | bit(utils::Square::E6 as usize)
            )
        );
        assert_eq!(
            king_attacks(utils::Square::A1),
            BitBoard::from_bits(
                bit(utils::Square::A2 as usize)
                    | bit(utils::Square::B2 as usize)
                    | bit(utils::Square::B1 as usize)
            )
        );
        assert_eq!(
            pawn_attacks(utils::Square::E4, utils::Color::White),
            BitBoard::from_bits(
                bit(utils::Square::D5 as usize)
                    | bit(utils::Square::F5 as usize)
            )
        );
        assert_eq!(
            pawn_attacks(utils::Square::E4, utils::Color::Black),
            BitBoard::from_bits(
                bit(utils::Square::D3 as usize)
                    | bit(utils::Square::F3 as usize)
            )
        );
    }

    #[test]
    fn runtime_leaper_initializers_match_checked_in_tables()
    {
        let pawns = init_pawn_attacks();
        let knights = init_knight_attacks();
        let kings = init_king_attacks();

        assert_eq!(pawns, PAWN_ATTACKS);
        assert_eq!(knights, KNIGHT_ATTACKS);
        assert_eq!(kings, KING_ATTACKS);
    }

    #[test]
    fn runtime_slider_initializers_match_checked_in_tables()
    {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(||
            {
                let rooks = init_rook_attacks();
                let bishops = init_bishop_attacks();

                for square in 0..utils::NUM_SQUARES
                {
                    assert_eq!(rooks[square], ROOK_ATTACKS[square]);
                    assert_eq!(bishops[square], BISHOP_ATTACKS[square]);
                }
            })
            .unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn rook_magics_match_slow_attacks_for_every_relevant_occupancy()
    {
        for square in 0..utils::NUM_SQUARES
        {
            let mask = ROOK_MASKS[square];
            let mut subset = 0;

            loop
            {
                let actual = rook_attacks(
                    utils::SQUARES[square],
                    BitBoard::from_bits(subset),
                )
                    .bits();
                let expected = rook_slow(square, subset);
                assert_attack_eq(
                    actual,
                    expected,
                    utils::SQUARES[square],
                    subset,
                );

                subset = subset.wrapping_sub(mask) & mask;
                if subset == 0
                {
                    break;
                }
            }
        }
    }

    #[test]
    fn bishop_magics_match_slow_attacks_for_every_relevant_occupancy()
    {
        for square in 0..utils::NUM_SQUARES
        {
            let mask = BISHOP_MASKS[square];
            let mut subset = 0;

            loop
            {
                let actual = bishop_attacks(
                    utils::SQUARES[square],
                    BitBoard::from_bits(subset),
                )
                    .bits();
                let expected = bishop_slow(square, subset);
                assert_attack_eq(
                    actual,
                    expected,
                    utils::SQUARES[square],
                    subset,
                );

                subset = subset.wrapping_sub(mask) & mask;
                if subset == 0
                {
                    break;
                }
            }
        }
    }
}
