use super::{Board, History, Position, State};

use crate::engine::utils::{
    CastlingRights,
    Color,
    Piece,
    PieceKind,
    SQUARES,
    Square,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field
{
    Board,
    Active,
    Castling,
    Enpassant,
    Halfmoves,
    Fullmoves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error
{
    MissingField(Field),
    TooManyFields,
    InvalidRankCount { found: usize },
    InvalidRankWidth { rank: u8, width: usize },
    AdjacentEmptyCounts { rank: u8 },
    InvalidPiece(char),
    InvalidActiveColor,
    InvalidCastling,
    InvalidEnpassant,
    InvalidHalfmoves,
    InvalidFullmoves,
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Error::MissingField(field) =>
            {
                write!(f, "missing FEN field: {:?}", field)
            }
            Error::TooManyFields => write!(f, "too many FEN fields"),
            Error::InvalidRankCount { found } =>
            {
                write!(f, "invalid number of FEN ranks: {}", found)
            }
            Error::InvalidRankWidth { rank, width } =>
            {
                write!(f, "FEN rank {} has width {}", rank, width)
            }
            Error::AdjacentEmptyCounts { rank } =>
            {
                write!(f, "FEN rank {} has adjacent empty counts", rank)
            }
            Error::InvalidPiece(piece) =>
            {
                write!(f, "invalid FEN piece: {}", piece)
            }
            Error::InvalidActiveColor => write!(f, "invalid FEN active color"),
            Error::InvalidCastling => write!(f, "invalid FEN castling rights"),
            Error::InvalidEnpassant =>
            {
                write!(f, "invalid FEN en passant square")
            }
            Error::InvalidHalfmoves => write!(f, "invalid FEN halfmove clock"),
            Error::InvalidFullmoves => write!(f, "invalid FEN fullmove number"),
        }
    }
}

impl std::error::Error for Error {}

pub fn parse(fen: &str, pos: &mut Position) -> Result<(), Error>
{
    let mut fields = fen.split_ascii_whitespace();

    let board_field = next_field(&mut fields, Field::Board)?;
    let active_field = next_field(&mut fields, Field::Active)?;
    let castling_field = next_field(&mut fields, Field::Castling)?;
    let enpassant_field = next_field(&mut fields, Field::Enpassant)?;
    let halfmoves_field = next_field(&mut fields, Field::Halfmoves)?;
    let fullmoves_field = next_field(&mut fields, Field::Fullmoves)?;

    if fields.next().is_some()
    {
        return Err(Error::TooManyFields);
    }

    let mut board = Board::default();
    let active = parse_active(active_field)?;
    let state = State {
        key: 0,
        active,
        castling: parse_castling(castling_field)?,
        enpassant: parse_enpassant(enpassant_field, active)?,
        halfmoves: parse_halfmoves(halfmoves_field)?,
        fullmoves: parse_fullmoves(fullmoves_field)?,
    };

    parse_board(board_field, &mut board)?;

    pos.board = board;
    pos.state = state;
    pos.history = History::default();

    Ok(())
}

fn next_field<'a>(
    fields: &mut std::str::SplitAsciiWhitespace<'a>,
    field: Field,
) -> Result<&'a str, Error>
{
    fields.next().ok_or(Error::MissingField(field))
}

fn parse_board(field: &str, board: &mut Board) -> Result<(), Error>
{
    let ranks = field.split('/').collect::<Vec<_>>();
    if ranks.len() != 8
    {
        return Err(Error::InvalidRankCount { found: ranks.len() });
    }

    for (index, rank) in ranks.into_iter().enumerate()
    {
        parse_rank(rank, 8 - index as u8, board)?;
    }

    Ok(())
}

