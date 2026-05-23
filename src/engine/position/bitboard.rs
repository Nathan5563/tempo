use super::super::utils;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BitBoard(u64);

impl BitBoard
{
    pub const fn from_bits(bits: u64) -> Self
    {
        Self(bits)
    }

    pub const fn from_square(square: utils::Square) -> Self
    {
        Self(1u64 << (square as u8))
    }

    pub const fn bits(&self) -> u64
    {
        self.0
    }

    pub const fn contains(&self, square: utils::Square) -> bool
    {
        (self.0 & (1u64 << (square as u8))) != 0
    }

    pub const fn is_empty(&self) -> bool
    {
        self.0 == 0
    }

    pub fn set(&mut self, square: utils::Square)
    {
        self.0 |= 1 << (square as u64);
    }

    pub fn clear(&mut self, square: utils::Square)
    {
        self.0 &= !(1 << (square as u64));
    }

    pub fn pop_lsb(&mut self) -> Option<utils::Square>
    {
        if self.is_empty()
        {
            return None;
        }

        let index = self.0.trailing_zeros() as usize;
        self.0 &= self.0 - 1;
        Some(utils::SQUARES[index])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Iter(BitBoard);

impl Iterator for Iter
{
    type Item = utils::Square;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.0.pop_lsb()
    }
}

impl IntoIterator for BitBoard
{
    type Item = utils::Square;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter
    {
        Iter(self)
    }
}

impl std::ops::BitAnd for BitBoard
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output
    {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for BitBoard
{
    fn bitand_assign(&mut self, rhs: Self)
    {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOr for BitBoard
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output
    {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BitBoard
{
    fn bitor_assign(&mut self, rhs: Self)
    {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXor for BitBoard
{
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output
    {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for BitBoard
{
    fn bitxor_assign(&mut self, rhs: Self)
    {
        self.0 ^= rhs.0;
    }
}

impl std::ops::Not for BitBoard
{
    type Output = Self;

    fn not(self) -> Self::Output
    {
        Self(!self.0)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn bit(square: utils::Square) -> u64
    {
        1u64 << (square as u64)
    }

    #[test]
    fn default_is_empty()
    {
        let bitboard = BitBoard::default();

        assert_eq!(bitboard.bits(), 0);
        assert!(bitboard.is_empty());
    }

    #[test]
    fn set_maps_every_square_to_its_bit()
    {
        for square in utils::SQUARES
        {
            let mut bitboard = BitBoard::default();

            bitboard.set(square);

            assert_eq!(bitboard.bits(), bit(square), "{:?}", square);
            assert!(bitboard.contains(square), "{:?}", square);
            assert_eq!(BitBoard::from_square(square), bitboard);
        }
    }

    #[test]
    fn from_bits_preserves_backing_bits()
    {
        let bits = bit(utils::Square::A1)
            | bit(utils::Square::D4)
            | bit(utils::Square::H8);
        let bitboard = BitBoard::from_bits(bits);

        assert_eq!(bitboard.bits(), bits);
        assert!(bitboard.contains(utils::Square::A1));
        assert!(bitboard.contains(utils::Square::D4));
        assert!(bitboard.contains(utils::Square::H8));
        assert!(!bitboard.contains(utils::Square::E5));
    }

    #[test]
    fn set_accumulates_bits_and_is_idempotent()
    {
        let mut bitboard = BitBoard::default();

        bitboard.set(utils::Square::A1);
        bitboard.set(utils::Square::D4);
        bitboard.set(utils::Square::H8);
        bitboard.set(utils::Square::D4);

        assert_eq!(
            bitboard.bits(),
            bit(utils::Square::A1) 
            | bit(utils::Square::D4) 
            | bit(utils::Square::H8)
        );
    }

    #[test]
    fn clear_removes_only_the_requested_bit()
    {
        let mut bitboard = BitBoard::default();

        bitboard.set(utils::Square::A1);
        bitboard.set(utils::Square::D4);
        bitboard.set(utils::Square::H8);

        bitboard.clear(utils::Square::D4);

        assert_eq!(
            bitboard.bits(),
            bit(utils::Square::A1) | bit(utils::Square::H8)
        );
    }

    #[test]
    fn clear_missing_square_is_noop()
    {
        let mut bitboard = BitBoard::default();

        bitboard.set(utils::Square::B2);
        bitboard.clear(utils::Square::C3);

        assert_eq!(bitboard.bits(), bit(utils::Square::B2));
    }

    #[test]
    fn pop_lsb_returns_squares_in_index_order()
    {
        let mut bitboard = BitBoard::from_bits(
            bit(utils::Square::H8)
                | bit(utils::Square::A1)
                | bit(utils::Square::D4),
        );

        assert_eq!(bitboard.pop_lsb(), Some(utils::Square::A1));
        assert_eq!(bitboard.pop_lsb(), Some(utils::Square::D4));
        assert_eq!(bitboard.pop_lsb(), Some(utils::Square::H8));
        assert_eq!(bitboard.pop_lsb(), None);
        assert!(bitboard.is_empty());
    }

    #[test]
    fn bitwise_ops_preserve_bitboard_type()
    {
        let left = BitBoard::from_bits(
            bit(utils::Square::A1) | bit(utils::Square::D4),
        );
        let right = BitBoard::from_bits(
            bit(utils::Square::D4) | bit(utils::Square::H8),
        );

        assert_eq!((left & right).bits(), bit(utils::Square::D4));
        assert_eq!(
            (left | right).bits(),
            bit(utils::Square::A1)
                | bit(utils::Square::D4)
                | bit(utils::Square::H8)
        );
        assert_eq!(
            (left ^ right).bits(),
            bit(utils::Square::A1) | bit(utils::Square::H8)
        );

        let mut assigned = left;
        assigned &= right;
        assert_eq!(assigned.bits(), bit(utils::Square::D4));

        assigned |= BitBoard::from_square(utils::Square::H8);
        assert_eq!(
            assigned.bits(),
            bit(utils::Square::D4) | bit(utils::Square::H8)
        );

        assigned ^= BitBoard::from_bits(
            bit(utils::Square::D4) | bit(utils::Square::A1),
        );
        assert_eq!(
            assigned.bits(),
            bit(utils::Square::A1) | bit(utils::Square::H8)
        );
    }
}
