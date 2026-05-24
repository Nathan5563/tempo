use crate::engine::utils::{
    CastlingRights,
    Color,
    MAX_NUM_MOVES,
    Piece,
    PieceKind,
    SQUARES,
    Square,
    enpassant_target,
};

use super::{
    Position,
    bitboard::BitBoard,
    makemove::{self, Move, MoveKind},
};

mod attacks;

const PROMOTIONS: [MoveKind; 4] = [
    MoveKind::PromoteQueen,
    MoveKind::PromoteRook,
    MoveKind::PromoteBishop,
    MoveKind::PromoteKnight,
];

const PROMOTION_CAPTURES: [MoveKind; 4] = [
    MoveKind::PromoteQueenCapture,
    MoveKind::PromoteRookCapture,
    MoveKind::PromoteBishopCapture,
    MoveKind::PromoteKnightCapture,
];

#[derive(Debug)]
pub struct MoveList
{
    arr: [Move; MAX_NUM_MOVES],
    len: usize
}

impl Default for MoveList
{
    fn default() -> Self
    {
        Self { arr: [Move::default(); MAX_NUM_MOVES], len: 0 }
    }
}

impl MoveList
{
    pub fn len(&self) -> usize
    {
        self.len
    }

    pub fn as_slice(&self) -> &[Move]
    {
        &self.arr[..self.len]
    }

    fn clear(&mut self)
    {
        self.len = 0;
    }

    fn push(&mut self, mov: Move)
    {
        self.arr[self.len] = mov;
        self.len += 1;
    }

    fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut(Move) -> bool,
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
    let active = pos.state.active;

    generate_pawns(pos, movelist, active);
    generate_piece_moves(
        pos,
        movelist,
        active,
        PieceKind::Knight,
        attacks::knight_attacks,
    );
    generate_sliders(
        pos,
        movelist,
        active,
        PieceKind::Bishop,
        attacks::bishop_attacks,
    );
    generate_sliders(
        pos,
        movelist,
        active,
        PieceKind::Rook,
        attacks::rook_attacks,
    );
    generate_sliders(
        pos,
        movelist,
        active,
        PieceKind::Queen,
        attacks::queen_attacks,
    );
    generate_piece_moves(
        pos,
        movelist,
        active,
        PieceKind::King,
        attacks::king_attacks,
    );
    generate_castles(pos, movelist, active);
}

fn generate_pawns(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
)
{
    let pawns = pos.board.colors[active as usize]
        & pos.board.pieces[PieceKind::Pawn as usize];

    for from in pawns
    {
        generate_pawn_pushes(pos, movelist, active, from);
        generate_pawn_captures(pos, movelist, active, from);
    }
}

fn generate_pawn_pushes(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
    from: Square,
)
{
    let from_index = from as usize;
    let Some(to) = pawn_step(active, from_index) else
    {
        return;
    };
    let to_square = SQUARES[to];

    if pos.board.mailbox[to_square].is_some()
    {
        return;
    }

    if is_promotion_square(active, to)
    {
        push_promotions(movelist, from, to_square, false);
        return;
    }

    movelist.push(Move::new(
        from,
        to_square,
        MoveKind::Quiet,
    ));

    if !is_pawn_start_square(active, from_index)
    {
        return;
    }

    let double_to = pawn_double_step(active, from_index);
    let double_to_square = SQUARES[double_to];

    if pos.board.mailbox[double_to_square].is_none()
    {
        movelist.push(Move::new(
            from,
            double_to_square,
            MoveKind::DoublePawnPush,
        ));
    }
}

fn generate_pawn_captures(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
    from: Square,
)
{
    let attackers = attacks::pawn_attacks(from, active);
    let enemy = active.opposite();
    let enemy_king = BitBoard::from_square(pos.board.kings[enemy as usize]);
    let capturable = pos.board.colors[enemy as usize] & !enemy_king;

    for to in attackers
    {
        if capturable.contains(to)
        {
            if is_promotion_square(active, to as usize)
            {
                push_promotions(movelist, from, to, true);
            }
            else
            {
                movelist.push(Move::new(
                    from,
                    to,
                    MoveKind::Capture,
                ));
            }
        }
        else if pos.state.enpassant == Some(to)
            && is_valid_enpassant_capture(pos, active, to)
        {
            movelist.push(Move::new(
                from,
                to,
                MoveKind::EnPassant,
            ));
        }
    }
}

