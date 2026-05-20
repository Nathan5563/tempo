use super::position;

pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECE_KINDS: usize = 12;
pub const NUM_COLORS: usize = 2;

// https://wismuth.com/chess/longest-game.html
pub const MAX_NUM_PLIES: usize = 17697;

// https://lichess.org/@/Tobs40/blog/why-a-reachable-position-can-have-at-most-218-playable-moves/a5xdxeqs
pub const MAX_NUM_MOVES: usize = 218;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Square
{
    #[default]
    A1 = 0,  A2 = 1,  A3 = 2,  A4 = 3,  A5 = 4,  A6 = 5,  A7 = 6,  A8 = 7,
    B1 = 8,  B2 = 9,  B3 = 10, B4 = 11, B5 = 12, B6 = 13, B7 = 14, B8 = 15,
    C1 = 16, C2 = 17, C3 = 18, C4 = 19, C5 = 20, C6 = 21, C7 = 22, C8 = 23,
    D1 = 24, D2 = 25, D3 = 26, D4 = 27, D5 = 28, D6 = 29, D7 = 30, D8 = 31,
    E1 = 32, E2 = 33, E3 = 34, E4 = 35, E5 = 36, E6 = 37, E7 = 38, E8 = 39,
    F1 = 40, F2 = 41, F3 = 42, F4 = 43, F5 = 44, F6 = 45, F7 = 46, F8 = 47,
    G1 = 48, G2 = 49, G3 = 50, G4 = 51, G5 = 52, G6 = 53, G7 = 54, G8 = 55,
    H1 = 56, H2 = 57, H3 = 58, H4 = 59, H5 = 60, H6 = 61, H7 = 62, H8 = 63,
}

pub const SQUARES: [Square; NUM_SQUARES] = [
    Square::A1, Square::A2, Square::A3, Square::A4, Square::A5, Square::A6, Square::A7, Square::A8,
    Square::B1, Square::B2, Square::B3, Square::B4, Square::B5, Square::B6, Square::B7, Square::B8,
    Square::C1, Square::C2, Square::C3, Square::C4, Square::C5, Square::C6, Square::C7, Square::C8,
    Square::D1, Square::D2, Square::D3, Square::D4, Square::D5, Square::D6, Square::D7, Square::D8,
    Square::E1, Square::E2, Square::E3, Square::E4, Square::E5, Square::E6, Square::E7, Square::E8,
    Square::F1, Square::F2, Square::F3, Square::F4, Square::F5, Square::F6, Square::F7, Square::F8,
    Square::G1, Square::G2, Square::G3, Square::G4, Square::G5, Square::G6, Square::G7, Square::G8,
    Square::H1, Square::H2, Square::H3, Square::H4, Square::H5, Square::H6, Square::H7, Square::H8,
];

#[derive(Debug, Clone, Copy, Default)]
pub enum Color
{
    #[default] White,
    Black
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PieceKind
{
    #[default] Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Piece
{
    color: Color,
    kind: PieceKind,
}

#[derive(Debug)]
pub struct Mailbox([Option<Piece>; NUM_SQUARES]);

impl Default for Mailbox
{
    fn default() -> Self
    {
        Self([None; NUM_SQUARES])
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CastlingRights(u8);
