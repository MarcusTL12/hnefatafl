use std::{
    io::{Stdout, stdout},
    time::Instant,
};

use bitarray::BitArray;
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
        KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
    execute,
    style::Stylize,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::board::{
    self, BoardState, Faction, HighlightedBoardState, to_linind,
    to_readable_coord,
};

fn screen_coord_to_game_coord([y, x]: [u16; 2]) -> Option<[u16; 2]> {
    let row = y.checked_sub(4)? / 2;
    if y > 24 || y % 2 != 0 {
        return None;
    }

    let col = x.checked_sub(7)? / 4;
    if col >= 11 || x % 4 == 2 {
        return None;
    }

    Some([row, col])
}

pub struct GameState {
    out: Stdout,
    board: BoardState,
    selected: Option<[u16; 2]>,
    legal_moves: BitArray<{ board::M }>,
    turn: Faction,
    history: Vec<BoardState>,
    looking_back_at: Option<usize>,
}

impl GameState {
    pub fn new() -> Self {
        let board = BoardState::standard_setup();

        Self {
            out: stdout(),
            board,
            selected: None,
            legal_moves: BitArray::new(),
            turn: Faction::Black,
            history: Vec::new(),
            looking_back_at: None,
        }
    }

    fn render(&mut self) {
        execute!(self.out, cursor::MoveTo(0, 0)).unwrap();

        if let Some(i) = self.looking_back_at {
            print!("{}", self.history[i]);

            let turn = [Faction::Black, Faction::White][i % 2];

            println!(
                "\
┏━━━┓ ┏━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┓ ┏━━━┯━━━┓
┃ {} ┃ ┃ {} │ Move number {i:3} ┃ ┃ {} │ ▶ ┃
┗━━━┛ ┗━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┛ ┗━━━┷━━━┛",
                match turn {
                    Faction::Black => "◯",
                    Faction::White => "⬤",
                }
                .bold(),
                format!("{:?} to move", turn).dim(),
                if i == 0 {
                    format!("{}", "◀".dim())
                } else {
                    "◀".to_owned()
                }
            );
        } else {
            print!("{}", self.board);

            println!(
                "\
┏━━━┓ ┏━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┓ ┏━━━┯━━━┓
┃ {} ┃ ┃ {:?} to move │ Move number {:3} ┃ ┃ {} │ {} ┃
┗━━━┛ ┗━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┛ ┗━━━┷━━━┛",
                match self.turn {
                    Faction::Black => "◯",
                    Faction::White => "⬤",
                }
                .bold(),
                self.turn,
                self.history.len(),
                if self.history.is_empty() {
                    format!("{}", "◀".dim())
                } else {
                    "◀".to_owned()
                },
                "▶".dim(),
            );
        }
    }

    fn handle_mouse_input(
        &mut self,
        column: u16,
        row: u16,
    ) -> Result<(), &'static str> {
        if self.looking_back_at.is_some() {
            return Ok(());
        }

        let Some(coord) = screen_coord_to_game_coord([row, column]) else {
            return Ok(());
        };

        if self.board.get_2d(coord).and_then(|x| x.try_into().ok())
            == Some(self.turn)
        {
            self.selected = Some(coord);

            self.legal_moves = self.board.moves_from(coord);

            execute!(self.out, cursor::MoveTo(0, 0)).unwrap();
            println!("{}", HighlightedBoardState(self.board, self.legal_moves));
        } else if self.legal_moves[to_linind(coord).unwrap()] {
            self.history.push(self.board);

            let won = self.board.do_move(self.selected.unwrap(), coord);

            self.selected = None;
            self.legal_moves = BitArray::new();

            self.turn = self.turn.other_faction();

            self.render();

            if won {
                execute!(
                    self.out,
                    cursor::MoveUp(3),
                    terminal::Clear(terminal::ClearType::FromCursorDown)
                )
                .unwrap();

                let winning_faction = self.turn.other_faction();

                println!(
                    "\
┏━━━┓ ┏━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━━━━━━━━━━━━━┓
┃ {} ┃ ┃ {:?} wins! ┃ ┃ Press any key to quit ┃
┗━━━┛ ┗━━━━━━━━━━━━━┛ ┗━━━━━━━━━━━━━━━━━━━━━━━┛",
                    match winning_faction {
                        Faction::Black => "◯",
                        Faction::White => "⬤",
                    }
                    .bold(),
                    winning_faction,
                );

                while let Ok(x) = event::read() {
                    if let Event::Key(_) = x {
                        break;
                    }
                }

                return Err("break");
            }

            println!();
        }

        Ok(())
    }

    pub fn run(&mut self) {
        execute!(
            self.out,
            EnableMouseCapture,
            EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();

        self.render();

        while let Ok(x) = event::read() {
            match x {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                })
                | Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 1..=3,
                    row: 1,
                    modifiers: KeyModifiers::NONE,
                }) => break,

                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: _,
                })
                | Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 43..=45,
                    row: 27,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if let Some(i) = self.looking_back_at.as_mut() {
                        if *i > 0 {
                            *i -= 1;
                        }
                    } else if !self.history.is_empty() {
                        self.looking_back_at = Some(self.history.len() - 1);
                    }

                    self.render();
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: _,
                })
                | Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 47..=49,
                    row: 27,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if let Some(i) = self.looking_back_at.as_mut() {
                        if *i < self.history.len() - 1 {
                            *i += 1;
                        } else {
                            self.looking_back_at = None;
                        }
                    }

                    self.render();
                }

                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }) => match self.handle_mouse_input(column, row) {
                    Ok(()) => {}
                    Err("break") => break,
                    Err(x) => panic!("{x}"),
                },

                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Right),
                    column: _,
                    row: _,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if self.selected.is_some() {
                        self.selected = None;

                        self.legal_moves = BitArray::new();

                        self.render();
                    }
                }

                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: _,
                }) if ('0'..='3').contains(&c) => {
                    self.render();
                    execute!(
                        self.out,
                        terminal::Clear(terminal::ClearType::FromCursorDown)
                    )
                    .unwrap();
                    println!("Computing best move:");
                    let t = Instant::now();

                    let best_move = self
                        .board
                        .best_move(self.turn, (c as u8 - b'0') as u32);

                    let t = t.elapsed();

                    println!(
                        "Best move: {} -> {}, score: {}, Took: {t:.2?}",
                        to_readable_coord(best_move.0[0]),
                        to_readable_coord(best_move.0[1]),
                        best_move.1,
                    );
                }

                _ => {}
            }
        }

        execute!(
            self.out,
            DisableMouseCapture,
            cursor::Show,
            LeaveAlternateScreen
        )
        .unwrap();
    }
}