fn generate_piece_moves(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
    kind: PieceKind,
    attacks: fn(Square) -> BitBoard,
)
{
    let pieces = pos.board.colors[active as usize]
        & pos.board.pieces[kind as usize];
    let own = pos.board.colors[active as usize];
    let enemy = active.opposite();
    let enemy_king = BitBoard::from_square(pos.board.kings[enemy as usize]);
    let capturable = pos.board.colors[enemy as usize] & !enemy_king;

    for from in pieces
    {
        let targets = attacks(from) & !own & !enemy_king;
        push_targets(movelist, from, targets, capturable);
    }
}

fn generate_sliders(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
    kind: PieceKind,
    attacks: fn(Square, BitBoard) -> BitBoard,
)
{
    let pieces = pos.board.colors[active as usize]
        & pos.board.pieces[kind as usize];
    let occupied = pos.board.occupied();
    let own = pos.board.colors[active as usize];
    let enemy = active.opposite();
    let enemy_king = BitBoard::from_square(pos.board.kings[enemy as usize]);
    let capturable = pos.board.colors[enemy as usize] & !enemy_king;

    for from in pieces
    {
        let targets = attacks(from, occupied) & !own & !enemy_king;
        push_targets(movelist, from, targets, capturable);
    }
}

fn push_targets(
    movelist: &mut MoveList,
    from: Square,
    targets: BitBoard,
    capturable: BitBoard,
)
{
    for to in targets
    {
        let kind = if capturable.contains(to)
        {
            MoveKind::Capture
        }
        else
        {
            MoveKind::Quiet
        };
        movelist.push(Move::new(from, to, kind));
    }
}

fn generate_castles(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
)
{
    match active
    {
        Color::White =>
        {
            try_castle(
                pos,
                movelist,
                active,
                CastlingRights::WHITE_KINGSIDE,
                Square::E1,
                Square::H1,
                Square::G1,
                &[Square::F1, Square::G1],
                MoveKind::KingCastle,
            );
            try_castle(
                pos,
                movelist,
                active,
                CastlingRights::WHITE_QUEENSIDE,
                Square::E1,
                Square::A1,
                Square::C1,
                &[Square::D1, Square::C1, Square::B1],
                MoveKind::QueenCastle,
            );
        }
        Color::Black =>
        {
            try_castle(
                pos,
                movelist,
                active,
                CastlingRights::BLACK_KINGSIDE,
                Square::E8,
                Square::H8,
                Square::G8,
                &[Square::F8, Square::G8],
                MoveKind::KingCastle,
            );
            try_castle(
                pos,
                movelist,
                active,
                CastlingRights::BLACK_QUEENSIDE,
                Square::E8,
                Square::A8,
                Square::C8,
                &[Square::D8, Square::C8, Square::B8],
                MoveKind::QueenCastle,
            );
        }
    }
}

fn try_castle(
    pos: &Position,
    movelist: &mut MoveList,
    active: Color,
    right: u8,
    king_square: Square,
    rook_square: Square,
    to: Square,
    empty_squares: &[Square],
    kind: MoveKind,
)
{
    if pos.state.castling.bits() & right == 0
    {
        return;
    }

    if can_castle(pos, active, king_square, rook_square, empty_squares)
    {
        movelist.push(Move::new(king_square, to, kind));
    }
}

fn can_castle(
    pos: &Position,
    active: Color,
    king_square: Square,
    rook_square: Square,
    empty_squares: &[Square],
) -> bool
{
    let king = Piece { color: active, kind: PieceKind::King };
    let rook = Piece { color: active, kind: PieceKind::Rook };

    pos.board.mailbox[king_square] == Some(king)
        && pos.board.mailbox[rook_square] == Some(rook)
        && empty_squares
            .iter()
            .all(|square| pos.board.mailbox[*square].is_none())
}

