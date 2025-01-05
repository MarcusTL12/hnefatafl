#![feature(iter_array_chunks)]

use std::{fmt::Display, io::stdout, pin};

use bitarray::BitArray;
use crossterm::{
    ExecutableCommand, cursor,
    event::{
        self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    style::Stylize,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
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
}

struct HighlightedBoardState {
    pub state: BoardState,
    pub highlights: Vec<[u16; 2]>,
}

impl Display for HighlightedBoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.render(f, &self.highlights)
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
    let state = BoardState::standard_setup();

    let mut hstate = HighlightedBoardState {
        state,
        highlights: Vec::new(),
    };

    let mut out = stdout();

    execute!(
        out,
        EnableMouseCapture,
        EnterAlternateScreen,
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    println!("{hstate}");

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
                execute!(
                    out,
                    cursor::MoveUp(1),
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    cursor::MoveUp(1),
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    cursor::MoveTo(0, 0),
                )
                .unwrap();

                let coord = screen_coord_to_game_coord([row, column]);
                if let Some(coord) = coord {
                    hstate.highlights.push(coord);
                }
                println!("{hstate}");
                println!("You pressed on coord: {row}, {column}");
                println!("Ingame coord: {coord:?}");
            }
            _ => {}
        }
    }

    stdout().execute(LeaveAlternateScreen).unwrap();
}
