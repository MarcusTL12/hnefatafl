use std::collections::HashMap;

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

    fn zeroeval(self) -> f64 {
        self.0
            .nbits_iter::<2>()
            .take(121)
            .map(|x| match Piece::try_from(x).unwrap().try_into() {
                Ok(Faction::White) => 1.0,
                Ok(Faction::Black) => -1.0,
                _ => 0.0,
            })
            .sum()
    }

    // Naive minimax
    pub fn _minimax(self, turn: Faction, depth: u32) -> f64 {
        if depth == 0 {
            return self.zeroeval();
        }

        let evals_iter = self.all_moves(turn).map(|[from, to]| {
            let mut new_board = self;
            if new_board.do_move(from, to) {
                match turn {
                    Faction::Black => -f64::INFINITY,
                    Faction::White => f64::INFINITY,
                }
            } else {
                new_board._minimax(turn.other_faction(), depth - 1)
            }
        });

        match turn {
            Faction::Black => evals_iter.min_by(|a, b| a.total_cmp(b)),
            Faction::White => evals_iter.max_by(|a, b| a.total_cmp(b)),
        }
        .expect("No legal moves!")
    }

    pub fn alphabeta(
        self,
        turn: Faction,
        depth: u32,
        mut alpha: f64,
        mut beta: f64,
        trans_table: &mut HashMap<Self, (u32, f64)>,
    ) -> f64 {
        if depth == 0 {
            return self.zeroeval();
        }

        if let Some(&(d, s)) = trans_table.get(&self) {
            if d >= depth {
                return s;
            }
        }

        let it = self.all_moves(turn).map(|[from, to]| {
            let mut new_board = self;
            (new_board.do_move(from, to), new_board)
        });

        let score = match turn {
            Faction::White => {
                let mut score = -f64::INFINITY;

                for (won, board) in it {
                    score = score.max(if won {
                        f64::INFINITY
                    } else {
                        board.alphabeta(
                            turn.other_faction(),
                            depth - 1,
                            alpha,
                            beta,
                            trans_table,
                        )
                    });

                    alpha = alpha.max(score);

                    if score >= beta {
                        break;
                    }
                }

                score
            }
            Faction::Black => {
                let mut score = f64::INFINITY;

                for (won, board) in it {
                    score = score.min(if won {
                        -f64::INFINITY
                    } else {
                        board.alphabeta(
                            turn.other_faction(),
                            depth - 1,
                            alpha,
                            beta,
                            trans_table,
                        )
                    });

                    beta = beta.min(score);

                    if score <= alpha {
                        break;
                    }
                }

                score
            }
        };

        *trans_table.entry(self).or_insert((depth, score)) = (depth, score);

        score
    }

    pub fn best_move(
        self,
        turn: Faction,
        depth: u32,
        trans_table: &mut HashMap<Self, (u32, f64)>,
    ) -> ([[u16; 2]; 2], f64) {
        let mut alpha = -f64::INFINITY;
        let mut beta = f64::INFINITY;

        let mut best_move = None;
        let mut score = match turn {
            Faction::Black => f64::INFINITY,
            Faction::White => -f64::INFINITY,
        };

        for [from, to] in self.all_moves(turn) {
            let mut new_board = self;

            let local_score = if new_board.do_move(from, to) {
                match turn {
                    Faction::Black => -f64::INFINITY,
                    Faction::White => f64::INFINITY,
                }
            } else {
                new_board.alphabeta(
                    turn.other_faction(),
                    depth,
                    alpha,
                    beta,
                    trans_table,
                )
            };

            match turn {
                Faction::Black => {
                    if local_score < score {
                        best_move = Some([from, to]);
                        score = local_score;
                    }
                    beta = beta.min(score);
                }
                Faction::White => {
                    if local_score > score {
                        best_move = Some([from, to]);
                        score = local_score;
                    }
                    alpha = alpha.max(score);
                }
            }
        }

        (best_move.unwrap(), score)
    }
}
