use std::fmt::Display;

use bitarray::BitArray;
use crossterm::style::Stylize;

pub const W: usize = 11;

// pub const N: usize = (2 * W * W).div_ceil(usize::BITS as usize);
pub const M: usize = (W * W).div_ceil(usize::BITS as usize);

pub const TOWERS: BitArray<M> =
    BitArray([1152921504606848001, 72127962782105600]);

pub fn to_readable_coord([y, x]: [u16; 2]) -> String {
    format!("{}{}", b"ABCDEFGHIJK"[x as usize] as char, y + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Empty = 0,
    King,
    Black,
    White,
}

impl Piece {
    fn is_empty(self) -> bool {
        self == Piece::Empty
    }
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

impl From<[bool; 2]> for Piece {
    fn from(value: [bool; 2]) -> Self {
        match value {
            [false, false] => Self::Empty,
            [true, false] => Self::King,
            [false, true] => Self::Black,
            [true, true] => Self::White,
        }
    }
}

impl From<Piece> for [bool; 2] {
    fn from(value: Piece) -> Self {
        match value {
            Piece::Empty => [false, false],
            Piece::King => [true, false],
            Piece::Black => [false, true],
            Piece::White => [true, true],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Black = 0,
    White,
}

impl TryFrom<Piece> for Faction {
    type Error = &'static str;

    fn try_from(value: Piece) -> Result<Self, Self::Error> {
        match value {
            Piece::Empty => Err("Empty piece"),
            Piece::White | Piece::King => Ok(Self::White),
            Piece::Black => Ok(Self::Black),
        }
    }
}

impl Faction {
    pub fn other_faction(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardState(pub [BitArray<M>; 2]);

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.render(f, BitArray::new())
    }
}

fn _to_2d(i: usize) -> [u16; 2] {
    let y = (i / W) as u16;
    let x = (i % W) as u16;

    [y, x]
}

pub fn to_linind([y, x]: [u16; 2]) -> Option<usize> {
    if x as usize >= W || y as usize >= W {
        return None;
    }

    Some(x as usize + y as usize * W)
}

impl BoardState {
    pub fn new() -> Self {
        Self([BitArray::new(); 2])
    }

    pub fn standard_setup() -> Self {
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
            state.set_2d(pos, Piece::Black);
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
            state.set_2d(pos, Piece::White);
        }

        state.set_2d([5, 5], Piece::King);

        state
    }

    pub fn get(&self, i: usize) -> Piece {
        self.0.map(|x| x[i]).into()
    }

    pub fn set(&mut self, i: usize, val: Piece) {
        for (val, mat) in <[bool; 2]>::from(val).into_iter().zip(&mut self.0) {
            mat.set(i, val);
        }
    }

    pub fn get_2d(&self, [y, x]: [u16; 2]) -> Option<Piece> {
        Some(self.get(to_linind([y, x])?))
    }

    pub fn set_2d(&mut self, [y, x]: [u16; 2], val: Piece) {
        self.set(to_linind([y, x]).unwrap(), val)
    }

    pub fn _empties(self) -> BitArray<M> {
        !self.0[0] & !self.0[1]
    }

    pub fn whites(self) -> BitArray<M> {
        self.0[0]
    }

    pub fn blacks(self) -> BitArray<M> {
        !self.0[0] & self.0[1]
    }

    pub fn select_faction(self, turn: Faction) -> BitArray<M> {
        match turn {
            Faction::Black => self.blacks(),
            Faction::White => self.whites(),
        }
    }

    fn render(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        highlights: BitArray<M>,
    ) -> std::fmt::Result {
        writeln!(f, "┏━━━┓ ┏━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┓")?;
        writeln!(f, "┃ X ┃ ┃ A ┃ B ┃ C ┃ D ┃ E ┃ F ┃ G ┃ H ┃ I ┃ J ┃ K ┃")?;
        writeln!(f, "┗━━━┛ ┗━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┛")?;

        for (row, c) in self.0[0]
            .into_iter()
            .zip(self.0[1])
            .map(|(a, b)| Piece::from([a, b]))
            .take(121)
            .array_chunks::<11>()
            .enumerate()
        {
            writeln!(f, "{}", match row {
                0 => "┏━━━┓ ┏━━━┳━━━┯━━━┯━━━┯━━━┯━━━┯━━━┯━━━┯━━━┯━━━┳━━━┓",
                1 => "┣━━━┫ ┣━━━╃───┼───┼───┼───┼───┼───┼───┼───┼───╄━━━┫",
                10 => "┣━━━┫ ┣━━━╅───┼───┼───┼───┼───┼───┼───┼───┼───╆━━━┫",
                5 => "┣━━━┫ ┠───┼───┼───┼───┼───╆━━━╅───┼───┼───┼───┼───┨",
                6 => "┣━━━┫ ┠───┼───┼───┼───┼───╄━━━╃───┼───┼───┼───┼───┨",
                _ => "┣━━━┫ ┠───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┨",
            })?;

            for (col, x) in c.into_iter().enumerate() {
                match [row, col] {
                    [0, 1] | [0, 10] | [10, 1] | [10, 10] | [5, 5] | [5, 6] => {
                        write!(f, "┃")?
                    }
                    [_, 0] => write!(f, "┃{:2} ┃ ┃", row + 1)?,
                    _ => write!(f, "│")?,
                };

                let highlight = highlights
                    .get(to_linind([row as u16, col as u16]).unwrap())
                    .unwrap();

                if highlight {
                    write!(f, "{}", "▐".dark_grey())?;
                } else {
                    write!(f, " ")?;
                }

                let piece = match x {
                    Piece::Empty => " ",
                    Piece::King => "ᛝ",
                    Piece::Black => "◯",
                    Piece::White => "⬤",
                }
                .bold();

                if highlight {
                    write!(f, "{}", piece.on_dark_grey())?;
                } else {
                    write!(f, "{}", piece)?;
                }

                if highlight {
                    write!(f, "{}", "▌".dark_grey())?;
                } else {
                    write!(f, " ")?;
                }
            }

            writeln!(f, "┃")?;
        }

        writeln!(f, "┗━━━┛ ┗━━━┻━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┷━━━┻━━━┛")
    }

    pub fn moves_from(&self, [y, x]: [u16; 2]) -> BitArray<M> {
        let mut moves = BitArray::new();

        for x in x + 1..11 {
            if self.get_2d([y, x]).unwrap().is_empty() {
                moves.set(to_linind([y, x]).unwrap(), true);
            } else {
                break;
            }
        }

        for x in (0..x).rev() {
            if self.get_2d([y, x]).unwrap().is_empty() {
                moves.set(to_linind([y, x]).unwrap(), true);
            } else {
                break;
            }
        }

        for y in y + 1..11 {
            if self.get_2d([y, x]).unwrap().is_empty() {
                moves.set(to_linind([y, x]).unwrap(), true);
            } else {
                break;
            }
        }

        for y in (0..y).rev() {
            if self.get_2d([y, x]).unwrap().is_empty() {
                moves.set(to_linind([y, x]).unwrap(), true);
            } else {
                break;
            }
        }

        if self.get_2d([y, x]) == Some(Piece::King) {
            moves
        } else {
            moves & !TOWERS
        }
    }

    pub fn do_move(&mut self, from: [u16; 2], to: [u16; 2]) -> bool {
        let piece = self.get_2d(from).unwrap();
        self.set_2d(from, Piece::Empty);
        self.set_2d(to, piece);

        if piece == Piece::King
            && TOWERS[to_linind(to).unwrap()]
            && to != [5, 5]
        {
            return true;
        }

        let cur_faction: Faction = piece.try_into().unwrap();

        let dirs = [[1, 0], [0, 1], [-1, 0], [0, -1]];

        // Capture detection:
        for d in dirs {
            let y = (to[0] as isize + d[0]) as u16;
            let x = (to[1] as isize + d[1]) as u16;

            if let Some(p) = self.get_2d([y, x]) {
                if p.try_into() == Ok(cur_faction.other_faction()) {
                    if p != Piece::King {
                        let ny = (y as isize + d[0]) as u16;
                        let nx = (x as isize + d[1]) as u16;

                        if let Some(i) = to_linind([ny, nx]) {
                            if TOWERS[i] && self.get(i) == Piece::Empty
                                || self.get(i).try_into() == Ok(cur_faction)
                            {
                                self.set_2d([y, x], Piece::Empty);
                            }
                        }
                    } else if dirs.into_iter().all(|d| {
                        let ny = (y as isize + d[0]) as u16;
                        let nx = (x as isize + d[1]) as u16;

                        if let Some(i) = to_linind([ny, nx]) {
                            TOWERS[i] && self.get(i) == Piece::Empty
                                || self.get(i).try_into() == Ok(cur_faction)
                        } else {
                            false
                        }
                    }) {
                        return true;
                    }
                }
            }
        }

        // Test for the following special moves:
        // Surrounded
        // Exit Fort
        // Shield wall

        false
    }
}

pub struct HighlightedBoardState(pub BoardState, pub BitArray<M>);

impl Display for HighlightedBoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f, self.1)
    }
}
