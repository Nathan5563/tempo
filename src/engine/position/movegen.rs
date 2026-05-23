use super::{Position, bitboard::BitBoard, makemove, super::utils};

mod attacks;

const PROMOTIONS: [makemove::MoveKind; 4] = [
    makemove::MoveKind::PromoteQueen,
    makemove::MoveKind::PromoteRook,
    makemove::MoveKind::PromoteBishop,
    makemove::MoveKind::PromoteKnight,
];

const PROMOTION_CAPTURES: [makemove::MoveKind; 4] = [
    makemove::MoveKind::PromoteQueenCapture,
    makemove::MoveKind::PromoteRookCapture,
    makemove::MoveKind::PromoteBishopCapture,
    makemove::MoveKind::PromoteKnightCapture,
];

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
    pub fn len(&self) -> usize
    {
        self.len
    }

    pub fn as_slice(&self) -> &[makemove::Move]
    {
        &self.arr[..self.len]
    }

    fn clear(&mut self)
    {
        self.len = 0;
    }

    fn push(&mut self, mov: makemove::Move)
    {
        self.arr[self.len] = mov;
        self.len += 1;
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
    let active = pos.state.active;

    generate_pawns(pos, movelist, active);
    generate_piece_moves(
        pos,
        movelist,
        active,
        utils::PieceKind::Knight,
        attacks::knight_attacks,
    );
    generate_sliders(
        pos,
        movelist,
        active,
        utils::PieceKind::Bishop,
        attacks::bishop_attacks,
    );
    generate_sliders(
        pos,
        movelist,
        active,
        utils::PieceKind::Rook,
        attacks::rook_attacks,
    );
    generate_sliders(
        pos,
        movelist,
        active,
        utils::PieceKind::Queen,
        attacks::queen_attacks,
    );
    generate_piece_moves(
        pos,
        movelist,
        active,
        utils::PieceKind::King,
        attacks::king_attacks,
    );
    generate_castles(pos, movelist, active);
}

fn generate_pawns(
    pos: &Position,
    movelist: &mut MoveList,
    active: utils::Color,
)
{
    let pawns = pos.board.colors[active as usize]
        & pos.board.pieces[utils::PieceKind::Pawn as usize];

    for from in pawns
    {
        generate_pawn_pushes(pos, movelist, active, from);
        generate_pawn_captures(pos, movelist, active, from);
    }
}

fn generate_pawn_pushes(
    pos: &Position,
    movelist: &mut MoveList,
    active: utils::Color,
    from: utils::Square,
)
{
    let from_index = from as usize;
    let Some(to) = pawn_step(active, from_index) else
    {
        return;
    };
    let to_square = utils::SQUARES[to];

    if pos.board.mailbox[to_square].is_some()
    {
        return;
    }

    if is_promotion_square(active, to)
    {
        push_promotions(movelist, from, to_square, false);
        return;
    }

    movelist.push(makemove::Move::new(
        from,
        to_square,
        makemove::MoveKind::Quiet,
    ));

    if !is_pawn_start_square(active, from_index)
    {
        return;
    }

    let double_to = pawn_double_step(active, from_index);
    let double_to_square = utils::SQUARES[double_to];

    if pos.board.mailbox[double_to_square].is_none()
    {
        movelist.push(makemove::Move::new(
            from,
            double_to_square,
            makemove::MoveKind::DoublePawnPush,
        ));
    }
}

fn generate_pawn_captures(
    pos: &Position,
    movelist: &mut MoveList,
    active: utils::Color,
    from: utils::Square,
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
                movelist.push(makemove::Move::new(
                    from,
                    to,
                    makemove::MoveKind::Capture,
                ));
            }
        }
        else if pos.state.enpassant == Some(to)
            && is_valid_enpassant_capture(pos, active, to)
        {
            movelist.push(makemove::Move::new(
                from,
                to,
                makemove::MoveKind::EnPassant,
            ));
        }
    }
}

