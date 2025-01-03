#![feature(iter_array_chunks)]

use std::fmt::Display;

use bitarray::BitArray;

const W: usize = 11;

const N: usize = 256usize.div_ceil(usize::BITS as usize);

#[derive(Debug, Clone, Copy)]
enum Piece {
    Empty = 0,
    King,
    Black,
    White,
}

impl TryFrom<usize> for Piece {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::King),
            2 => Ok(Self::Black),
            3 => Ok(Self::White),
            _ => Err("Invalid Piece Code"),
        }
    }
}

struct BoardState(pub BitArray<N>);

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (row, c) in self
            .0
            .into_iter()
            .array_chunks::<2>()
            .take(121)
            .array_chunks::<11>()
            .enumerate()
        {
            writeln!(f, "{}", match row {
                0 => "┏━━━┳━━━┯━━━┯━━━┯━━━┯━━━┯━━━┯━━━┯━━━┯━━━┳━━━┓",
                1 => "┣━━━╃───┼───┼───┼───┼───┼───┼───┼───┼───╄━━━┫",
                10 => "┣━━━╅───┼───┼───┼───┼───┼───┼───┼───┼───╆━━━┫",
                5 => "┠───┼───┼───┼───┼───╆━━━╅───┼───┼───┼───┼───┨",
                6 => "┠───┼───┼───┼───┼───╄━━━╃───┼───┼───┼───┼───┨",
                _ => "┠───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┨",
            })?;

            for (col, x) in c.into_iter().enumerate() {
                write!(f, "{}", match [row, col] {
                    [_, 0]
                    | [0, 1]
                    | [0, 10]
                    | [10, 1]
                    | [10, 10]
                    | [5, 5]
                    | [5, 6] => "┃",
                    _ => "│",
                })?;

                write!(f, "{}", match x {
                    [false, false] => "   ",
                    [true, false] => " \x1b[1mᛝ\x1b[0m ",
                    [false, true] => " ◯ ",
                    [true, true] => " ⬤ ",
                })?;
            }

            writeln!(f, "┃")?;
        }

        writeln!(f, "┗━━━┻━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┻━━━┛")
    }
}

impl BoardState {
    fn new() -> Self {
        Self(BitArray::new())
    }

    fn standard_setup() -> Self {
        let mut state = Self::new();

        let black_pieces = [
            [0, 3],
            [0, 4],
            [0, 5],
            [0, 6],
            [0, 7],
            [1, 5],
            [3, 0],
            [4, 0],
            [5, 0],
            [6, 0],
            [7, 0],
            [5, 1],
            [10, 3],
            [10, 4],
            [10, 5],
            [10, 6],
            [10, 7],
            [9, 5],
            [3, 10],
            [4, 10],
            [5, 10],
            [6, 10],
            [7, 10],
            [5, 9],
        ];

        for pos in black_pieces {
            state.set_xy(pos, Piece::Black);
        }

        let white_pieces = [
            [3, 5],
            [4, 4],
            [4, 5],
            [4, 6],
            [5, 3],
            [5, 4],
            [5, 6],
            [5, 7],
            [6, 4],
            [6, 5],
            [6, 6],
            [7, 5],
        ];

        for pos in white_pieces {
            state.set_xy(pos, Piece::White);
        }

        state.set_xy([5, 5], Piece::King);

        state
    }

    fn get(&self, i: usize) -> Piece {
        self.0.get_nbit::<2>(i).try_into().unwrap()
    }

    fn set(&mut self, i: usize, val: Piece) {
        self.0.set_nbit::<2>(i, val as usize)
    }

    fn get_xy(&self, [y, x]: [usize; 2]) -> Piece {
        self.get(x + y * W)
    }

    fn set_xy(&mut self, [y, x]: [usize; 2], val: Piece) {
        self.set(x + y * W, val)
    }
}

fn main() {
    let state = BoardState::standard_setup();

    println!("{state}");
}
