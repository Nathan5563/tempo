use super::Position;

use crate::engine::prng::Xoshiro256StarStar;
use crate::engine::utils::{
    CastlingRights,
    Color,
    NUM_COLORS,
    NUM_PIECE_KINDS,
    NUM_SQUARES,
    Piece,
    SQUARES,
    Square,
};

pub type ZobristType = u64;
const NUM_CASTLING_STATES: usize = 16;
const NUM_ENPASSANT_STATES: usize = 17;
const PRNG_SEED: u64 = 42;

#[derive(Debug, Clone)]
pub struct ZobristRandoms
{
    active: [ZobristType; NUM_COLORS],
    pieces: [[[ZobristType; NUM_SQUARES]; NUM_PIECE_KINDS];
        NUM_COLORS],
    castling: [ZobristType; NUM_CASTLING_STATES],
    enpassant: [ZobristType; NUM_ENPASSANT_STATES],
}

impl Default for ZobristRandoms
{
    fn default() -> Self
    {
        let mut rng = Xoshiro256StarStar::from_seed(PRNG_SEED);
        let active = [(); NUM_COLORS].map(|_| rng.next_u64());
        let pieces = [(); NUM_COLORS].map(|_| {
            [(); NUM_PIECE_KINDS]
                .map(|_| [(); NUM_SQUARES].map(|_| rng.next_u64()))
        });
        let castling = [(); NUM_CASTLING_STATES].map(|_| rng.next_u64());
        let enpassant = [(); NUM_ENPASSANT_STATES].map(|_| rng.next_u64());
        Self { active, pieces, castling, enpassant }
    }
}

impl ZobristRandoms
{
    pub fn hash(&self, pos: &Position) -> ZobristType
    {
        let mut key = 0;

        for square in SQUARES
        {
            if let Some(piece) = pos.board.mailbox[square]
            {
                key ^= self.piece(piece, square);
            }
        }

        key ^= self.active(pos.state.active);
        key ^= self.castling(pos.state.castling);
        key ^= self.enpassant(pos.state.enpassant);

        key
    }

    pub fn piece(&self, piece: Piece, sq: Square) -> ZobristType
    {
        self.pieces[piece.color as usize][piece.kind as usize][sq as usize]
    }

    pub fn active(&self, color: Color) -> ZobristType
    {
        self.active[color as usize]
    }

    pub fn castling(&self, rights: CastlingRights) -> ZobristType
    {
        self.castling[rights.bits() as usize]
    }

    pub fn enpassant(&self, square: Option<Square>) -> ZobristType
    {
        self.enpassant[enpassant_index(square)]
    }
}