fn parse_rank(field: &str, rank: u8, board: &mut Board) -> Result<(), Error>
{
    let mut file = 0;
    let mut previous_empty_count = false;

    for ch in field.chars()
    {
        match ch
        {
            '1'..='8' =>
            {
                if previous_empty_count
                {
                    return Err(Error::AdjacentEmptyCounts { rank });
                }

                file += ch as usize - '0' as usize;
                previous_empty_count = true;
            }
            _ =>
            {
                let piece = parse_piece(ch)?;
                if file >= 8
                {
                    return Err(Error::InvalidRankWidth {
                        rank,
                        width: file + 1,
                    });
                }

                let square = SQUARES[(rank as usize - 1) * 8 + file];
                board.set_piece(square, piece);
                file += 1;
                previous_empty_count = false;
            }
        }

        if file > 8
        {
            return Err(Error::InvalidRankWidth { rank, width: file });
        }
    }

    if file != 8
    {
        return Err(Error::InvalidRankWidth { rank, width: file });
    }

    Ok(())
}

fn parse_piece(ch: char) -> Result<Piece, Error>
{
    let color = if ch.is_ascii_uppercase()
    {
        Color::White
    }
    else
    {
        Color::Black
    };

    let kind = match ch.to_ascii_lowercase()
    {
        'p' => PieceKind::Pawn,
        'n' => PieceKind::Knight,
        'b' => PieceKind::Bishop,
        'r' => PieceKind::Rook,
        'q' => PieceKind::Queen,
        'k' => PieceKind::King,
        _ => return Err(Error::InvalidPiece(ch)),
    };

    Ok(Piece { color, kind })
}

fn parse_active(field: &str) -> Result<Color, Error>
{
    match field
    {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(Error::InvalidActiveColor),
    }
}

fn parse_castling(field: &str) -> Result<CastlingRights, Error>
{
    if field == "-"
    {
        return Ok(CastlingRights::from_bits(0));
    }

    let mut bits = 0;
    let mut previous_order = 0;

    for ch in field.chars()
    {
        let (bit, order) = match ch
        {
            'K' => (CastlingRights::WHITE_KINGSIDE, 1),
            'Q' => (CastlingRights::WHITE_QUEENSIDE, 2),
            'k' => (CastlingRights::BLACK_KINGSIDE, 3),
            'q' => (CastlingRights::BLACK_QUEENSIDE, 4),
            _ => return Err(Error::InvalidCastling),
        };

        if order <= previous_order || bits & bit != 0
        {
            return Err(Error::InvalidCastling);
        }

        bits |= bit;
        previous_order = order;
    }

    if bits == 0
    {
        return Err(Error::InvalidCastling);
    }

    Ok(CastlingRights::from_bits(bits))
}

fn parse_enpassant(
    field: &str,
    active: Color,
) -> Result<Option<Square>, Error>
{
    if field == "-"
    {
        return Ok(None);
    }

    let bytes = field.as_bytes();
    if bytes.len() != 2
    {
        return Err(Error::InvalidEnpassant);
    }

    let file = bytes[0];
    let rank = bytes[1];
    if !(b'a'..=b'h').contains(&file)
    {
        return Err(Error::InvalidEnpassant);
    }

    let expected_rank = match active
    {
        Color::White => b'6',
        Color::Black => b'3',
    };
    if rank != expected_rank
    {
        return Err(Error::InvalidEnpassant);
    }

    let file_index = (file - b'a') as usize;
    let rank_index = (rank - b'1') as usize;

    Ok(Some(SQUARES[rank_index * 8 + file_index]))
}

fn parse_halfmoves(field: &str) -> Result<u8, Error>
{
    if !is_unsigned_decimal(field)
    {
        return Err(Error::InvalidHalfmoves);
    }

    field.parse().map_err(|_| Error::InvalidHalfmoves)
}

fn parse_fullmoves(field: &str) -> Result<u16, Error>
{
    if !is_unsigned_decimal(field)
    {
        return Err(Error::InvalidFullmoves);
    }

    let fullmoves = field.parse().map_err(|_| Error::InvalidFullmoves)?;
    if fullmoves == 0
    {
        return Err(Error::InvalidFullmoves);
    }

    Ok(fullmoves)
}

