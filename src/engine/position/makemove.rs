use super::{Position, super::utils};

const SRC_MASK: u16 = 0b111111_000000_0000;
const DEST_MASK: u16 = 0b000000_111111_0000;
const KIND_MASK: u16 = 0b000000_000000_1111;

const SRC_BITS: u16 = 6;
const DEST_BITS: u16 = 6;
const KIND_BITS: u16 = 4;

const CAPTURE_FLAG: u16 = 0b0100;
const PROMOTION_FLAG: u16 = 0b1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveKind
{
    Quiet = 0b0000,
    DoublePawnPush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EnPassant = 0b0101,
    PromoteKnight = 0b1000,
    PromoteBishop = 0b1001,
    PromoteRook = 0b1010,
    PromoteQueen = 0b1011,
    PromoteKnightCapture = 0b1100,
    PromoteBishopCapture = 0b1101,
    PromoteRookCapture = 0b1110,
    PromoteQueenCapture = 0b1111,
}

// ssssss dddddd kkkk
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Move(u16);

impl Move
{
    #[inline]
    pub fn new(from: utils::Square, to: utils::Square, kind: MoveKind) -> Self
    {
        Self(
            ((from as u16) << (DEST_BITS + KIND_BITS)) 
            | ((to as u16) << KIND_BITS) 
            | (kind as u16)
        )
    }

    #[inline]
    pub fn from(&self) -> utils::Square
    {
        let index = ((self.0 & SRC_MASK) >> (DEST_BITS + KIND_BITS)) as usize;
        utils::SQUARES[index]
    }

    #[inline]
    pub fn to(&self) -> utils::Square
    {
        let index = ((self.0 & DEST_MASK) >> KIND_BITS) as usize;
        utils::SQUARES[index]
    }

    #[inline]
    pub fn kind(&self) -> MoveKind
    {
        let kind = (self.0 & KIND_MASK) as u8;
        match kind
        {
            0 => MoveKind::Quiet,
            1 => MoveKind::DoublePawnPush,
            2 => MoveKind::KingCastle,
            3 => MoveKind::QueenCastle,
            4 => MoveKind::Capture,
            5 => MoveKind::EnPassant,
            8 => MoveKind::PromoteKnight,
            9 => MoveKind::PromoteBishop,
            10 => MoveKind::PromoteRook,
            11 => MoveKind::PromoteQueen,
            12 => MoveKind::PromoteKnightCapture,
            13 => MoveKind::PromoteBishopCapture,
            14 => MoveKind::PromoteRookCapture,
            15 => MoveKind::PromoteQueenCapture,
            _ => unreachable!("Invalid move kind: {}", kind)
        }
    }

    #[inline]
    pub fn is_capture(&self) -> bool
    {
        (self.0 & CAPTURE_FLAG) != 0
    }

    #[inline]
    pub fn is_promotion(&self) -> bool
    {
        (self.0 & PROMOTION_FLAG) != 0
    }
}

// TODO: Implement make function
pub fn make(pos: &mut Position, mov: Move)
{
    let from = mov.from();
    let to = mov.to();
    let kind = mov.kind();

    // modify board (bitboards, mailbox)
    // modify state (active, enpassant, castling, halfmoves, fullmoves, new zobrist key)
    // modify history (push new state, captured piece if any)
}

// TODO: Implement unmake function
pub fn unmake(pos: &mut Position, mov: Move)
{
    // modify history (pop undo information)
    // modify state (reset from undo information)
    // modify board (bitboards, mailbox)
}