fn enpassant_index(square: Option<Square>) -> usize
{
    match square
    {
        None => 0,
        Some(square) =>
        {
            let index = square as usize;
            if index >= Square::A3 as usize
                && index <= Square::H3 as usize
            {
                1 + index - Square::A3 as usize
            }
            else if index >= Square::A6 as usize
                && index <= Square::H6 as usize
            {
                9 + index - Square::A6 as usize
            }
            else
            {
                unreachable!("Invalid en passant square: {:?}", square)
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use crate::engine::utils::PieceKind;

    use super::*;

    fn piece(color: Color, kind: PieceKind) -> Piece
    {
        Piece { color, kind }
    }

    fn indexed_randoms() -> ZobristRandoms
    {
        let mut randoms = ZobristRandoms {
            active: [101, 202],
            pieces: [[[0; NUM_SQUARES]; NUM_PIECE_KINDS];
                NUM_COLORS],
            castling: [0; NUM_CASTLING_STATES],
            enpassant: [0; NUM_ENPASSANT_STATES],
        };

        for color in 0..NUM_COLORS
        {
            for kind in 0..NUM_PIECE_KINDS
            {
                for square in 0..NUM_SQUARES
                {
                    randoms.pieces[color][kind][square] = 1_000
                        + (color as u64 * 10_000)
                        + (kind as u64 * 1_000)
                        + square as u64;
                }
            }
        }
        for index in 0..NUM_CASTLING_STATES
        {
            randoms.castling[index] = 50_000 + index as u64;
        }
        for index in 0..NUM_ENPASSANT_STATES
        {
            randoms.enpassant[index] = 60_000 + index as u64;
        }

        randoms
    }

    #[test]
    fn default_randoms_are_deterministic_and_filled_in_prng_order()
    {
        let randoms = ZobristRandoms::default();
        let mut rng = Xoshiro256StarStar::from_seed(PRNG_SEED);

        for color in 0..NUM_COLORS
        {
            assert_eq!(randoms.active[color], rng.next_u64());
        }
        for color in 0..NUM_COLORS
        {
            for kind in 0..NUM_PIECE_KINDS
            {
                for square in 0..NUM_SQUARES
                {
                    assert_eq!(
                        randoms.pieces[color][kind][square],
                        rng.next_u64()
                    );
                }
            }
        }
        for index in 0..NUM_CASTLING_STATES
        {
            assert_eq!(randoms.castling[index], rng.next_u64());
        }
        for index in 0..NUM_ENPASSANT_STATES
        {
            assert_eq!(randoms.enpassant[index], rng.next_u64());
        }

        let same_seed = ZobristRandoms::default();
        assert_eq!(same_seed.active, randoms.active);
        assert_eq!(same_seed.pieces, randoms.pieces);
        assert_eq!(same_seed.castling, randoms.castling);
        assert_eq!(same_seed.enpassant, randoms.enpassant);
    }

    #[test]
    fn accessors_index_their_backing_tables()
    {
        let randoms = indexed_randoms();
        let queen = piece(Color::Black, PieceKind::Queen);
        let rights = CastlingRights::from_bits(0b1011);

        assert_eq!(
            randoms.piece(queen, Square::H8),
            randoms.pieces[1][4][63]
        );
        assert_eq!(randoms.active(Color::Black), randoms.active[1]);
        assert_eq!(randoms.castling(rights), randoms.castling[0b1011]);
        assert_eq!(randoms.enpassant(None), randoms.enpassant[0]);
        assert_eq!(
            randoms.enpassant(Some(Square::A3)),
            randoms.enpassant[1]
        );
        assert_eq!(
            randoms.enpassant(Some(Square::H6)),
            randoms.enpassant[16]
        );
    }

    #[test]
    fn hash_xors_occupied_squares_and_state_components()
    {
        let randoms = indexed_randoms();
        let mut pos = Position::default();
        let knight = piece(Color::White, PieceKind::Knight);
        let queen = piece(Color::Black, PieceKind::Queen);

        pos.board.set_piece(Square::B1, knight);
        pos.board.set_piece(Square::H8, queen);
        pos.state.active = Color::Black;
        pos.state.castling = CastlingRights::from_bits(0b1011);
        pos.state.enpassant = Some(Square::D6);

        let expected = randoms.pieces[Color::White as usize]
            [PieceKind::Knight as usize][Square::B1 as usize]
            ^ randoms.pieces[Color::Black as usize]
                [PieceKind::Queen as usize][Square::H8 as usize]
            ^ randoms.active[Color::Black as usize]
            ^ randoms.castling[0b1011]
            ^ randoms.enpassant[enpassant_index(Some(Square::D6))];

        assert_eq!(randoms.hash(&pos), expected);
    }

    #[test]
    fn enpassant_index_maps_every_legal_state()
    {
        assert_eq!(enpassant_index(None), 0);

        for file in 0..8
        {
            let third_rank = SQUARES[Square::A3 as usize + file];
            let sixth_rank = SQUARES[Square::A6 as usize + file];

            assert_eq!(enpassant_index(Some(third_rank)), 1 + file);
            assert_eq!(enpassant_index(Some(sixth_rank)), 9 + file);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid en passant square")]
    fn enpassant_index_rejects_invalid_square()
    {
        enpassant_index(Some(Square::E4));
    }
}
