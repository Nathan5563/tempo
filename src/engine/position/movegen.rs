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

pub fn generate(pos: &Position, movelist: &mut MoveList)
{

}
