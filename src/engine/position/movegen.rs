use super::{Position, makemove, super::utils};

#[derive(Debug)]
pub struct MoveList
{
    arr: [makemove::Move; utils::MAX_NUM_MOVES],
    len: usize
}

impl Default for MoveList
{
    fn default() -> Self
    {
        Self { arr: [makemove::Move::default(); utils::MAX_NUM_MOVES], len: 0 }
    }
}

impl MoveList
{
    fn clear(&mut self)
    {
        self.len = 0;
    }

    fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut(makemove::Move) -> bool,
    {
        let mut write = 0;

        for read in 0..self.len
        {
            let mov = self.arr[read];

            if keep(mov)
            {
                self.arr[write] = mov;
                write += 1;
            }
        }

        self.len = write;
    }
}

pub fn generate(pos: &mut Position, movelist: &mut MoveList)
{
    movelist.clear();
    generate_pseudolegal(pos, movelist);
    movelist.retain(|mov| is_legal(pos, mov));
}

fn generate_pseudolegal(pos: &Position, movelist: &mut MoveList)
{
    todo!("Implement pseudolegal move generation")
}

fn is_legal(pos: &mut Position, mov: makemove::Move) -> bool
{
    let legal;
    let active = pos.state.active;
    let attacker = active.opposite();
    if mov.kind() == makemove::MoveKind::KingCastle
        || mov.kind() == makemove::MoveKind::QueenCastle
    {
        legal = is_legal_castle(pos, mov, active, attacker)
    }
    else
    {
        makemove::make(pos, mov);
        let king = pos.board.kings[active as usize];
        legal = !is_square_attacked(pos, king, attacker);
        makemove::unmake(pos);
    }
    legal
}

fn is_legal_castle(
    pos: &Position, 
    mov: makemove::Move, 
    active: utils::Color, 
    attacker: utils::Color
) -> bool
{
    match (active, mov.kind())
    {
        (utils::Color::White, makemove::MoveKind::KingCastle) =>
        {
            !is_square_attacked(pos, utils::Square::E1, attacker)
                && !is_square_attacked(pos, utils::Square::F1, attacker)
                && !is_square_attacked(pos, utils::Square::G1, attacker)
        }
        (utils::Color::White, makemove::MoveKind::QueenCastle) =>
        {
            !is_square_attacked(pos, utils::Square::E1, attacker)
                && !is_square_attacked(pos, utils::Square::D1, attacker)
                && !is_square_attacked(pos, utils::Square::C1, attacker)
        }
        (utils::Color::Black, makemove::MoveKind::KingCastle) =>
        {
            !is_square_attacked(pos, utils::Square::E8, attacker)
                && !is_square_attacked(pos, utils::Square::F8, attacker)
                && !is_square_attacked(pos, utils::Square::G8, attacker)
        }
        (utils::Color::Black, makemove::MoveKind::QueenCastle) =>
        {
            !is_square_attacked(pos, utils::Square::E8, attacker)
                && !is_square_attacked(pos, utils::Square::D8, attacker)
                && !is_square_attacked(pos, utils::Square::C8, attacker)
        }
        _ => true,
    }
}

fn is_square_attacked(
    pos: &Position, 
    square: utils::Square, 
    attacker: utils::Color
) -> bool
{
    todo!()
}