fn generate_piece_moves(
    pos: &Position,
    movelist: &mut MoveList,
    active: utils::Color,
    kind: utils::PieceKind,
    attacks: fn(utils::Square) -> BitBoard,
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
    active: utils::Color,
    kind: utils::PieceKind,
    attacks: fn(utils::Square, BitBoard) -> BitBoard,
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
    from: utils::Square,
    targets: BitBoard,
    capturable: BitBoard,
)
{
    for to in targets
    {
        let kind = if capturable.contains(to)
        {
            makemove::MoveKind::Capture
        }
        else
        {
            makemove::MoveKind::Quiet
        };
        movelist.push(makemove::Move::new(from, to, kind));
    }
}

fn generate_castles(
    pos: &Position,
    movelist: &mut MoveList,
    active: utils::Color,
)
{
    match active
    {
        utils::Color::White =>
        {
            try_castle(
                pos,
                movelist,
                active,
                utils::CastlingRights::WHITE_KINGSIDE,
                utils::Square::E1,
                utils::Square::H1,
                utils::Square::G1,
                &[utils::Square::F1, utils::Square::G1],
                makemove::MoveKind::KingCastle,
            );
            try_castle(
                pos,
                movelist,
                active,
                utils::CastlingRights::WHITE_QUEENSIDE,
                utils::Square::E1,
                utils::Square::A1,
                utils::Square::C1,
                &[utils::Square::D1, utils::Square::C1, utils::Square::B1],
                makemove::MoveKind::QueenCastle,
            );
        }
        utils::Color::Black =>
        {
            try_castle(
                pos,
                movelist,
                active,
                utils::CastlingRights::BLACK_KINGSIDE,
                utils::Square::E8,
                utils::Square::H8,
                utils::Square::G8,
                &[utils::Square::F8, utils::Square::G8],
                makemove::MoveKind::KingCastle,
            );
            try_castle(
                pos,
                movelist,
                active,
                utils::CastlingRights::BLACK_QUEENSIDE,
                utils::Square::E8,
                utils::Square::A8,
                utils::Square::C8,
                &[utils::Square::D8, utils::Square::C8, utils::Square::B8],
                makemove::MoveKind::QueenCastle,
            );
        }
    }
}

fn try_castle(
    pos: &Position,
    movelist: &mut MoveList,
    active: utils::Color,
    right: u8,
    king_square: utils::Square,
    rook_square: utils::Square,
    to: utils::Square,
    empty_squares: &[utils::Square],
    kind: makemove::MoveKind,
)
{
    if pos.state.castling.bits() & right == 0
    {
        return;
    }

    if can_castle(pos, active, king_square, rook_square, empty_squares)
    {
        movelist.push(makemove::Move::new(king_square, to, kind));
    }
}

fn can_castle(
    pos: &Position,
    active: utils::Color,
    king_square: utils::Square,
    rook_square: utils::Square,
    empty_squares: &[utils::Square],
) -> bool
{
    let king = utils::Piece { color: active, kind: utils::PieceKind::King };
    let rook = utils::Piece { color: active, kind: utils::PieceKind::Rook };

    pos.board.mailbox[king_square] == Some(king)
        && pos.board.mailbox[rook_square] == Some(rook)
        && empty_squares
            .iter()
            .all(|square| pos.board.mailbox[*square].is_none())
}

fn push_promotions(
    movelist: &mut MoveList,
    from: utils::Square,
    to: utils::Square,
    capture: bool,
)
{
    let kinds = if capture { PROMOTION_CAPTURES } else { PROMOTIONS };

    for kind in kinds
    {
        movelist.push(makemove::Move::new(from, to, kind));
    }
}

fn is_valid_enpassant_capture(
    pos: &Position,
    active: utils::Color,
    to: utils::Square,
) -> bool
{
    let capture_square = utils::enpassant_target(active, to);
    let captured = utils::Piece {
        color: active.opposite(),
        kind: utils::PieceKind::Pawn,
    };

    pos.board.mailbox[capture_square] == Some(captured)
}

