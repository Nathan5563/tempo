use super::zobrist::{ZobristRandoms, ZobristType};
use super::{Position, State, Undo};

use crate::engine::utils::{
    Color,
    Piece,
    PieceKind,
    SQUARES,
    Square,
    enpassant_target,
};

const SRC_MASK: u16 = 0b111111_000000_0000;
const DEST_MASK: u16 = 0b000000_111111_0000;
const KIND_MASK: u16 = 0b000000_000000_1111;

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
    pub fn new(from: Square, to: Square, kind: MoveKind) -> Self
    {
        Self(
            ((from as u16) << (DEST_BITS + KIND_BITS))
            | ((to as u16) << KIND_BITS)
            | (kind as u16)
        )
    }

    #[inline]
    pub fn from(&self) -> Square
    {
        let index = ((self.0 & SRC_MASK) >> (DEST_BITS + KIND_BITS)) as usize;
        SQUARES[index]
    }

    #[inline]
    pub fn to(&self) -> Square
    {
        let index = ((self.0 & DEST_MASK) >> KIND_BITS) as usize;
        SQUARES[index]
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

impl std::fmt::Display for Move
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write_square(f, self.from())?;
        write_square(f, self.to())?;

        if let Some(suffix) = promotion_suffix(self.kind())
        {
            write!(f, "{}", suffix)?;
        }

        Ok(())
    }
}

fn write_square(
    f: &mut std::fmt::Formatter<'_>,
    square: Square,
) -> std::fmt::Result
{
    let index = square as u8;
    let file = char::from(b'a' + index % 8);
    let rank = char::from(b'1' + index / 8);

    write!(f, "{}{}", file, rank)
}

fn promotion_suffix(kind: MoveKind) -> Option<char>
{
    match kind
    {
        MoveKind::PromoteKnight | MoveKind::PromoteKnightCapture => Some('n'),
        MoveKind::PromoteBishop | MoveKind::PromoteBishopCapture => Some('b'),
        MoveKind::PromoteRook | MoveKind::PromoteRookCapture => Some('r'),
        MoveKind::PromoteQueen | MoveKind::PromoteQueenCapture => Some('q'),
        _ => None,
    }
}

pub fn make(pos: &mut Position, mov: Move)
{
    let old_state = pos.state;

    let from = mov.from();
    let to = mov.to();
    let kind = mov.kind();

    let moved = pos.board.mailbox[from].unwrap();
    let captured = if kind == MoveKind::EnPassant
    {
        Some(Piece {
            color: old_state.active.opposite(),
            kind: PieceKind::Pawn,
        })
    }
    else
    {
        pos.board.mailbox[to]
    };

    update_board(pos, from, to, kind, old_state.active, moved, captured);
    pos.state =
        update_state(&pos.zobrists, old_state, from, to, kind, moved, captured);
    pos.history.push(Undo { state: old_state, mov: mov, captured: captured });
}

pub fn unmake(pos: &mut Position)
{
    let undo = pos.history.pop();
    let old_state = undo.state;
    let mov = undo.mov;

    let from = mov.from();
    let to = mov.to();
    let kind = mov.kind();

    let placed = pos.board.mailbox[to].unwrap();
    let moved = Piece {
        color: placed.color,
        kind: if mov.is_promotion()
        {
            PieceKind::Pawn
        }
        else
        {
            placed.kind
        },
    };

    restore_board(pos, from, to, kind, old_state.active, moved, placed, undo.captured);
    pos.state = old_state;
}

