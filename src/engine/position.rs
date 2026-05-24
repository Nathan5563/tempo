mod bitboard;
mod fenparser;
mod makemove;
mod movegen;
mod zobrist;

use super::utils::{
    CastlingRights,
    Color,
    Mailbox,
    MAX_NUM_PLIES,
    NUM_COLORS,
    NUM_PIECE_KINDS,
    Piece,
    PieceKind,
    Square,
};

use self::bitboard::BitBoard;
use self::zobrist::{ZobristRandoms, ZobristType};

pub use self::fenparser::Error as FenError;
pub use self::makemove::{Move, MoveKind};
pub use self::movegen::MoveList;

#[derive(Debug, Clone, Default)]
struct Board
{
    pieces: [BitBoard; NUM_PIECE_KINDS],
    colors: [BitBoard; NUM_COLORS],
    mailbox: Mailbox,
    kings: [Square; NUM_COLORS],
}

impl Board
{
    fn occupied(&self) -> BitBoard
    {
        self.colors[Color::White as usize]
            | self.colors[Color::Black as usize]
    }

    fn set_piece(&mut self, square: Square, piece: Piece)
    {
        self.pieces[piece.kind as usize].set(square);
        self.colors[piece.color as usize].set(square);
        self.mailbox[square] = Some(piece);
        if piece.kind == PieceKind::King
        {
            self.kings[piece.color as usize] = square;
        }
    }

    fn clear_piece(&mut self, square: Square, piece: Piece)
    {
        self.pieces[piece.kind as usize].clear(square);
        self.colors[piece.color as usize].clear(square);
        self.mailbox[square] = None;
    }

