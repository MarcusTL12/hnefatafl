use crate::board::{BoardState, Faction, Piece, W};

impl BoardState {
    pub fn all_moves(
        &self,
        turn: Faction,
    ) -> impl Iterator<Item = [[u16; 2]; 2]> {
        self.0
            .nbits_iter::<2>()
            .map(|x| Piece::try_from(x).unwrap())
            .take(121)
            .array_chunks::<11>()
            .enumerate()
            .flat_map(|(i, row)| {
                row.into_iter()
                    .enumerate()
                    .map(move |(j, x)| ([i as u16, j as u16], x))
            })
            .filter_map(move |(coord, x)| {
                (x.try_into() == Ok(turn))
                    .then_some((coord, self.moves_from(coord)))
            })
            .flat_map(|(from, legal_moves)| {
                legal_moves
                    .trues_iter()
                    .map(|i| [(i / W) as u16, (i % W) as u16])
                    .map(move |to| [from, to])
            })
    }

    pub fn eval(self, turn: Faction, depth: u32) -> f64 {
        if depth == 0 {
            return self
                .0
                .nbits_iter::<2>()
                .take(121)
                .map(|x| match Piece::try_from(x).unwrap().try_into() {
                    Ok(Faction::White) => 1.0,
                    Ok(Faction::Black) => -1.0,
                    _ => 0.0,
                })
                .sum();
        }

        let evals_iter = self.all_moves(turn).map(|[from, to]| {
            let mut new_board = self;
            if new_board.do_move(from, to) {
                match turn {
                    Faction::Black => -f64::INFINITY,
                    Faction::White => f64::INFINITY,
                }
            } else {
                new_board.eval(turn.other_faction(), depth - 1)
            }
        });

        match turn {
            Faction::Black => evals_iter.min_by(|a, b| a.total_cmp(b)),
            Faction::White => evals_iter.max_by(|a, b| a.total_cmp(b)),
        }
        .expect("No legal moves!")
    }

    pub fn best_move(self, turn: Faction, depth: u32) -> ([[u16; 2]; 2], f64) {
        let evals_iter = self.all_moves(turn).map(|[from, to]| {
            let mut new_board = self;
            (
                [from, to],
                if new_board.do_move(from, to) {
                    match turn {
                        Faction::Black => -f64::INFINITY,
                        Faction::White => f64::INFINITY,
                    }
                } else {
                    new_board.eval(turn.other_faction(), depth)
                },
            )
        });

        match turn {
            Faction::Black => {
                evals_iter.min_by(|(_, a), (_, b)| a.total_cmp(b))
            }
            Faction::White => {
                evals_iter.max_by(|(_, a), (_, b)| a.total_cmp(b))
            }
        }
        .expect("No best move!")
    }
}
