pub const NUM_SQUARES: usize = 64;
pub const NUM_PIECE_KINDS: usize = 6;
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
    A1 = 0,  B1 = 1,  C1 = 2,  D1 = 3,  E1 = 4,  F1 = 5,  G1 = 6,  H1 = 7,
    A2 = 8,  B2 = 9,  C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
}

pub const SQUARES: [Square; NUM_SQUARES] = [
    Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
    Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
    Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
    Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
    Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
    Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
    Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
    Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
];

impl std::ops::Add<i32> for Square
{
    type Output = Self;

    fn add(self, rhs: i32) -> Self
    {
        SQUARES[(self as u8 as i32 + rhs) as usize]
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Color
{
    #[default] White,
    Black
}

impl Color
{
    pub fn opposite(&self) -> Self
    {
        if *self == Color::White
        {
            Color::Black
        }
        else
        {
            Color::White
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceKind
{
    #[default] Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Piece
{
    pub color: Color,
    pub kind: PieceKind,
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

impl std::ops::Index<Square> for Mailbox
{
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output
    {
        &self.0[index as u8 as usize]
    }
}

impl std::ops::IndexMut<Square> for Mailbox
{
    fn index_mut(&mut self, index: Square) -> &mut Self::Output
    {
        &mut self.0[index as u8 as usize]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CastlingRights(u8);

impl CastlingRights
{
    pub const WHITE_KINGSIDE: u8  = 0b0001;
    pub const WHITE_QUEENSIDE: u8 = 0b0010;
    pub const BLACK_KINGSIDE: u8  = 0b0100;
    pub const BLACK_QUEENSIDE: u8 = 0b1000;

    #[cfg(test)]
    pub fn from_bits(bits: u8) -> Self
    {
        Self(bits)
    }

    pub fn bits(&self) -> u8
    {
        self.0
    }

    pub fn clear_white(&mut self)
    {
        self.0 &= !(Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE);
    }

    pub fn clear_black(&mut self)
    {
        self.0 &= !(Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE);
    }

    pub fn clear_rook(&mut self, sq: Square)
    {
        match sq
        {
            Square::A1 => self.0 &= !Self::WHITE_QUEENSIDE,
            Square::H1 => self.0 &= !Self::WHITE_KINGSIDE,
            Square::A8 => self.0 &= !Self::BLACK_QUEENSIDE,
            Square::H8 => self.0 &= !Self::BLACK_KINGSIDE,
            _ => {}
        }
    }
}

pub fn enpassant_target(color: Color, square: Square) -> Square
{
    let offset = if color == Color::White { -8 } else { 8 };
    square + offset
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL_RIGHTS: u8 = CastlingRights::WHITE_KINGSIDE
        | CastlingRights::WHITE_QUEENSIDE
        | CastlingRights::BLACK_KINGSIDE
        | CastlingRights::BLACK_QUEENSIDE;

    fn piece(color: Color, kind: PieceKind) -> Piece
    {
        Piece { color, kind }
    }

    #[test]
    fn constants_match_backing_types()
    {
        assert_eq!(NUM_SQUARES, 64);
        assert_eq!(NUM_PIECE_KINDS, 6);
        assert_eq!(NUM_COLORS, 2);
        assert_eq!(MAX_NUM_PLIES, 17697);
        assert_eq!(MAX_NUM_MOVES, 218);

        assert_eq!(Square::A1 as usize, 0);
        assert_eq!(Square::H8 as usize, NUM_SQUARES - 1);
        assert_eq!(Color::White as usize, 0);
        assert_eq!(Color::Black as usize, NUM_COLORS - 1);
        assert_eq!(PieceKind::Pawn as usize, 0);
        assert_eq!(PieceKind::King as usize, NUM_PIECE_KINDS - 1);
    }

    #[test]
    fn squares_array_maps_every_index_to_its_square()
    {
        assert_eq!(SQUARES.len(), NUM_SQUARES);

        for (index, square) in SQUARES.into_iter().enumerate()
        {
            assert_eq!(square as usize, index, "{:?}", square);
            assert_eq!(SQUARES[square as usize], square);
        }
    }

    #[test]
    fn square_addition_moves_by_index_offset()
    {
        assert_eq!(Square::A1 + 1, Square::B1);
        assert_eq!(Square::A1 + 8, Square::A2);
        assert_eq!(Square::E4 + -8, Square::E3);
        assert_eq!(Square::H8 + -63, Square::A1);
    }

    #[test]
    fn defaults_are_empty_white_pawn_state()
    {
        assert_eq!(Square::default(), Square::A1);
        assert_eq!(Color::default(), Color::White);
        assert_eq!(PieceKind::default(), PieceKind::Pawn);
        assert_eq!(Piece::default(), piece(Color::White, PieceKind::Pawn));
        assert_eq!(CastlingRights::default().bits(), 0);
    }

    #[test]
    fn color_opposite_round_trips()
    {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
        assert_eq!(Color::White.opposite().opposite(), Color::White);
        assert_eq!(Color::Black.opposite().opposite(), Color::Black);
    }

    #[test]
    fn mailbox_default_is_empty()
    {
        let mailbox = Mailbox::default();

        for square in SQUARES
        {
            assert_eq!(mailbox[square], None, "{:?}", square);
        }
    }

    #[test]
    fn mailbox_index_mut_updates_only_selected_square()
    {
        let mut mailbox = Mailbox::default();
        let knight = piece(Color::White, PieceKind::Knight);
        let bishop = piece(Color::Black, PieceKind::Bishop);

        mailbox[Square::E4] = Some(knight);
        mailbox[Square::A8] = Some(bishop);
        mailbox[Square::E4] = None;

        for square in SQUARES
        {
            let expected = match square
            {
                Square::A8 => Some(bishop),
                _ => None,
            };

            assert_eq!(mailbox[square], expected, "{:?}", square);
        }
    }

    #[test]
    fn castling_right_bits_are_stable()
    {
        assert_eq!(CastlingRights::WHITE_KINGSIDE, 0b0001);
        assert_eq!(CastlingRights::WHITE_QUEENSIDE, 0b0010);
        assert_eq!(CastlingRights::BLACK_KINGSIDE, 0b0100);
        assert_eq!(CastlingRights::BLACK_QUEENSIDE, 0b1000);
        assert_eq!(ALL_RIGHTS, 0b1111);
        assert_eq!(CastlingRights::from_bits(0b1010).bits(), 0b1010);
    }

    #[test]
    fn clear_white_and_black_remove_side_rights()
    {
        let mut rights = CastlingRights::from_bits(ALL_RIGHTS);

        rights.clear_white();

        assert_eq!(
            rights.bits(),
            CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE
        );

        rights.clear_black();

        assert_eq!(rights.bits(), 0);
    }

    #[test]
    fn clear_rook_removes_only_matching_corner_right()
    {
        let cases = [
            (Square::A1, ALL_RIGHTS & !CastlingRights::WHITE_QUEENSIDE),
            (Square::H1, ALL_RIGHTS & !CastlingRights::WHITE_KINGSIDE),
            (Square::A8, ALL_RIGHTS & !CastlingRights::BLACK_QUEENSIDE),
            (Square::H8, ALL_RIGHTS & !CastlingRights::BLACK_KINGSIDE),
            (Square::E1, ALL_RIGHTS),
            (Square::D4, ALL_RIGHTS),
        ];

        for (square, expected) in cases
        {
            let mut rights = CastlingRights::from_bits(ALL_RIGHTS);

            rights.clear_rook(square);

            assert_eq!(rights.bits(), expected, "{:?}", square);
        }
    }

    #[test]
    fn enpassant_target_steps_behind_double_pushed_pawn()
    {
        assert_eq!(enpassant_target(Color::White, Square::A4), Square::A3);
        assert_eq!(enpassant_target(Color::White, Square::E4), Square::E3);
        assert_eq!(enpassant_target(Color::Black, Square::D5), Square::D6);
        assert_eq!(enpassant_target(Color::Black, Square::H5), Square::H6);
    }
}