    fn move_piece(
        &mut self,
        from: Square,
        to: Square,
        piece: Piece,
    )
    {
        self.clear_piece(from, piece);
        self.set_piece(to, piece);
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct State
{
    key: ZobristType,
    active: Color,
    castling: CastlingRights,
    enpassant: Option<Square>,
    halfmoves: u8,
    fullmoves: u16,
}

#[derive(Debug, Clone, Copy, Default)]
struct Undo
{
    state: State,
    mov: Move,
    captured: Option<Piece>,
}

#[derive(Debug, Clone)]
struct History
{
    arr: [Undo; MAX_NUM_PLIES],
    len: usize
}

impl Default for History
{
    fn default() -> Self
    {
        Self { arr: [Undo::default(); MAX_NUM_PLIES], len: 0 }
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
}

#[derive(Debug, Clone, Default)]
pub struct Position
{
    board: Board,
    state: State,
    history: History,
    zobrists: ZobristRandoms,
}

impl Position
{
    pub fn new(fen: &str) -> Result<Self, FenError>
    {
        let mut pos = Self::default();
        fenparser::parse(fen, &mut pos)?;
        pos.state.key = pos.zobrists.hash(&pos);
        Ok(pos)
    }

    pub fn generate_moves(&mut self, movelist: &mut MoveList)
    {
        movegen::generate(self, movelist);
    }

    pub fn make_move(&mut self, mov: Move)
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
    use crate::engine::utils::SQUARES;

    use super::*;

    fn piece(color: Color, kind: PieceKind) -> Piece
    {
        Piece { color, kind }
    }

    fn bitboard_with(squares: &[Square]) -> BitBoard
    {
        let mut bitboard = BitBoard::default();
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
        let knight = piece(Color::White, PieceKind::Knight);
        let rook = piece(Color::Black, PieceKind::Rook);
        let white_king = piece(Color::White, PieceKind::King);
        let black_king = piece(Color::Black, PieceKind::King);

        board.set_piece(Square::B1, knight);
        board.set_piece(Square::H8, rook);
        board.set_piece(Square::E1, white_king);
        board.set_piece(Square::E8, black_king);

        assert_eq!(board.mailbox[Square::B1], Some(knight));
        assert_eq!(board.mailbox[Square::H8], Some(rook));
        assert_eq!(board.mailbox[Square::E1], Some(white_king));
        assert_eq!(board.mailbox[Square::E8], Some(black_king));
        assert_eq!(
            board.kings[Color::White as usize],
            Square::E1
        );
        assert_eq!(
            board.kings[Color::Black as usize],
            Square::E8
        );
        assert_eq!(
            board.pieces[PieceKind::Knight as usize],
            bitboard_with(&[Square::B1])
        );
        assert_eq!(
            board.pieces[PieceKind::Rook as usize],
            bitboard_with(&[Square::H8])
        );
        assert_eq!(
            board.pieces[PieceKind::King as usize],
            bitboard_with(&[Square::E1, Square::E8])
        );
        assert_eq!(
            board.colors[Color::White as usize],
            bitboard_with(&[Square::B1, Square::E1])
        );
        assert_eq!(
            board.colors[Color::Black as usize],
            bitboard_with(&[Square::H8, Square::E8])
        );

        board.move_piece(Square::B1, Square::C3, knight);
        board.move_piece(Square::E1, Square::G1, white_king);

        assert_eq!(board.mailbox[Square::B1], None);
        assert_eq!(board.mailbox[Square::C3], Some(knight));
        assert_eq!(board.mailbox[Square::E1], None);
        assert_eq!(board.mailbox[Square::G1], Some(white_king));
        assert_eq!(
            board.kings[Color::White as usize],
            Square::G1
        );
        assert_eq!(
            board.pieces[PieceKind::Knight as usize],
            bitboard_with(&[Square::C3])
        );
        assert_eq!(
            board.pieces[PieceKind::King as usize],
            bitboard_with(&[Square::G1, Square::E8])
        );
        assert_eq!(
            board.colors[Color::White as usize],
            bitboard_with(&[Square::C3, Square::G1])
        );

        board.clear_piece(Square::H8, rook);

        assert_eq!(board.mailbox[Square::H8], None);
        assert_eq!(
            board.pieces[PieceKind::Rook as usize],
            BitBoard::default()
        );
        assert_eq!(
            board.colors[Color::Black as usize],
            bitboard_with(&[Square::E8])
        );
    }

    #[test]
    fn history_push_pop_is_lifo()
    {
        let first = Undo {
            state: State {
                key: 11,
                active: Color::White,
                castling: CastlingRights::from_bits(0b0011),
                enpassant: Some(Square::E3),
                halfmoves: 4,
                fullmoves: 9,
            },
            mov: Move::new(
                Square::E2,
                Square::E4,
                MoveKind::DoublePawnPush,
            ),
            captured: None,
        };
        let second = Undo {
            state: State {
                key: 22,
                active: Color::Black,
                castling: CastlingRights::from_bits(0b1100),
                enpassant: Some(Square::D6),
                halfmoves: 0,
                fullmoves: 10,
            },
            mov: Move::new(
                Square::D7,
                Square::D5,
                MoveKind::DoublePawnPush,
            ),
            captured: Some(piece(Color::White, PieceKind::Pawn)),
        };
        let mut history = History::default();

        history.push(first);
        history.push(second);

        let popped = history.pop();
        assert_state_eq(popped.state, second.state);
        assert_eq!(popped.mov, second.mov);
        assert_eq!(popped.captured, second.captured);

        let popped = history.pop();
        assert_state_eq(popped.state, first.state);
        assert_eq!(popped.mov, first.mov);
        assert_eq!(popped.captured, first.captured);
    }

    #[test]
    fn position_default_starts_empty_with_default_state()
    {
        let pos = Position::default();

        for square in SQUARES
        {
            assert_eq!(pos.board.mailbox[square], None);
        }
        assert_eq!(
            pos.board.pieces,
            [BitBoard::default(); NUM_PIECE_KINDS]
        );
        assert_eq!(
            pos.board.colors,
            [BitBoard::default(); NUM_COLORS]
        );
        assert_eq!(pos.state.key, 0);
        assert_eq!(pos.state.active, Color::White);
        assert_eq!(pos.state.castling.bits(), 0);
        assert_eq!(pos.state.enpassant, None);
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.state.fullmoves, 0);
    }

    #[test]
    fn position_make_and_unmake_delegate_to_makemove()
    {
        let mut pos = Position::default();
        let pawn = piece(Color::White, PieceKind::Pawn);
        let mov = Move::new(
            Square::E2,
            Square::E4,
            MoveKind::DoublePawnPush,
        );

        pos.board.set_piece(Square::E2, pawn);
        pos.state.fullmoves = 1;
        pos.state.key = pos.zobrists.hash(&pos);
        let old_state = pos.state;

        pos.make_move(mov);

        assert_eq!(pos.board.mailbox[Square::E2], None);
        assert_eq!(pos.board.mailbox[Square::E4], Some(pawn));
        assert_eq!(pos.state.active, Color::Black);
        assert_eq!(pos.state.enpassant, Some(Square::E3));

        pos.unmake_move();

        assert_eq!(pos.board.mailbox[Square::E2], Some(pawn));
        assert_eq!(pos.board.mailbox[Square::E4], None);
        assert_state_eq(pos.state, old_state);
    }

    #[test]
    fn position_generate_moves_delegates_to_movegen()
    {
        let mut pos = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ).unwrap();
        let mut movelist = movegen::MoveList::default();

        pos.generate_moves(&mut movelist);

        assert_eq!(movelist.len(), 20);
    }

    #[test]
    fn position_new_returns_parse_errors()
    {
        assert!(Position::new("not a fen").is_err());
    }

    #[test]
    fn position_new_parses_fen_and_hashes_state()
    {
        let pos = Position::new("8/8/8/8/4P3/8/8/8 b - e3 0 1").unwrap();

        assert_eq!(
            pos.board.mailbox[Square::E4],
            Some(piece(Color::White, PieceKind::Pawn))
        );
        assert_eq!(pos.state.active, Color::Black);
        assert_eq!(pos.state.enpassant, Some(Square::E3));
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.state.fullmoves, 1);
        assert_eq!(pos.state.key, pos.zobrists.hash(&pos));
    }
}