fn push_promotions(
    movelist: &mut MoveList,
    from: Square,
    to: Square,
    capture: bool,
)
{
    let kinds = if capture { PROMOTION_CAPTURES } else { PROMOTIONS };

    for kind in kinds
    {
        movelist.push(Move::new(from, to, kind));
    }
}

fn is_valid_enpassant_capture(
    pos: &Position,
    active: Color,
    to: Square,
) -> bool
{
    let capture_square = enpassant_target(active, to);
    let captured = Piece {
        color: active.opposite(),
        kind: PieceKind::Pawn,
    };

    pos.board.mailbox[capture_square] == Some(captured)
}

fn pawn_step(active: Color, from: usize) -> Option<usize>
{
    match active
    {
        Color::White if from <= Square::H7 as usize =>
        {
            Some(from + 8)
        }
        Color::Black if from >= Square::A2 as usize =>
        {
            Some(from - 8)
        }
        _ => None,
    }
}

fn pawn_double_step(active: Color, from: usize) -> usize
{
    match active
    {
        Color::White =>
        {
            from + 16
        }
        Color::Black =>
        {
            from - 16
        }
    }
}

fn is_pawn_start_square(active: Color, square: usize) -> bool
{
    match active
    {
        Color::White =>
        {
            square >= Square::A2 as usize
                && square <= Square::H2 as usize
        }
        Color::Black =>
        {
            square >= Square::A7 as usize
                && square <= Square::H7 as usize
        }
    }
}

fn is_promotion_square(active: Color, square: usize) -> bool
{
    match active
    {
        Color::White => square >= Square::A8 as usize,
        Color::Black => square <= Square::H1 as usize,
    }
}

fn is_legal(pos: &mut Position, mov: Move) -> bool
{
    let active = pos.state.active;
    let attacker = active.opposite();

    if matches!(
        mov.kind(),
        MoveKind::KingCastle | MoveKind::QueenCastle,
    )
    {
        return is_legal_castle(pos, mov, active, attacker);
    }

    makemove::make(pos, mov);
    let king = pos.board.kings[active as usize];
    let legal = !is_square_attacked(pos, king, attacker);
    makemove::unmake(pos);

    legal
}

fn is_legal_castle(
    pos: &Position,
    mov: Move,
    active: Color,
    attacker: Color,
) -> bool
{
    match (active, mov.kind())
    {
        (Color::White, MoveKind::KingCastle) =>
        {
            !is_square_attacked(pos, Square::E1, attacker)
                && !is_square_attacked(pos, Square::F1, attacker)
                && !is_square_attacked(pos, Square::G1, attacker)
        }
        (Color::White, MoveKind::QueenCastle) =>
        {
            !is_square_attacked(pos, Square::E1, attacker)
                && !is_square_attacked(pos, Square::D1, attacker)
                && !is_square_attacked(pos, Square::C1, attacker)
        }
        (Color::Black, MoveKind::KingCastle) =>
        {
            !is_square_attacked(pos, Square::E8, attacker)
                && !is_square_attacked(pos, Square::F8, attacker)
                && !is_square_attacked(pos, Square::G8, attacker)
        }
        (Color::Black, MoveKind::QueenCastle) =>
        {
            !is_square_attacked(pos, Square::E8, attacker)
                && !is_square_attacked(pos, Square::D8, attacker)
                && !is_square_attacked(pos, Square::C8, attacker)
        }
        _ => true,
    }
}