fn update_board(
    pos: &mut Position,
    from: Square,
    to: Square,
    kind: MoveKind,
    active: Color,
    moved: Piece,
    captured: Option<Piece>,
)
{
    // remove moved piece
    pos.board.clear_piece(from, moved);

    // remove captured piece, if any
    if let Some(piece) = captured
    {
        let capture_square = if kind == MoveKind::EnPassant
        {
            enpassant_target(active, to)
        }
        else
        {
            to
        };
        pos.board.clear_piece(capture_square, piece);
    }

    // set placed piece
    pos.board.set_piece(to, placed_piece(moved, kind));

    // move rook if castling
    if let Some((rook_from, rook_to)) = castling_rook_squares(active, kind)
    {
        pos.board.move_piece(
            rook_from,
            rook_to,
            Piece {
                color: active,
                kind: PieceKind::Rook,
            },
        );
    }
}

fn restore_board(
    pos: &mut Position,
    from: Square,
    to: Square,
    kind: MoveKind,
    active: Color,
    moved: Piece,
    placed: Piece,
    captured: Option<Piece>,
)
{
    // move rook back if castling
    if let Some((rook_from, rook_to)) = castling_rook_squares(active, kind)
    {
        pos.board.move_piece(
            rook_to,
            rook_from,
            Piece {
                color: active,
                kind: PieceKind::Rook,
            },
        );
    }

    // remove placed piece
    pos.board.clear_piece(to, placed);

    // restore moved piece
    pos.board.set_piece(from, moved);

    // restore captured piece, if any
    if let Some(piece) = captured
    {
        let capture_square = if kind == MoveKind::EnPassant
        {
            enpassant_target(active, to)
        }
        else
        {
            to
        };
        pos.board.set_piece(capture_square, piece);
    }
}

fn update_state(
    zobrists: &ZobristRandoms,
    old_state: State,
    from: Square,
    to: Square,
    kind: MoveKind,
    moved: Piece,
    captured: Option<Piece>,
) -> State
{
    let mut new_state = old_state;

    // flip active side
    new_state.active = old_state.active.opposite();

    // set en passant square, if any
    new_state.enpassant = if kind == MoveKind::DoublePawnPush
    {
        Some(enpassant_target(old_state.active, to))
    }
    else
    {
        None
    };

    // update castling rights
    if moved.kind == PieceKind::King
    {
        match moved.color
        {
            Color::White => new_state.castling.clear_white(),
            Color::Black => new_state.castling.clear_black(),
        }
    }
    else if moved.kind == PieceKind::Rook
    {
        new_state.castling.clear_rook(from);
    }
    if let Some(piece) = captured
        && piece.kind == PieceKind::Rook
    {
        new_state.castling.clear_rook(to);
    }

    // update halfmove clock
    if captured.is_some() || moved.kind == PieceKind::Pawn
    {
        new_state.halfmoves = 0;
    }
    else
    {
        new_state.halfmoves += 1;
    }

    // update fullmove number
    new_state.fullmoves += if old_state.active == Color::Black
    {
        1
    }
    else
    {
        0
    };

    // update zobrist key
    new_state.key = update_key(
        zobrists, old_state, new_state, from, to, kind, moved, captured,
    );

    new_state
}

fn update_key(
    zobrists: &ZobristRandoms,
    old_state: State,
    new_state: State,
    from: Square,
    to: Square,
    kind: MoveKind,
    moved: Piece,
    captured: Option<Piece>,
) -> ZobristType
{
    let mut key = old_state.key;

    key ^= zobrists.active(old_state.active);
    key ^= zobrists.active(new_state.active);

    key ^= zobrists.castling(old_state.castling);
    key ^= zobrists.castling(new_state.castling);

    key ^= zobrists.enpassant(old_state.enpassant);
    key ^= zobrists.enpassant(new_state.enpassant);

    key ^= zobrists.piece(moved, from);

    if let Some(piece) = captured
    {
        let capture_square = if kind == MoveKind::EnPassant
        {
            enpassant_target(old_state.active, to)
        }
        else
        {
            to
        };
        key ^= zobrists.piece(piece, capture_square);
    }

    key ^= zobrists.piece(placed_piece(moved, kind), to);

    if let Some((rook_from, rook_to)) =
        castling_rook_squares(old_state.active, kind)
    {
        let rook = Piece {
            color: old_state.active,
            kind: PieceKind::Rook,
        };
        key ^= zobrists.piece(rook, rook_from);
        key ^= zobrists.piece(rook, rook_to);
    }

    key
}

