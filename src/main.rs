#![feature(iter_array_chunks)]

use std::{fmt::Display, io::stdout};

use bitarray::BitArray;
use crossterm::{
    ExecutableCommand, cursor,
    event::{
        self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    style::Stylize,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Faction {
    White,
    Black,
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
    fn other_faction(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BoardState(pub BitArray<N>);

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.render(f, &[])
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

    fn get(&self, i: usize) -> Piece {
        self.0.get_nbit::<2>(i).try_into().unwrap()
    }

    fn set(&mut self, i: usize, val: Piece) {
        self.0.set_nbit::<2>(i, val as usize)
    }

    fn get_2d(&self, [y, x]: [u16; 2]) -> Piece {
        self.get(x as usize + y as usize * W)
    }

    fn set_2d(&mut self, [y, x]: [u16; 2], val: Piece) {
        self.set(x as usize + y as usize * W, val)
    }

    fn render(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        highlights: &[[u16; 2]],
    ) -> std::fmt::Result {
        writeln!(f, "      ┏━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┓")?;
        writeln!(f, "      ┃ A ┃ B ┃ C ┃ D ┃ E ┃ F ┃ G ┃ H ┃ I ┃ J ┃ K ┃")?;
        writeln!(f, "      ┗━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┛")?;

        for (row, c) in self
            .0
            .nbits_iter::<2>()
            .map(|x| Piece::try_from(x).unwrap())
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

                let highlight = highlights.contains(&[row as u16, col as u16]);

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

    fn highlight(self, highlights: &[[u16; 2]]) -> HighlightedBoardState<'_> {
        HighlightedBoardState(self, highlights)
    }
}

struct HighlightedBoardState<'a>(BoardState, &'a [[u16; 2]]);

impl Display for HighlightedBoardState<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f, self.1)
    }
}

fn screen_coord_to_game_coord([y, x]: [u16; 2]) -> Option<[u16; 2]> {
    let row = y.checked_sub(4)? / 2;
    if y <= 24 && y % 2 != 0 {
        return None;
    }

    let col = x.checked_sub(7)? / 4;
    if col < 11 && x % 4 == 2 {
        return None;
    }

    Some([row, col])
}

fn main() {
    let board = BoardState::standard_setup();

    let mut selected = None;
    let turn = Faction::Black;

    let mut out = stdout();

    execute!(
        out,
        EnableMouseCapture,
        EnterAlternateScreen,
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    println!("{board}");

    while let Ok(x) = event::read() {
        match x {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: _,
                kind: _,
                state: _,
            }) => break,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                let Some(coord) = screen_coord_to_game_coord([row, column])
                else {
                    continue;
                };

                if selected.is_none()
                    && board.get_2d(coord).try_into() == Ok(turn)
                {
                    selected = Some(coord);

                    execute!(out, cursor::MoveTo(0, 0)).unwrap();
                    println!("{}", board.highlight(&[coord]));
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                column: _,
                row: _,
                modifiers: KeyModifiers::NONE,
            }) => {
                if selected.is_some() {
                    selected = None;

                    execute!(out, cursor::MoveTo(0, 0)).unwrap();
                    println!("{board}");
                }
            }
            _ => {}
        }
    }

    stdout().execute(LeaveAlternateScreen).unwrap();
}
