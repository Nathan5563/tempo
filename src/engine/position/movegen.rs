use super::{Position, makemove, super::utils};

#[derive(Debug)]
pub struct MoveList([makemove::Move; utils::MAX_NUM_MOVES]);

impl Default for MoveList
{
    fn default() -> Self
    {
        Self([makemove::Move::default(); utils::MAX_NUM_MOVES])
    }
}

pub fn is_legal(pos: &Position, mov: makemove::Move) -> bool
{
    todo!("Implement legality checking")
}

pub fn generate(pos: &Position, movelist: &mut MoveList)
{
    todo!("Implement move generation")
}