fn castling_rook_squares(
    active: Color,
    kind: MoveKind,
) -> Option<(Square, Square)>
{
    match (active, kind)
    {
        (Color::White, MoveKind::KingCastle) =>
        {
            Some((Square::H1, Square::F1))
        }
        (Color::White, MoveKind::QueenCastle) =>
        {
            Some((Square::A1, Square::D1))
        }
        (Color::Black, MoveKind::KingCastle) =>
        {
            Some((Square::H8, Square::F8))
        }
        (Color::Black, MoveKind::QueenCastle) =>
        {
            Some((Square::A8, Square::D8))
        }
        _ => None,
    }
}

fn placed_piece(moved: Piece, kind: MoveKind) -> Piece
{
    Piece {
        color: moved.color,
        kind: match kind
        {
            MoveKind::PromoteKnight | MoveKind::PromoteKnightCapture =>
            {
                PieceKind::Knight
            }
            MoveKind::PromoteBishop | MoveKind::PromoteBishopCapture =>
            {
                PieceKind::Bishop
            }
            MoveKind::PromoteRook | MoveKind::PromoteRookCapture =>
            {
                PieceKind::Rook
            }
            MoveKind::PromoteQueen | MoveKind::PromoteQueenCapture =>
            {
                PieceKind::Queen
            }
            _ => moved.kind,
        },
    }
}

#[cfg(test)]
mod tests
{
    use crate::engine::position::bitboard::BitBoard;
    use crate::engine::utils::{
        CastlingRights,
        NUM_COLORS,
        NUM_PIECE_KINDS,
        NUM_SQUARES,
    };

    use super::*;

    const ALL_RIGHTS: u8 = CastlingRights::WHITE_KINGSIDE
        | CastlingRights::WHITE_QUEENSIDE
        | CastlingRights::BLACK_KINGSIDE
        | CastlingRights::BLACK_QUEENSIDE;

    fn piece(color: Color, kind: PieceKind) -> Piece
    {
        Piece { color, kind }
    }

    fn pos_with_active(active: Color) -> Position
    {
        let mut pos = Position::default();
        pos.state.active = active;
        pos.state.castling = CastlingRights::from_bits(ALL_RIGHTS);
        pos.state.halfmoves = 7;
        pos.state.fullmoves = 12;
        pos
    }

    fn refresh_key(pos: &mut Position)
    {
        pos.state.key = pos.zobrists.hash(pos);
    }

    fn assert_key_is_fresh(pos: &Position)
    {
        assert_eq!(pos.state.key, pos.zobrists.hash(pos));
    }

    fn mailbox_snapshot(
        pos: &Position,
    ) -> [Option<Piece>; NUM_SQUARES]
    {
        let mut snapshot = [None; NUM_SQUARES];
        for square in SQUARES
        {
            snapshot[square as usize] = pos.board.mailbox[square];
        }
        snapshot
    }

    fn assert_mailbox_eq(
        pos: &Position,
        expected: [Option<Piece>; NUM_SQUARES],
    )
    {
        for square in SQUARES
        {
            assert_eq!(
                pos.board.mailbox[square], expected[square as usize],
                "{:?}",
                square
            );
        }
    }

    fn assert_state_eq(left: State, right: State)
    {
        assert_eq!(left.key, right.key);
        assert_eq!(left.active, right.active);
        assert_eq!(left.castling.bits(), right.castling.bits());
        assert_eq!(left.enpassant, right.enpassant);
        assert_eq!(left.halfmoves, right.halfmoves);
        assert_eq!(left.fullmoves, right.fullmoves);
    }