fn is_unsigned_decimal(field: &str) -> bool
{
    !field.is_empty() && field.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn piece(color: Color, kind: PieceKind) -> Piece
    {
        Piece { color, kind }
    }

    fn parse_error(fen: &str) -> Error
    {
        let mut pos = Position::default();
        parse(fen, &mut pos).unwrap_err()
    }

    #[test]
    fn parse_starting_position()
    {
        let mut pos = Position::default();

        parse(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &mut pos,
        )
        .unwrap();

        assert_eq!(
            pos.board.mailbox[Square::A1],
            Some(piece(Color::White, PieceKind::Rook))
        );
        assert_eq!(
            pos.board.mailbox[Square::E1],
            Some(piece(Color::White, PieceKind::King))
        );
        assert_eq!(
            pos.board.mailbox[Square::D8],
            Some(piece(Color::Black, PieceKind::Queen))
        );
        assert_eq!(
            pos.board.mailbox[Square::H7],
            Some(piece(Color::Black, PieceKind::Pawn))
        );
        assert_eq!(
            pos.board.kings[Color::White as usize],
            Square::E1
        );
        assert_eq!(
            pos.board.kings[Color::Black as usize],
            Square::E8
        );
        assert_eq!(pos.board.mailbox[Square::E4], None);
        assert_eq!(pos.state.key, 0);
        assert_eq!(pos.state.active, Color::White);
        assert_eq!(pos.state.castling.bits(), 0b1111);
        assert_eq!(pos.state.enpassant, None);
        assert_eq!(pos.state.halfmoves, 0);
        assert_eq!(pos.state.fullmoves, 1);
    }

    #[test]
    fn parse_white_enpassant_square()
    {
        let mut pos = Position::default();

        parse(
            "rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR w KQkq e6 0 2",
            &mut pos,
        )
        .unwrap();

        assert_eq!(pos.state.active, Color::White);
        assert_eq!(pos.state.enpassant, Some(Square::E6));
        assert_eq!(
            pos.board.mailbox[Square::E5],
            Some(piece(Color::Black, PieceKind::Pawn))
        );
    }

    #[test]
    fn parse_enpassant_square_for_side_to_move()
    {
        let mut pos = Position::default();

        parse(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            &mut pos,
        )
        .unwrap();

        assert_eq!(pos.state.active, Color::Black);
        assert_eq!(pos.state.enpassant, Some(Square::E3));
        assert_eq!(
            pos.board.mailbox[Square::E4],
            Some(piece(Color::White, PieceKind::Pawn))
        );
    }

    #[test]
    fn reject_invalid_board_rank_width()
    {
        let mut pos = Position::default();

        let err = parse("8/8/8/8/8/8/8/7 w - - 0 1", &mut pos).unwrap_err();

        assert_eq!(err, Error::InvalidRankWidth { rank: 1, width: 7 });
    }

    #[test]
    fn reject_invalid_board_rank_count()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8 w - - 0 1"),
            Error::InvalidRankCount { found: 7 }
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8/8 w - - 0 1"),
            Error::InvalidRankCount { found: 9 }
        );
    }

    #[test]
    fn reject_invalid_board_piece()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/7X w - - 0 1"),
            Error::InvalidPiece('X')
        );
    }

    #[test]
    fn reject_board_rank_overflow()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8P w - - 0 1"),
            Error::InvalidRankWidth { rank: 1, width: 9 }
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/P8 w - - 0 1"),
            Error::InvalidRankWidth { rank: 1, width: 9 }
        );
    }

    #[test]
    fn reject_noncanonical_adjacent_empty_counts()
    {
        let mut pos = Position::default();

        let err = parse("8/8/8/8/8/8/8/11K5 w - - 0 1", &mut pos).unwrap_err();

        assert_eq!(err, Error::AdjacentEmptyCounts { rank: 1 });
    }

    #[test]
    fn reject_invalid_enpassant_rank_for_active_color()
    {
        let mut pos = Position::default();

        let err = parse("8/8/8/8/8/8/8/8 w - e3 0 1", &mut pos).unwrap_err();

        assert_eq!(err, Error::InvalidEnpassant);
    }

    #[test]
    fn reject_missing_fields()
    {
        let cases = [
            ("", Error::MissingField(Field::Board)),
            ("8/8/8/8/8/8/8/8", Error::MissingField(Field::Active)),
            ("8/8/8/8/8/8/8/8 w", Error::MissingField(Field::Castling)),
            ("8/8/8/8/8/8/8/8 w -", Error::MissingField(Field::Enpassant)),
            (
                "8/8/8/8/8/8/8/8 w - -",
                Error::MissingField(Field::Halfmoves),
            ),
            (
                "8/8/8/8/8/8/8/8 w - - 0",
                Error::MissingField(Field::Fullmoves),
            ),
        ];

        for (fen, expected) in cases
        {
            assert_eq!(parse_error(fen), expected);
        }
    }

    #[test]
    fn reject_invalid_active_color()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 x - - 0 1"),
            Error::InvalidActiveColor
        );
    }

    #[test]
    fn reject_invalid_castling_rights()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 w A - 0 1"),
            Error::InvalidCastling
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 w QK - 0 1"),
            Error::InvalidCastling
        );
        assert_eq!(parse_castling("").unwrap_err(), Error::InvalidCastling);
    }

    #[test]
    fn reject_invalid_enpassant_shape()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 b - e33 0 1"),
            Error::InvalidEnpassant
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 b - i3 0 1"),
            Error::InvalidEnpassant
        );
    }

    #[test]
    fn reject_invalid_move_counters()
    {
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 w - - x 1"),
            Error::InvalidHalfmoves
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 w - - 256 1"),
            Error::InvalidHalfmoves
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 w - - 0 x"),
            Error::InvalidFullmoves
        );
        assert_eq!(
            parse_error("8/8/8/8/8/8/8/8 w - - 0 65536"),
            Error::InvalidFullmoves
        );
        assert!(!is_unsigned_decimal(""));
    }

    #[test]
    fn reject_extra_fields()
    {
        let mut pos = Position::default();

        let err =
            parse("8/8/8/8/8/8/8/8 w - - 0 1 extra", &mut pos).unwrap_err();

        assert_eq!(err, Error::TooManyFields);
    }

    #[test]
    fn failed_parse_does_not_mutate_position()
    {
        let mut pos = Position::default();
        let rook = piece(Color::White, PieceKind::Rook);
        pos.board.set_piece(Square::A1, rook);
        pos.state.active = Color::Black;
        pos.state.fullmoves = 9;

        let err = parse("8/8/8/8/8/8/8/8 w - - 0 0", &mut pos).unwrap_err();

        assert_eq!(err, Error::InvalidFullmoves);
        assert_eq!(pos.board.mailbox[Square::A1], Some(rook));
        assert_eq!(pos.state.active, Color::Black);
        assert_eq!(pos.state.fullmoves, 9);
    }

    #[test]
    fn error_display_messages_are_specific()
    {
        let cases = [
            (
                Error::MissingField(Field::Board),
                "missing FEN field: Board",
            ),
            (Error::TooManyFields, "too many FEN fields"),
            (
                Error::InvalidRankCount { found: 3 },
                "invalid number of FEN ranks: 3",
            ),
            (
                Error::InvalidRankWidth { rank: 4, width: 9 },
                "FEN rank 4 has width 9",
            ),
            (
                Error::AdjacentEmptyCounts { rank: 2 },
                "FEN rank 2 has adjacent empty counts",
            ),
            (Error::InvalidPiece('x'), "invalid FEN piece: x"),
            (Error::InvalidActiveColor, "invalid FEN active color"),
            (Error::InvalidCastling, "invalid FEN castling rights"),
            (Error::InvalidEnpassant, "invalid FEN en passant square"),
            (Error::InvalidHalfmoves, "invalid FEN halfmove clock"),
            (Error::InvalidFullmoves, "invalid FEN fullmove number"),
        ];

        for (error, expected) in cases
        {
            assert_eq!(error.to_string(), expected);
        }
    }
}
