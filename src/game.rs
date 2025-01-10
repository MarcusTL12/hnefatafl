use std::io::{Stdout, stdout};

use bitarray::BitArray;
use crossterm::{
    cursor,
    event::{
        self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::board::{
    self, BoardState, Faction, HighlightedBoardState, to_linind,
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
}

impl GameState {
    pub fn new() -> Self {
        Self {
            out: stdout(),
            board: BoardState::standard_setup(),
            selected: None,
            legal_moves: BitArray::new(),
            turn: Faction::Black,
        }
    }

    fn handle_mouse_input(
        &mut self,
        column: u16,
        row: u16,
    ) -> Result<(), &'static str> {
        let Some(coord) = screen_coord_to_game_coord([row, column]) else {
            return Err("continue");
        };

        if self.board.get_2d(coord).and_then(|x| x.try_into().ok())
            == Some(self.turn)
        {
            self.selected = Some(coord);

            self.legal_moves = self.board.moves_from(coord);

            execute!(self.out, cursor::MoveTo(0, 0)).unwrap();
            println!("{}", HighlightedBoardState(self.board, self.legal_moves));
        } else if self.legal_moves[to_linind(coord).unwrap()] {
            let won = self.board.do_move(self.selected.unwrap(), coord);

            self.turn = self.turn.other_faction();

            execute!(self.out, cursor::MoveTo(0, 0)).unwrap();
            print!("{}", self.board);

            if won {
                execute!(
                    self.out,
                    terminal::Clear(terminal::ClearType::CurrentLine)
                )
                .unwrap();
                println!("{:?} wins!", self.turn.other_faction());

                println!("Press any key to quit");

                while let Ok(x) = event::read() {
                    if let Event::Key(_) = x {
                        break;
                    }
                }

                return Err("break");
            } else {
                print!("{:?} to move", self.turn);
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
            cursor::MoveTo(0, 0)
        )
        .unwrap();

        println!("{}{:?} to move", self.board, self.turn);

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
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }) => match self.handle_mouse_input(column, row) {
                    Err("continue") => continue,
                    Err("break") => break,
                    Ok(()) => {}
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

                        execute!(self.out, cursor::MoveTo(0, 0)).unwrap();
                        println!("{}", self.board);
                    }
                }
                _ => {}
            }
        }

        execute!(self.out, LeaveAlternateScreen).unwrap();
    }
}
