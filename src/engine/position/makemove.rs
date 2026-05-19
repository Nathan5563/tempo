use super::Position;

// TODO: Fill out MoveType enum
pub enum MoveType
{

}

// TODO: Pack src, dest, type information into Move struct
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Move(u16);

// TODO: Fill out UndoMove struct
pub struct UndoMove
{

}

// TODO: Implement make function
pub fn make(pos: &mut Position, mov: Move) -> UndoMove
{
    UndoMove {  }
}

// TODO: Implement unmake function
pub fn unmake(pos: &mut Position, undo: UndoMove)
{
    
}
