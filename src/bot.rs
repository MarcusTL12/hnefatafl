use bitarray::BitArray;

use crate::board::{BoardState, Faction, Piece};

impl BoardState {
    pub fn all_moves(
        &self,
        turn: Faction,
    ) -> impl Iterator<Item = BitArray<2>> {
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
                (x.try_into() == Ok(turn)).then_some(self.moves_from(coord))
            })
    }
}
