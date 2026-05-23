use super::{Position, State, Undo, zobrist, super::utils};

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

pub fn make(pos: &mut Position, mov: Move)
{
    let old_state = pos.state;

    let from = mov.from();
    let to = mov.to();
    let kind = mov.kind();

    let moved = pos.board.mailbox[from].unwrap();
    let captured = if kind == MoveKind::EnPassant
    {
        Some(utils::Piece {
            color: old_state.active.opposite(),
            kind: utils::PieceKind::Pawn,
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
    let moved = utils::Piece {
        color: placed.color,
        kind: if mov.is_promotion()
        {
            utils::PieceKind::Pawn
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
    from: utils::Square,
    to: utils::Square,
    kind: MoveKind,
    active: utils::Color,
    moved: utils::Piece,
    captured: Option<utils::Piece>,
)
{
    // remove moved piece
    pos.board.clear_piece(from, moved);

    // remove captured piece, if any
    if let Some(piece) = captured
    {
        let capture_square = if kind == MoveKind::EnPassant
        {
            utils::enpassant_target(active, to)
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
            utils::Piece {
                color: active,
                kind: utils::PieceKind::Rook,
            },
        );
    }
}

fn restore_board(
    pos: &mut Position,
    from: utils::Square,
    to: utils::Square,
    kind: MoveKind,
    active: utils::Color,
    moved: utils::Piece,
    placed: utils::Piece,
    captured: Option<utils::Piece>,
)
{
    // move rook back if castling
    if let Some((rook_from, rook_to)) = castling_rook_squares(active, kind)
    {
        pos.board.move_piece(
            rook_to,
            rook_from,
            utils::Piece {
                color: active,
                kind: utils::PieceKind::Rook,
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
            utils::enpassant_target(active, to)
        }
        else
        {
            to
        };
        pos.board.set_piece(capture_square, piece);
    }
}

fn update_state(
    zobrists: &zobrist::ZobristRandoms,
    old_state: State,
    from: utils::Square,
    to: utils::Square,
    kind: MoveKind,
    moved: utils::Piece,
    captured: Option<utils::Piece>,
) -> State
{
    let mut new_state = old_state;

    // flip active side
    new_state.active = old_state.active.opposite();

    // set en passant square, if any
    new_state.enpassant = if kind == MoveKind::DoublePawnPush
    {
        Some(utils::enpassant_target(old_state.active, to))
    }
    else
    {
        None
    };

    // update castling rights
    if moved.kind == utils::PieceKind::King
    {
        match moved.color
        {
            utils::Color::White => new_state.castling.clear_white(),
            utils::Color::Black => new_state.castling.clear_black(),
        }
    }
    else if moved.kind == utils::PieceKind::Rook
    {
        new_state.castling.clear_rook(from);
    }
    if let Some(piece) = captured
        && piece.kind == utils::PieceKind::Rook
    {
        new_state.castling.clear_rook(to);
    }

    // update halfmove clock
    if captured.is_some() || moved.kind == utils::PieceKind::Pawn
    {
        new_state.halfmoves = 0;
    }
    else
    {
        new_state.halfmoves += 1;
    }

    // update fullmove number
    new_state.fullmoves += if old_state.active == utils::Color::Black
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
    zobrists: &zobrist::ZobristRandoms,
    old_state: State,
    new_state: State,
    from: utils::Square,
    to: utils::Square,
    kind: MoveKind,
    moved: utils::Piece,
    captured: Option<utils::Piece>,
) -> zobrist::ZobristType
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
            utils::enpassant_target(old_state.active, to)
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
        let rook = utils::Piece {
            color: old_state.active,
            kind: utils::PieceKind::Rook,
        };
        key ^= zobrists.piece(rook, rook_from);
        key ^= zobrists.piece(rook, rook_to);
    }

    key
}

fn castling_rook_squares(
    active: utils::Color,
    kind: MoveKind,
) -> Option<(utils::Square, utils::Square)>
{
    match (active, kind)
    {
        (utils::Color::White, MoveKind::KingCastle) =>
        {
            Some((utils::Square::H1, utils::Square::F1))
        }
        (utils::Color::White, MoveKind::QueenCastle) =>
        {
            Some((utils::Square::A1, utils::Square::D1))
        }
        (utils::Color::Black, MoveKind::KingCastle) =>
        {
            Some((utils::Square::H8, utils::Square::F8))
        }
        (utils::Color::Black, MoveKind::QueenCastle) =>
        {
            Some((utils::Square::A8, utils::Square::D8))
        }
        _ => None,
    }
}

fn placed_piece(moved: utils::Piece, kind: MoveKind) -> utils::Piece
{
    utils::Piece {
        color: moved.color,
        kind: match kind
        {
            MoveKind::PromoteKnight | MoveKind::PromoteKnightCapture =>
            {
                utils::PieceKind::Knight
            }
            MoveKind::PromoteBishop | MoveKind::PromoteBishopCapture =>
            {
                utils::PieceKind::Bishop
            }
            MoveKind::PromoteRook | MoveKind::PromoteRookCapture =>
            {
                utils::PieceKind::Rook
            }
            MoveKind::PromoteQueen | MoveKind::PromoteQueenCapture =>
            {
                utils::PieceKind::Queen
            }
            _ => moved.kind,
        },
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_RIGHTS: u8 = utils::CastlingRights::WHITE_KINGSIDE
        | utils::CastlingRights::WHITE_QUEENSIDE
        | utils::CastlingRights::BLACK_KINGSIDE
        | utils::CastlingRights::BLACK_QUEENSIDE;

    fn piece(color: utils::Color, kind: utils::PieceKind) -> utils::Piece
    {
        utils::Piece { color, kind }
    }

    fn pos_with_active(active: utils::Color) -> Position
    {
        let mut pos = Position::default();
        pos.state.active = active;
        pos.state.castling = utils::CastlingRights::from_bits(ALL_RIGHTS);
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
    ) -> [Option<utils::Piece>; utils::NUM_SQUARES]
    {
        let mut snapshot = [None; utils::NUM_SQUARES];
        for square in utils::SQUARES
        {
            snapshot[square as usize] = pos.board.mailbox[square];
        }
        snapshot
    }

    fn assert_mailbox_eq(
        pos: &Position,
        expected: [Option<utils::Piece>; utils::NUM_SQUARES],
    )
    {
        for square in utils::SQUARES
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
        let mut pieces = [super::super::bitboard::BitBoard::default();
            utils::NUM_PIECE_KINDS];
        let mut colors =
            [super::super::bitboard::BitBoard::default(); utils::NUM_COLORS];
        let mut kings = [utils::Square::A1; utils::NUM_COLORS];

        for square in utils::SQUARES
        {
            if let Some(piece) = pos.board.mailbox[square]
            {
                pieces[piece.kind as usize].set(square);
                colors[piece.color as usize].set(square);

                if piece.kind == utils::PieceKind::King
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
            let from = utils::SQUARES[index];
            let to = utils::SQUARES[utils::NUM_SQUARES - 1 - index];
            let mov = Move::new(from, to, kind);

            assert_eq!(mov.from(), from);
            assert_eq!(mov.to(), to);
            assert_eq!(mov.kind(), kind);
            assert_eq!(mov.is_capture(), is_capture);
            assert_eq!(mov.is_promotion(), is_promotion);
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
        let pawn = piece(utils::Color::White, utils::PieceKind::Pawn);

        assert_eq!(placed_piece(pawn, MoveKind::Quiet), pawn);
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteKnight).kind,
            utils::PieceKind::Knight
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteKnightCapture).kind,
            utils::PieceKind::Knight
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteBishop).kind,
            utils::PieceKind::Bishop
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteBishopCapture).kind,
            utils::PieceKind::Bishop
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteRook).kind,
            utils::PieceKind::Rook
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteRookCapture).kind,
            utils::PieceKind::Rook
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteQueen).kind,
            utils::PieceKind::Queen
        );
        assert_eq!(
            placed_piece(pawn, MoveKind::PromoteQueenCapture).kind,
            utils::PieceKind::Queen
        );
    }

    #[test]
    fn quiet_move_updates_board_state_history_and_key()
    {
        let mut pos = pos_with_active(utils::Color::White);
        let knight = piece(utils::Color::White, utils::PieceKind::Knight);
        let mov =
            Move::new(utils::Square::G1, utils::Square::F3, MoveKind::Quiet);

        pos.board.set_piece(utils::Square::G1, knight);
        pos.state.enpassant = Some(utils::Square::A3);
        refresh_key(&mut pos);
        let old_state = pos.state;

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[utils::Square::G1], None);
        assert_eq!(pos.board.mailbox[utils::Square::F3], Some(knight));
        assert_eq!(pos.state.active, utils::Color::Black);
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

        assert_eq!(pos.board.mailbox[utils::Square::G1], Some(knight));
        assert_eq!(pos.board.mailbox[utils::Square::F3], None);
        assert_state_eq(pos.state, old_state);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn double_pawn_push_sets_enpassant_and_resets_halfmoves_for_both_colors()
    {
        let cases = [
            (
                utils::Color::White,
                utils::Square::E2,
                utils::Square::E4,
                utils::Square::E3,
                12,
            ),
            (
                utils::Color::Black,
                utils::Square::D7,
                utils::Square::D5,
                utils::Square::D6,
                13,
            ),
        ];

        for (active, from, to, enpassant, expected_fullmoves) in cases
        {
            let mut pos = pos_with_active(active);
            let pawn = piece(active, utils::PieceKind::Pawn);
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
        let mut pos = pos_with_active(utils::Color::White);
        let bishop = piece(utils::Color::White, utils::PieceKind::Bishop);
        let rook = piece(utils::Color::Black, utils::PieceKind::Rook);
        let mov =
            Move::new(utils::Square::G7, utils::Square::H8, MoveKind::Capture);

        pos.board.set_piece(utils::Square::G7, bishop);
        pos.board.set_piece(utils::Square::H8, rook);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[utils::Square::G7], None);
        assert_eq!(pos.board.mailbox[utils::Square::H8], Some(bishop));
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(
            pos.state.castling.bits(),
            ALL_RIGHTS & !utils::CastlingRights::BLACK_KINGSIDE
        );
        assert_eq!(pos.history.arr[0].captured, Some(rook));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[utils::Square::G7], Some(bishop));
        assert_eq!(pos.board.mailbox[utils::Square::H8], Some(rook));
        assert_eq!(pos.state.castling.bits(), ALL_RIGHTS);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn enpassant_captures_on_target_square_and_restores_cleanly()
    {
        let mut pos = pos_with_active(utils::Color::White);
        let white_pawn = piece(utils::Color::White, utils::PieceKind::Pawn);
        let black_pawn = piece(utils::Color::Black, utils::PieceKind::Pawn);
        let mov = Move::new(
            utils::Square::E5,
            utils::Square::D6,
            MoveKind::EnPassant,
        );

        pos.board.set_piece(utils::Square::E5, white_pawn);
        pos.board.set_piece(utils::Square::D5, black_pawn);
        pos.state.enpassant = Some(utils::Square::D6);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[utils::Square::E5], None);
        assert_eq!(pos.board.mailbox[utils::Square::D5], None);
        assert_eq!(pos.board.mailbox[utils::Square::D6], Some(white_pawn));
        assert_eq!(pos.state.enpassant, None);
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.history.arr[0].captured, Some(black_pawn));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[utils::Square::E5], Some(white_pawn));
        assert_eq!(pos.board.mailbox[utils::Square::D5], Some(black_pawn));
        assert_eq!(pos.board.mailbox[utils::Square::D6], None);
        assert_eq!(pos.state.enpassant, Some(utils::Square::D6));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn promotions_place_promoted_piece_and_unmake_restores_pawn()
    {
        let mut pos = pos_with_active(utils::Color::White);
        let pawn = piece(utils::Color::White, utils::PieceKind::Pawn);
        let queen = piece(utils::Color::White, utils::PieceKind::Queen);
        let mov = Move::new(
            utils::Square::A7,
            utils::Square::A8,
            MoveKind::PromoteQueen,
        );

        pos.board.set_piece(utils::Square::A7, pawn);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[utils::Square::A7], None);
        assert_eq!(pos.board.mailbox[utils::Square::A8], Some(queen));
        assert_eq!(pos.state.halfmoves, 0);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[utils::Square::A7], Some(pawn));
        assert_eq!(pos.board.mailbox[utils::Square::A8], None);
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn promotion_captures_restore_captured_piece_on_unmake()
    {
        let mut pos = pos_with_active(utils::Color::White);
        let pawn = piece(utils::Color::White, utils::PieceKind::Pawn);
        let knight = piece(utils::Color::White, utils::PieceKind::Knight);
        let rook = piece(utils::Color::Black, utils::PieceKind::Rook);
        let mov = Move::new(
            utils::Square::G7,
            utils::Square::H8,
            MoveKind::PromoteKnightCapture,
        );

        pos.board.set_piece(utils::Square::G7, pawn);
        pos.board.set_piece(utils::Square::H8, rook);
        refresh_key(&mut pos);

        make(&mut pos, mov);

        assert_eq!(pos.board.mailbox[utils::Square::G7], None);
        assert_eq!(pos.board.mailbox[utils::Square::H8], Some(knight));
        assert_eq!(pos.history.arr[0].captured, Some(rook));
        assert_eq!(
            pos.state.castling.bits(),
            ALL_RIGHTS & !utils::CastlingRights::BLACK_KINGSIDE
        );
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);

        unmake(&mut pos);

        assert_eq!(pos.board.mailbox[utils::Square::G7], Some(pawn));
        assert_eq!(pos.board.mailbox[utils::Square::H8], Some(rook));
        assert_board_consistent(&pos);
        assert_key_is_fresh(&pos);
    }

    #[test]
    fn rook_moves_clear_only_matching_castling_rights()
    {
        let cases = [
            (
                utils::Color::White,
                utils::Square::A1,
                utils::Square::A2,
                ALL_RIGHTS & !utils::CastlingRights::WHITE_QUEENSIDE,
            ),
            (
                utils::Color::White,
                utils::Square::H1,
                utils::Square::H2,
                ALL_RIGHTS & !utils::CastlingRights::WHITE_KINGSIDE,
            ),
            (
                utils::Color::Black,
                utils::Square::A8,
                utils::Square::A7,
                ALL_RIGHTS & !utils::CastlingRights::BLACK_QUEENSIDE,
            ),
            (
                utils::Color::Black,
                utils::Square::H8,
                utils::Square::H7,
                ALL_RIGHTS & !utils::CastlingRights::BLACK_KINGSIDE,
            ),
            (
                utils::Color::White,
                utils::Square::B1,
                utils::Square::B2,
                ALL_RIGHTS,
            ),
        ];

        for (active, from, to, expected_rights) in cases
        {
            let mut pos = pos_with_active(active);
            let rook = piece(active, utils::PieceKind::Rook);
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
                utils::Color::White,
                MoveKind::KingCastle,
                utils::Square::E1,
                utils::Square::G1,
                utils::Square::H1,
                utils::Square::F1,
                ALL_RIGHTS
                    & !(utils::CastlingRights::WHITE_KINGSIDE
                        | utils::CastlingRights::WHITE_QUEENSIDE),
            ),
            (
                utils::Color::White,
                MoveKind::QueenCastle,
                utils::Square::E1,
                utils::Square::C1,
                utils::Square::A1,
                utils::Square::D1,
                ALL_RIGHTS
                    & !(utils::CastlingRights::WHITE_KINGSIDE
                        | utils::CastlingRights::WHITE_QUEENSIDE),
            ),
            (
                utils::Color::Black,
                MoveKind::KingCastle,
                utils::Square::E8,
                utils::Square::G8,
                utils::Square::H8,
                utils::Square::F8,
                ALL_RIGHTS
                    & !(utils::CastlingRights::BLACK_KINGSIDE
                        | utils::CastlingRights::BLACK_QUEENSIDE),
            ),
            (
                utils::Color::Black,
                MoveKind::QueenCastle,
                utils::Square::E8,
                utils::Square::C8,
                utils::Square::A8,
                utils::Square::D8,
                ALL_RIGHTS
                    & !(utils::CastlingRights::BLACK_KINGSIDE
                        | utils::CastlingRights::BLACK_QUEENSIDE),
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
            let king = piece(active, utils::PieceKind::King);
            let rook = piece(active, utils::PieceKind::Rook);
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
        let mut pos = pos_with_active(utils::Color::White);
        let knight = piece(utils::Color::White, utils::PieceKind::Knight);
        let mov =
            Move::new(utils::Square::B1, utils::Square::C3, MoveKind::Quiet);

        pos.board.set_piece(utils::Square::B1, knight);
        refresh_key(&mut pos);

        assert_unmake_restores(&mut pos, mov);
    }
}
