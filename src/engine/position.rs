use super::utils;

mod bitboard;
mod fenparser;
mod makemove;
mod movegen;
mod zobrist;

#[derive(Debug, Default)]
struct Board
{
    pieces: [bitboard::BitBoard; utils::NUM_PIECE_KINDS],
    colors: [bitboard::BitBoard; utils::NUM_COLORS],
    mailbox: utils::Mailbox,
}

impl Board
{
    fn set_piece(&mut self, square: utils::Square, piece: utils::Piece)
    {
        self.pieces[piece.kind as usize].set(square);
        self.colors[piece.color as usize].set(square);
        self.mailbox[square] = Some(piece);
    }

    fn clear_piece(&mut self, square: utils::Square, piece: utils::Piece)
    {
        self.pieces[piece.kind as usize].clear(square);
        self.colors[piece.color as usize].clear(square);
        self.mailbox[square] = None;
    }

    fn move_piece(
        &mut self,
        from: utils::Square,
        to: utils::Square,
        piece: utils::Piece,
    )
    {
        self.clear_piece(from, piece);
        self.set_piece(to, piece);
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct State
{
    key: zobrist::ZobristType,
    active: utils::Color,
    castling: utils::CastlingRights,
    enpassant: Option<utils::Square>,
    halfmoves: u8,
    fullmoves: u16,
}

#[derive(Debug, Clone, Copy, Default)]
struct Undo
{
    state: State,
    mov: makemove::Move,
    captured: Option<utils::Piece>,
}

#[derive(Debug)]
struct History
{
    arr: [Undo; utils::MAX_NUM_PLIES],
    len: usize
}

impl Default for History
{
    fn default() -> Self
    {
        Self { arr: [Undo::default(); utils::MAX_NUM_PLIES], len: 0 }
    }
}

impl History
{
    pub fn push(&mut self, undo: Undo)
    {
        self.arr[self.len] = undo;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Undo
    {
        self.len -= 1;
        self.arr[self.len]
    }

    pub fn length(&self) -> usize
    {
        self.len
    }
}

#[derive(Debug, Default)]
pub struct Position
{
    board: Board,
    state: State,
    history: History,
    zobrists: zobrist::ZobristRandoms,
}

impl Position
{
    pub fn new(fen: &str) -> Result<Self, fenparser::Error>
    {
        let mut pos = Self::default();
        fenparser::parse(fen, &mut pos)?;
        pos.state.key = pos.zobrists.hash(&pos);
        Ok(pos)
    }

    pub fn generate_moves(&self, movelist: &mut movegen::MoveList)
    {
        movegen::generate(self, movelist);
    }

    pub fn make_move(&mut self, mov: makemove::Move)
    {
        makemove::make(self, mov);
    }

    pub fn unmake_move(&mut self)
    {
        makemove::unmake(self);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn piece(color: utils::Color, kind: utils::PieceKind) -> utils::Piece
    {
        utils::Piece { color, kind }
    }

    fn bitboard_with(squares: &[utils::Square]) -> bitboard::BitBoard
    {
        let mut bitboard = bitboard::BitBoard::default();
        for square in squares
        {
            bitboard.set(*square);
        }
        bitboard
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

    #[test]
    fn board_set_clear_and_move_keep_all_representations_in_sync()
    {
        let mut board = Board::default();
        let knight = piece(utils::Color::White, utils::PieceKind::Knight);
        let rook = piece(utils::Color::Black, utils::PieceKind::Rook);

        board.set_piece(utils::Square::B1, knight);
        board.set_piece(utils::Square::H8, rook);

        assert_eq!(board.mailbox[utils::Square::B1], Some(knight));
        assert_eq!(board.mailbox[utils::Square::H8], Some(rook));
        assert_eq!(
            board.pieces[utils::PieceKind::Knight as usize],
            bitboard_with(&[utils::Square::B1])
        );
        assert_eq!(
            board.pieces[utils::PieceKind::Rook as usize],
            bitboard_with(&[utils::Square::H8])
        );
        assert_eq!(
            board.colors[utils::Color::White as usize],
            bitboard_with(&[utils::Square::B1])
        );
        assert_eq!(
            board.colors[utils::Color::Black as usize],
            bitboard_with(&[utils::Square::H8])
        );

        board.move_piece(utils::Square::B1, utils::Square::C3, knight);

        assert_eq!(board.mailbox[utils::Square::B1], None);
        assert_eq!(board.mailbox[utils::Square::C3], Some(knight));
        assert_eq!(
            board.pieces[utils::PieceKind::Knight as usize],
            bitboard_with(&[utils::Square::C3])
        );
        assert_eq!(
            board.colors[utils::Color::White as usize],
            bitboard_with(&[utils::Square::C3])
        );

        board.clear_piece(utils::Square::H8, rook);

        assert_eq!(board.mailbox[utils::Square::H8], None);
        assert_eq!(
            board.pieces[utils::PieceKind::Rook as usize],
            bitboard::BitBoard::default()
        );
        assert_eq!(
            board.colors[utils::Color::Black as usize],
            bitboard::BitBoard::default()
        );
    }

    #[test]
    fn history_push_pop_is_lifo_and_tracks_length()
    {
        let first = Undo {
            state: State {
                key: 11,
                active: utils::Color::White,
                castling: utils::CastlingRights::from_bits(0b0011),
                enpassant: Some(utils::Square::E3),
                halfmoves: 4,
                fullmoves: 9,
            },
            mov: makemove::Move::new(
                utils::Square::E2,
                utils::Square::E4,
                makemove::MoveKind::DoublePawnPush,
            ),
            captured: None,
        };
        let second = Undo {
            state: State {
                key: 22,
                active: utils::Color::Black,
                castling: utils::CastlingRights::from_bits(0b1100),
                enpassant: Some(utils::Square::D6),
                halfmoves: 0,
                fullmoves: 10,
            },
            mov: makemove::Move::new(
                utils::Square::D7,
                utils::Square::D5,
                makemove::MoveKind::DoublePawnPush,
            ),
            captured: Some(piece(utils::Color::White, utils::PieceKind::Pawn)),
        };
        let mut history = History::default();

        assert_eq!(history.length(), 0);

        history.push(first);
        history.push(second);

        assert_eq!(history.length(), 2);

        let popped = history.pop();
        assert_state_eq(popped.state, second.state);
        assert_eq!(popped.mov, second.mov);
        assert_eq!(popped.captured, second.captured);
        assert_eq!(history.length(), 1);

        let popped = history.pop();
        assert_state_eq(popped.state, first.state);
        assert_eq!(popped.mov, first.mov);
        assert_eq!(popped.captured, first.captured);
        assert_eq!(history.length(), 0);
    }

    #[test]
    fn position_default_starts_empty_with_default_state()
    {
        let pos = Position::default();

        for square in utils::SQUARES
        {
            assert_eq!(pos.board.mailbox[square], None);
        }
        assert_eq!(
            pos.board.pieces,
            [bitboard::BitBoard::default(); utils::NUM_PIECE_KINDS]
        );
        assert_eq!(
            pos.board.colors,
            [bitboard::BitBoard::default(); utils::NUM_COLORS]
        );
        assert_eq!(pos.state.key, 0);
        assert_eq!(pos.state.active, utils::Color::White);
        assert_eq!(pos.state.castling.bits(), 0);
        assert_eq!(pos.state.enpassant, None);
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.state.fullmoves, 0);
        assert_eq!(pos.history.length(), 0);
    }

    #[test]
    fn position_make_and_unmake_delegate_to_makemove()
    {
        let mut pos = Position::default();
        let pawn = piece(utils::Color::White, utils::PieceKind::Pawn);
        let mov = makemove::Move::new(
            utils::Square::E2,
            utils::Square::E4,
            makemove::MoveKind::DoublePawnPush,
        );

        pos.board.set_piece(utils::Square::E2, pawn);
        pos.state.fullmoves = 1;
        pos.state.key = pos.zobrists.hash(&pos);
        let old_state = pos.state;

        pos.make_move(mov);

        assert_eq!(pos.board.mailbox[utils::Square::E2], None);
        assert_eq!(pos.board.mailbox[utils::Square::E4], Some(pawn));
        assert_eq!(pos.state.active, utils::Color::Black);
        assert_eq!(pos.state.enpassant, Some(utils::Square::E3));
        assert_eq!(pos.history.length(), 1);

        pos.unmake_move();

        assert_eq!(pos.board.mailbox[utils::Square::E2], Some(pawn));
        assert_eq!(pos.board.mailbox[utils::Square::E4], None);
        assert_state_eq(pos.state, old_state);
        assert_eq!(pos.history.length(), 0);
    }

    #[test]
    fn position_new_parses_fen_and_hashes_state()
    {
        let pos = Position::new("8/8/8/8/4P3/8/8/8 b - e3 0 1").unwrap();

        assert_eq!(
            pos.board.mailbox[utils::Square::E4],
            Some(piece(utils::Color::White, utils::PieceKind::Pawn))
        );
        assert_eq!(pos.state.active, utils::Color::Black);
        assert_eq!(pos.state.enpassant, Some(utils::Square::E3));
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.state.fullmoves, 1);
        assert_eq!(pos.state.key, pos.zobrists.hash(&pos));
    }
}