fn is_square_attacked(
    pos: &Position,
    square: Square,
    attacker: Color,
) -> bool
{
    let occupied = pos.board.occupied();
    let attackers = pos.board.colors[attacker as usize];
    let pawns = attackers & pos.board.pieces[PieceKind::Pawn as usize];
    let knights = attackers & pos.board.pieces[PieceKind::Knight as usize];
    let bishops = attackers & pos.board.pieces[PieceKind::Bishop as usize];
    let rooks = attackers & pos.board.pieces[PieceKind::Rook as usize];
    let queens = attackers & pos.board.pieces[PieceKind::Queen as usize];
    let king = attackers & pos.board.pieces[PieceKind::King as usize];

    !(attacks::pawn_attacks(square, attacker.opposite()) & pawns).is_empty()
        || !(attacks::knight_attacks(square) & knights).is_empty()
        || !(attacks::bishop_attacks(square, occupied) & (bishops | queens))
            .is_empty()
        || !(attacks::rook_attacks(square, occupied) & (rooks | queens))
            .is_empty()
        || !(attacks::king_attacks(square) & king).is_empty()
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn moves(fen: &str) -> MoveList
    {
        let mut pos = Position::new(fen).unwrap();
        let mut movelist = MoveList::default();
        generate(&mut pos, &mut movelist);
        movelist
    }

    fn has_move(
        movelist: &MoveList,
        from: Square,
        to: Square,
        kind: MoveKind,
    ) -> bool
    {
        movelist
            .as_slice()
            .iter()
            .any(|mov| mov.from() == from && mov.to() == to && mov.kind() == kind)
    }

    fn perft(pos: &mut Position, depth: u8) -> u64
    {
        if depth == 0
        {
            return 1;
        }

        let mut movelist = MoveList::default();
        generate(pos, &mut movelist);

        if depth == 1
        {
            return movelist.len() as u64;
        }

        let mut nodes = 0;
        for mov in movelist.as_slice()
        {
            makemove::make(pos, *mov);
            nodes += perft(pos, depth - 1);
            makemove::unmake(pos);
        }

        nodes
    }

    fn assert_perft(fen: &str, expected: &[(u8, u64)])
    {
        for (depth, nodes) in expected
        {
            let mut pos = Position::new(fen).unwrap();
            assert_eq!(perft(&mut pos, *depth), *nodes, "depth {}", depth);
        }
    }

    #[test]
    fn movelist_reports_len_and_slice()
    {
        let mut movelist = MoveList::default();
        let mov = Move::new(
            Square::E2,
            Square::E4,
            MoveKind::DoublePawnPush,
        );

        assert_eq!(movelist.len(), 0);

        movelist.push(mov);

        assert_eq!(movelist.len(), 1);
        assert_eq!(movelist.as_slice(), &[mov]);
    }

    #[test]
    fn pawns_generate_single_double_capture_enpassant_and_promotions()
    {
        let quiets = moves("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1");

        assert!(has_move(
            &quiets,
            Square::E2,
            Square::E3,
            MoveKind::Quiet,
        ));
        assert!(has_move(
            &quiets,
            Square::E2,
            Square::E4,
            MoveKind::DoublePawnPush,
        ));

        let captures = moves("4k2r/6P1/8/8/8/8/8/4K3 w - - 0 1");

        assert!(has_move(
            &captures,
            Square::G7,
            Square::G8,
            MoveKind::PromoteQueen,
        ));
        assert!(has_move(
            &captures,
            Square::G7,
            Square::H8,
            MoveKind::PromoteQueenCapture,
        ));

        let enpassant = moves("k7/8/8/3pP3/8/8/8/4K3 w - d6 0 1");

        assert!(has_move(
            &enpassant,
            Square::E5,
            Square::D6,
            MoveKind::EnPassant,
        ));
    }

    #[test]
    fn illegal_enpassant_discovered_check_is_filtered()
    {
        let movelist = moves("k3r3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");

        assert!(!has_move(
            &movelist,
            Square::E5,
            Square::D6,
            MoveKind::EnPassant,
        ));
    }

    #[test]
    fn castling_requires_rights_home_pieces_empty_path_and_safe_king_path()
    {
        let clear = moves("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

        assert!(has_move(
            &clear,
            Square::E1,
            Square::G1,
            MoveKind::KingCastle,
        ));
        assert!(has_move(
            &clear,
            Square::E1,
            Square::C1,
            MoveKind::QueenCastle,
        ));

        let blocked = moves("4k3/8/8/8/8/8/8/R2BK2R w KQ - 0 1");

        assert!(!has_move(
            &blocked,
            Square::E1,
            Square::C1,
            MoveKind::QueenCastle,
        ));
        assert!(has_move(
            &blocked,
            Square::E1,
            Square::G1,
            MoveKind::KingCastle,
        ));

        let through_check = moves("4kr2/8/8/8/8/8/8/R3K2R w KQ - 0 1");

        assert!(!has_move(
            &through_check,
            Square::E1,
            Square::G1,
            MoveKind::KingCastle,
        ));
        assert!(has_move(
            &through_check,
            Square::E1,
            Square::C1,
            MoveKind::QueenCastle,
        ));

        let black_clear = moves("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");

        assert!(has_move(
            &black_clear,
            Square::E8,
            Square::G8,
            MoveKind::KingCastle,
        ));
        assert!(has_move(
            &black_clear,
            Square::E8,
            Square::C8,
            MoveKind::QueenCastle,
        ));
    }

    #[test]
    fn legal_filter_rejects_check_violations_pins_and_king_adjacency()
    {
        let in_check = moves("k3r3/8/8/8/8/8/8/4K3 w - - 0 1");

        assert!(!has_move(
            &in_check,
            Square::E1,
            Square::E2,
            MoveKind::Quiet,
        ));

        let pinned = moves("k3r3/8/8/8/8/8/4R3/4K3 w - - 0 1");

        assert!(!has_move(
            &pinned,
            Square::E2,
            Square::D2,
            MoveKind::Quiet,
        ));

        let kings = moves("8/8/8/8/8/4k3/8/4K3 w - - 0 1");

        assert!(!has_move(
            &kings,
            Square::E1,
            Square::E2,
            MoveKind::Quiet,
        ));
    }

    #[test]
    fn pawn_helpers_handle_board_edges_and_start_ranks()
    {
        let mut pos = Position::new(
            "P3k3/8/8/8/8/8/8/4K3 w - - 0 1",
        ).unwrap();
        let mut movelist = MoveList::default();

        generate_pawn_pushes(
            &pos,
            &mut movelist,
            Color::White,
            Square::A8,
        );

        assert_eq!(movelist.len(), 0);
        assert_eq!(pawn_step(Color::White, Square::A8 as usize), None);
        assert_eq!(pawn_step(Color::Black, Square::H1 as usize), None);
        assert_eq!(
            pawn_double_step(Color::White, Square::E2 as usize),
            Square::E4 as usize
        );
        assert_eq!(
            pawn_double_step(Color::Black, Square::D7 as usize),
            Square::D5 as usize
        );
        assert!(is_pawn_start_square(
            Color::White,
            Square::A2 as usize,
        ));
        assert!(is_pawn_start_square(
            Color::Black,
            Square::H7 as usize,
        ));
        assert!(is_promotion_square(
            Color::White,
            Square::A8 as usize,
        ));
        assert!(is_promotion_square(
            Color::Black,
            Square::H1 as usize,
        ));

        pos.state.active = Color::Black;
        movelist.clear();
        generate_pawn_pushes(
            &pos,
            &mut movelist,
            Color::Black,
            Square::H1,
        );

        assert_eq!(movelist.len(), 0);
    }

    #[test]
    fn non_castle_move_is_legal_castle_fallback()
    {
        let pos = Position::new("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mov = Move::new(
            Square::E1,
            Square::E2,
            MoveKind::Quiet,
        );

        assert!(is_legal_castle(
            &pos,
            mov,
            Color::White,
            Color::Black,
        ));
    }

    #[test]
    fn perft_depth_zero_is_one()
    {
        let mut pos = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ).unwrap();

        assert_eq!(perft(&mut pos, 0), 1);
    }

    #[test]
    fn start_position_perft_matches_reference_counts()
    {
        assert_perft(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &[(1, 20), (2, 400), (3, 8902), (4, 197281)],
        );
    }

    #[test]
    fn kiwipete_perft_matches_reference_counts()
    {
        assert_perft(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            &[(1, 48), (2, 2039), (3, 97862)],
        );
    }

    #[test]
    fn promotion_heavy_perft_matches_reference_counts()
    {
        assert_perft(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            &[(1, 6), (2, 264), (3, 9467)],
        );
    }
}
