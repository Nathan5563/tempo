use super::position;

pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECE_TYPES: usize = 12;
pub const NUM_COLORS: usize = 2;

// https://wismuth.com/chess/longest-game.html
pub const MAX_NUM_PLIES: usize = 17697;

// https://lichess.org/@/Tobs40/blog/why-a-reachable-position-can-have-at-most-218-playable-moves/a5xdxeqs
pub const MAX_NUM_MOVES: usize = 218;

#[derive(Debug, Clone, Copy, Default)]
pub struct Square(u8);

#[derive(Debug, Clone, Copy, Default)]
pub enum Color
{
    #[default] White,
    Black
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PieceType
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
    piecetype: PieceType,
}

#[derive(Debug)]
pub struct Mailbox([Option<Piece>; NUM_SQUARES]);

impl Default for Mailbox
{
    fn default() -> Self
    {
        Mailbox([None; NUM_SQUARES])
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CastlingRights(u8);