    fn assert_board_consistent(pos: &Position)
    {
        let mut pieces = [BitBoard::default();
            NUM_PIECE_KINDS];
        let mut colors =
            [BitBoard::default(); NUM_COLORS];
        let mut kings = [Square::A1; NUM_COLORS];

        for square in SQUARES
        {
            if let Some(piece) = pos.board.mailbox[square]
            {
                pieces[piece.kind as usize].set(square);
                colors[piece.color as usize].set(square);

                if piece.kind == PieceKind::King
                {
                    kings[piece.color as usize] = square;
                }
            }
        }

        assert_eq!(pos.board.pieces, pieces);
        assert_eq!(pos.board.colors, colors);
        assert_eq!(pos.board.kings, kings);
    }

    fn assert_unmake_restores(pos: &mut Position, mov: Move)
    {
        let old_mailbox = mailbox_snapshot(pos);
        let old_state = pos.state;

        make(pos, mov);
        assert_board_consistent(pos);
        assert_key_is_fresh(pos);

        unmake(pos);
        assert_mailbox_eq(pos, old_mailbox);
        assert_state_eq(pos.state, old_state);
        assert_board_consistent(pos);
        assert_key_is_fresh(pos);
    }

    #[test]
    fn move_encoding_round_trips_all_kinds_and_flags()
    {
        let cases = [
            (MoveKind::Quiet, false, false),
            (MoveKind::DoublePawnPush, false, false),
            (MoveKind::KingCastle, false, false),
            (MoveKind::QueenCastle, false, false),
            (MoveKind::Capture, true, false),
            (MoveKind::EnPassant, true, false),
            (MoveKind::PromoteKnight, false, true),
            (MoveKind::PromoteBishop, false, true),
            (MoveKind::PromoteRook, false, true),
            (MoveKind::PromoteQueen, false, true),
            (MoveKind::PromoteKnightCapture, true, true),
            (MoveKind::PromoteBishopCapture, true, true),
            (MoveKind::PromoteRookCapture, true, true),
            (MoveKind::PromoteQueenCapture, true, true),
        ];

        for (index, (kind, is_capture, is_promotion)) in
            cases.into_iter().enumerate()
        {
            let from = SQUARES[index];
            let to = SQUARES[NUM_SQUARES - 1 - index];
            let mov = Move::new(from, to, kind);

            assert_eq!(mov.from(), from);
            assert_eq!(mov.to(), to);
            assert_eq!(mov.kind(), kind);
            assert_eq!(mov.is_capture(), is_capture);
            assert_eq!(mov.is_promotion(), is_promotion);
        }
    }

