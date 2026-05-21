use super::super::utils;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BitBoard(u64);

impl BitBoard
{
    pub fn set(&mut self, square: utils::Square)
    {
        self.0 |= 1 << (square as u64);
    }

    pub fn clear(&mut self, square: utils::Square)
    {
        self.0 &= !(1 << (square as u64));
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

        assert_eq!(bitboard.0, 0);
    }

    #[test]
    fn set_maps_every_square_to_its_bit()
    {
        for square in utils::SQUARES
        {
            let mut bitboard = BitBoard::default();

            bitboard.set(square);

            assert_eq!(bitboard.0, bit(square), "{:?}", square);
        }
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
            bitboard.0,
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
            bitboard.0,
            bit(utils::Square::A1) | bit(utils::Square::H8)
        );
    }

    #[test]
    fn clear_missing_square_is_noop()
    {
        let mut bitboard = BitBoard::default();

        bitboard.set(utils::Square::B2);
        bitboard.clear(utils::Square::C3);

        assert_eq!(bitboard.0, bit(utils::Square::B2));
    }
}