fn pawn_step(active: utils::Color, from: usize) -> Option<usize>
{
    match active
    {
        utils::Color::White if from <= utils::Square::H7 as usize =>
        {
            Some(from + 8)
        }
        utils::Color::Black if from >= utils::Square::A2 as usize =>
        {
            Some(from - 8)
        }
        _ => None,
    }
}

fn pawn_double_step(active: utils::Color, from: usize) -> usize
{
    match active
    {
        utils::Color::White =>
        {
            from + 16
        }
        utils::Color::Black =>
        {
            from - 16
        }
    }
}

fn is_pawn_start_square(active: utils::Color, square: usize) -> bool
{
    match active
    {
        utils::Color::White =>
        {
            square >= utils::Square::A2 as usize
                && square <= utils::Square::H2 as usize
        }
        utils::Color::Black =>
        {
            square >= utils::Square::A7 as usize
                && square <= utils::Square::H7 as usize
        }
    }
}

fn is_promotion_square(active: utils::Color, square: usize) -> bool
{
    match active
    {
        utils::Color::White => square >= utils::Square::A8 as usize,
        utils::Color::Black => square <= utils::Square::H1 as usize,
    }
}

fn is_legal(pos: &mut Position, mov: makemove::Move) -> bool
{
    let active = pos.state.active;
    let attacker = active.opposite();

    if matches!(
        mov.kind(),
        makemove::MoveKind::KingCastle | makemove::MoveKind::QueenCastle,
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
    mov: makemove::Move,
    active: utils::Color,
    attacker: utils::Color,
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
    attacker: utils::Color,
) -> bool
{
    let occupied = pos.board.occupied();
    let attackers = pos.board.colors[attacker as usize];
    let pawns = attackers & pos.board.pieces[utils::PieceKind::Pawn as usize];
    let knights = attackers & pos.board.pieces[utils::PieceKind::Knight as usize];
    let bishops = attackers & pos.board.pieces[utils::PieceKind::Bishop as usize];
    let rooks = attackers & pos.board.pieces[utils::PieceKind::Rook as usize];
    let queens = attackers & pos.board.pieces[utils::PieceKind::Queen as usize];
    let king = attackers & pos.board.pieces[utils::PieceKind::King as usize];

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
        from: utils::Square,
        to: utils::Square,
        kind: makemove::MoveKind,
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
        let mov = makemove::Move::new(
            utils::Square::E2,
            utils::Square::E4,
            makemove::MoveKind::DoublePawnPush,
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
            utils::Square::E2,
            utils::Square::E3,
            makemove::MoveKind::Quiet,
        ));
        assert!(has_move(
            &quiets,
            utils::Square::E2,
            utils::Square::E4,
            makemove::MoveKind::DoublePawnPush,
        ));

        let captures = moves("4k2r/6P1/8/8/8/8/8/4K3 w - - 0 1");

        assert!(has_move(
            &captures,
            utils::Square::G7,
            utils::Square::G8,
            makemove::MoveKind::PromoteQueen,
        ));
        assert!(has_move(
            &captures,
            utils::Square::G7,
            utils::Square::H8,
            makemove::MoveKind::PromoteQueenCapture,
        ));

        let enpassant = moves("k7/8/8/3pP3/8/8/8/4K3 w - d6 0 1");

        assert!(has_move(
            &enpassant,
            utils::Square::E5,
            utils::Square::D6,
            makemove::MoveKind::EnPassant,
        ));
    }

    #[test]
    fn illegal_enpassant_discovered_check_is_filtered()
    {
        let movelist = moves("k3r3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");

        assert!(!has_move(
            &movelist,
            utils::Square::E5,
            utils::Square::D6,
            makemove::MoveKind::EnPassant,
        ));
    }

    #[test]
    fn castling_requires_rights_home_pieces_empty_path_and_safe_king_path()
    {
        let clear = moves("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

        assert!(has_move(
            &clear,
            utils::Square::E1,
            utils::Square::G1,
            makemove::MoveKind::KingCastle,
        ));
        assert!(has_move(
            &clear,
            utils::Square::E1,
            utils::Square::C1,
            makemove::MoveKind::QueenCastle,
        ));

        let blocked = moves("4k3/8/8/8/8/8/8/R2BK2R w KQ - 0 1");

        assert!(!has_move(
            &blocked,
            utils::Square::E1,
            utils::Square::C1,
            makemove::MoveKind::QueenCastle,
        ));
        assert!(has_move(
            &blocked,
            utils::Square::E1,
            utils::Square::G1,
            makemove::MoveKind::KingCastle,
        ));

        let through_check = moves("4kr2/8/8/8/8/8/8/R3K2R w KQ - 0 1");

        assert!(!has_move(
            &through_check,
            utils::Square::E1,
            utils::Square::G1,
            makemove::MoveKind::KingCastle,
        ));
        assert!(has_move(
            &through_check,
            utils::Square::E1,
            utils::Square::C1,
            makemove::MoveKind::QueenCastle,
        ));

        let black_clear = moves("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1");

        assert!(has_move(
            &black_clear,
            utils::Square::E8,
            utils::Square::G8,
            makemove::MoveKind::KingCastle,
        ));
        assert!(has_move(
            &black_clear,
            utils::Square::E8,
            utils::Square::C8,
            makemove::MoveKind::QueenCastle,
        ));
    }

    #[test]
    fn legal_filter_rejects_check_violations_pins_and_king_adjacency()
    {
        let in_check = moves("k3r3/8/8/8/8/8/8/4K3 w - - 0 1");

        assert!(!has_move(
            &in_check,
            utils::Square::E1,
            utils::Square::E2,
            makemove::MoveKind::Quiet,
        ));

        let pinned = moves("k3r3/8/8/8/8/8/4R3/4K3 w - - 0 1");

        assert!(!has_move(
            &pinned,
            utils::Square::E2,
            utils::Square::D2,
            makemove::MoveKind::Quiet,
        ));

        let kings = moves("8/8/8/8/8/4k3/8/4K3 w - - 0 1");

        assert!(!has_move(
            &kings,
            utils::Square::E1,
            utils::Square::E2,
            makemove::MoveKind::Quiet,
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
            utils::Color::White,
            utils::Square::A8,
        );

        assert_eq!(movelist.len(), 0);
        assert_eq!(pawn_step(utils::Color::White, utils::Square::A8 as usize), None);
        assert_eq!(pawn_step(utils::Color::Black, utils::Square::H1 as usize), None);
        assert_eq!(
            pawn_double_step(utils::Color::White, utils::Square::E2 as usize),
            utils::Square::E4 as usize
        );
        assert_eq!(
            pawn_double_step(utils::Color::Black, utils::Square::D7 as usize),
            utils::Square::D5 as usize
        );
        assert!(is_pawn_start_square(
            utils::Color::White,
            utils::Square::A2 as usize,
        ));
        assert!(is_pawn_start_square(
            utils::Color::Black,
            utils::Square::H7 as usize,
        ));
        assert!(is_promotion_square(
            utils::Color::White,
            utils::Square::A8 as usize,
        ));
        assert!(is_promotion_square(
            utils::Color::Black,
            utils::Square::H1 as usize,
        ));

        pos.state.active = utils::Color::Black;
        movelist.clear();
        generate_pawn_pushes(
            &pos,
            &mut movelist,
            utils::Color::Black,
            utils::Square::H1,
        );

        assert_eq!(movelist.len(), 0);
    }

    #[test]
    fn non_castle_move_is_legal_castle_fallback()
    {
        let pos = Position::new("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mov = makemove::Move::new(
            utils::Square::E1,
            utils::Square::E2,
            makemove::MoveKind::Quiet,
        );

        assert!(is_legal_castle(
            &pos,
            mov,
            utils::Color::White,
            utils::Color::Black,
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