    #[test]
    fn move_display_uses_uci_long_algebraic()
    {
        let cases = [
            (
                Move::new(Square::E2, Square::E4, MoveKind::Quiet),
                "e2e4",
            ),
            (
                Move::new(
                    Square::E1,
                    Square::G1,
                    MoveKind::KingCastle,
                ),
                "e1g1",
            ),
            (
                Move::new(
                    Square::A7,
                    Square::A8,
                    MoveKind::PromoteQueen,
                ),
                "a7a8q",
            ),
            (
                Move::new(
                    Square::G7,
                    Square::H8,
                    MoveKind::PromoteKnightCapture,
                ),
                "g7h8n",
            ),
        ];

        for (mov, expected) in cases
        {
            assert_eq!(mov.to_string(), expected);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid move kind: 6")]
    fn move_kind_rejects_unused_encoding()
    {
        Move(6).kind();
    }

    #[test]
    fn placed_piece_maps_promotion_kinds()
    {
        let pawn = piece(Color::White, PieceKind::Pawn);

        assert_eq!(placed_piece(pawn, MoveKind::Quiet), pawn);
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteKnight).kind,
            PieceKind::Knight
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteKnightCapture).kind,
            PieceKind::Knight
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteBishop).kind,
            PieceKind::Bishop
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteBishopCapture).kind,
            PieceKind::Bishop
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteRook).kind,
            PieceKind::Rook
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteRookCapture).kind,
            PieceKind::Rook
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteQueen).kind,
            PieceKind::Queen
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteQueenCapture).kind,
            PieceKind::Queen
        );
    }

    #[test]
    fn quiet_move_updates_board_state_history_and_key()
    {
        let mut pos = pos_with_active(Color::White);
        let knight = piece(Color::White, PieceKind::Knight);
        let mov =
            Move::new(Square::G1, Square::F3, MoveKind::Quiet);

        pos.board.set_piece(Square::G1, knight);
        pos.state.enpassant = Some(Square::A3);
        refresh_key(&mut pos);
        let old_state = pos.state;

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[Square::G1], None);
        assert_eq!(pos.board.mailbox[Square::F3], Some(knight));
        assert_eq!(pos.state.active, Color::Black);
        assert_eq!(pos.state.castling.bits(), ALL_RIGHTS);
        assert_eq!(pos.state.enpassant, None);
        assert_eq!(pos.state.halfmoves, old_state.halfmoves + 1);
        assert_eq!(pos.state.fullmoves, old_state.fullmoves);
        assert_state_eq(pos.history.arr[0].state, old_state);
        assert_eq!(pos.history.arr[0].mov, mov);
        assert_eq!(pos.history.arr[0].captured, None);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[Square::G1], Some(knight));
        assert_eq!(pos.board.mailbox[Square::F3], None);
        assert_state_eq(pos.state, old_state);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn double_pawn_push_sets_enpassant_and_resets_halfmoves_for_both_colors()
    {
        let cases = [
            (
                Color::White,
                Square::E2,
                Square::E4,
                Square::E3,
                12,
            ),
            (
                Color::Black,
                Square::D7,
                Square::D5,
                Square::D6,
                13,
            ),
        ];

        for (active, from, to, enpassant, expected_fullmoves) in cases
        {
            let mut pos = pos_with_active(active);
            let pawn = piece(active, PieceKind::Pawn);
            let mov = Move::new(from, to, MoveKind::DoublePawnPush);

            pos.board.set_piece(from, pawn);
            refresh_key(&mut pos);

            make(&mut pos, mov);

            assert_eq!(pos.board.mailbox[from], None);
            assert_eq!(pos.board.mailbox[to], Some(pawn));
            assert_eq!(pos.state.active, active.opposite());
            assert_eq!(pos.state.enpassant, Some(enpassant));
            assert_eq!(pos.state.halfmoves, 0);
            assert_eq!(pos.state.fullmoves, expected_fullmoves);
            assert_board_consistent(&pos);
            assert_key_is_fresh(&pos);

            unmake(&mut pos);

            assert_eq!(pos.board.mailbox[from], Some(pawn));
            assert_eq!(pos.board.mailbox[to], None);
            assert_board_consistent(&pos);
            assert_key_is_fresh(&pos);
        }
    }

    #[test]
    fn capture_removes_piece_resets_halfmoves_and_updates_castling_rights()
    {
        let mut pos = pos_with_active(Color::White);
        let bishop = piece(Color::White, PieceKind::Bishop);
        let rook = piece(Color::Black, PieceKind::Rook);
        let mov =
            Move::new(Square::G7, Square::H8, MoveKind::Capture);

        pos.board.set_piece(Square::G7, bishop);
        pos.board.set_piece(Square::H8, rook);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[Square::G7], None);
        assert_eq!(pos.board.mailbox[Square::H8], Some(bishop));
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(
            pos.state.castling.bits(),
            ALL_RIGHTS & !CastlingRights::BLACK_KINGSIDE
        );
        assert_eq!(pos.history.arr[0].captured, Some(rook));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[Square::G7], Some(bishop));
        assert_eq!(pos.board.mailbox[Square::H8], Some(rook));
        assert_eq!(pos.state.castling.bits(), ALL_RIGHTS);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn enpassant_captures_on_target_square_and_restores_cleanly()
    {
        let mut pos = pos_with_active(Color::White);
        let white_pawn = piece(Color::White, PieceKind::Pawn);
        let black_pawn = piece(Color::Black, PieceKind::Pawn);
        let mov = Move::new(
            Square::E5,
            Square::D6,
            MoveKind::EnPassant,
        );

        pos.board.set_piece(Square::E5, white_pawn);
        pos.board.set_piece(Square::D5, black_pawn);
        pos.state.enpassant = Some(Square::D6);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[Square::E5], None);
        assert_eq!(pos.board.mailbox[Square::D5], None);
        assert_eq!(pos.board.mailbox[Square::D6], Some(white_pawn));
        assert_eq!(pos.state.enpassant, None);
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.history.arr[0].captured, Some(black_pawn));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[Square::E5], Some(white_pawn));
        assert_eq!(pos.board.mailbox[Square::D5], Some(black_pawn));
        assert_eq!(pos.board.mailbox[Square::D6], None);
        assert_eq!(pos.state.enpassant, Some(Square::D6));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn promotions_place_promoted_piece_and_unmake_restores_pawn()
    {
        let mut pos = pos_with_active(Color::White);
        let pawn = piece(Color::White, PieceKind::Pawn);
        let queen = piece(Color::White, PieceKind::Queen);
        let mov = Move::new(
            Square::A7,
            Square::A8,
            MoveKind::PromoteQueen,
        );

        pos.board.set_piece(Square::A7, pawn);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[Square::A7], None);
        assert_eq!(pos.board.mailbox[Square::A8], Some(queen));
        assert_eq!(pos.state.halfmoves, 0);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[Square::A7], Some(pawn));
        assert_eq!(pos.board.mailbox[Square::A8], None);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn promotion_captures_restore_captured_piece_on_unmake()
    {
        let mut pos = pos_with_active(Color::White);
        let pawn = piece(Color::White, PieceKind::Pawn);
        let knight = piece(Color::White, PieceKind::Knight);
        let rook = piece(Color::Black, PieceKind::Rook);
        let mov = Move::new(
            Square::G7,
            Square::H8,
            MoveKind::PromoteKnightCapture,
        );

        pos.board.set_piece(Square::G7, pawn);
        pos.board.set_piece(Square::H8, rook);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[Square::G7], None);
        assert_eq!(pos.board.mailbox[Square::H8], Some(knight));
        assert_eq!(pos.history.arr[0].captured, Some(rook));
        assert_eq!(
            pos.state.castling.bits(),
            ALL_RIGHTS & !CastlingRights::BLACK_KINGSIDE
        );
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[Square::G7], Some(pawn));
        assert_eq!(pos.board.mailbox[Square::H8], Some(rook));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn rook_moves_clear_only_matching_castling_rights()
    {
        let cases = [
            (
                Color::White,
                Square::A1,
                Square::A2,
                ALL_RIGHTS & !CastlingRights::WHITE_QUEENSIDE,
            ),
            (
                Color::White,
                Square::H1,
                Square::H2,
                ALL_RIGHTS & !CastlingRights::WHITE_KINGSIDE,
            ),
            (
                Color::Black,
                Square::A8,
                Square::A7,
                ALL_RIGHTS & !CastlingRights::BLACK_QUEENSIDE,
            ),
            (
                Color::Black,
                Square::H8,
                Square::H7,
                ALL_RIGHTS & !CastlingRights::BLACK_KINGSIDE,
            ),
            (
                Color::White,
                Square::B1,
                Square::B2,
                ALL_RIGHTS,
            ),
        ];

        for (active, from, to, expected_rights) in cases
        {
            let mut pos = pos_with_active(active);
            let rook = piece(active, PieceKind::Rook);
            let mov = Move::new(from, to, MoveKind::Quiet);

            pos.board.set_piece(from, rook);
            refresh_key(&mut pos);

            make(&mut pos, mov);

            assert_eq!(pos.board.mailbox[from], None);
            assert_eq!(pos.board.mailbox[to], Some(rook));
            assert_eq!(pos.state.castling.bits(), expected_rights);
            assert_board_consistent(&pos);
            assert_key_is_fresh(&pos);

            unmake(&mut pos);

            assert_eq!(pos.board.mailbox[from], Some(rook));
            assert_eq!(pos.board.mailbox[to], None);
            assert_eq!(pos.state.castling.bits(), ALL_RIGHTS);
            assert_board_consistent(&pos);
            assert_key_is_fresh(&pos);
        }
    }

    #[test]
    fn castling_moves_king_and_rook_for_all_sides_and_directions()
    {
        let cases = [
            (
                Color::White,
                MoveKind::KingCastle,
                Square::E1,
                Square::G1,
                Square::H1,
                Square::F1,
                ALL_RIGHTS
                    & !(CastlingRights::WHITE_KINGSIDE
                        | CastlingRights::WHITE_QUEENSIDE),
            ),
            (
                Color::White,
                MoveKind::QueenCastle,
                Square::E1,
                Square::C1,
                Square::A1,
                Square::D1,
                ALL_RIGHTS
                    & !(CastlingRights::WHITE_KINGSIDE
                        | CastlingRights::WHITE_QUEENSIDE),
            ),
            (
                Color::Black,
                MoveKind::KingCastle,
                Square::E8,
                Square::G8,
                Square::H8,
                Square::F8,
                ALL_RIGHTS
                    & !(CastlingRights::BLACK_KINGSIDE
                        | CastlingRights::BLACK_QUEENSIDE),
            ),
            (
                Color::Black,
                MoveKind::QueenCastle,
                Square::E8,
                Square::C8,
                Square::A8,
                Square::D8,
                ALL_RIGHTS
                    & !(CastlingRights::BLACK_KINGSIDE
                        | CastlingRights::BLACK_QUEENSIDE),
            ),
        ];

        for (
            active,
            kind,
            king_from,
            king_to,
            rook_from,
            rook_to,
            expected_rights,
        ) in cases
        {
            let mut pos = pos_with_active(active);
            let king = piece(active, PieceKind::King);
            let rook = piece(active, PieceKind::Rook);
            let mov = Move::new(king_from, king_to, kind);

            pos.board.set_piece(king_from, king);
            pos.board.set_piece(rook_from, rook);
            refresh_key(&mut pos);

            make(&mut pos, mov);

            assert_eq!(pos.board.mailbox[king_from], None);
            assert_eq!(pos.board.mailbox[rook_from], None);
            assert_eq!(pos.board.mailbox[king_to], Some(king));
            assert_eq!(pos.board.mailbox[rook_to], Some(rook));
            assert_eq!(pos.state.castling.bits(), expected_rights);
            assert_board_consistent(&pos);
            assert_key_is_fresh(&pos);

            unmake(&mut pos);

            assert_eq!(pos.board.mailbox[king_from], Some(king));
            assert_eq!(pos.board.mailbox[rook_from], Some(rook));
            assert_eq!(pos.board.mailbox[king_to], None);
            assert_eq!(pos.board.mailbox[rook_to], None);
            assert_eq!(pos.state.castling.bits(), ALL_RIGHTS);
            assert_board_consistent(&pos);
            assert_key_is_fresh(&pos);
        }
    }

    #[test]
    fn make_unmake_round_trips_a_representative_quiet_move()
    {
        let mut pos = pos_with_active(Color::White);
        let knight = piece(Color::White, PieceKind::Knight);
        let mov =
            Move::new(Square::B1, Square::C3, MoveKind::Quiet);

        pos.board.set_piece(Square::B1, knight);
        refresh_key(&mut pos);

        assert_unmake_restores(&mut pos, mov);
    }
}
