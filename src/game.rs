use std::io::stdout;

use bitarray::BitArray;
use crossterm::{
    cursor,
    event::{
        self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
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
    board: BoardState,
    selected: Option<[u16; 2]>,
    legal_moves: BitArray<{ board::M }>,
    turn: Faction,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: BoardState::standard_setup(),
            selected: None,
            legal_moves: BitArray::new(),
            turn: Faction::Black,
        }
    }

    pub fn run(&mut self) {
        let mut out = stdout();

        execute!(
            out,
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

                    if self.board.get_2d(coord).and_then(|x| x.try_into().ok())
                        == Some(self.turn)
                    {
                        self.selected = Some(coord);

                        self.legal_moves = self.board.moves_from(coord);

                        execute!(out, cursor::MoveTo(0, 0)).unwrap();
                        println!(
                            "{}",
                            HighlightedBoardState(self.board, self.legal_moves)
                        );
                    } else if self.legal_moves[to_linind(coord).unwrap()] {
                        let won =
                            self.board.do_move(self.selected.unwrap(), coord);

                        self.turn = self.turn.other_faction();

                        execute!(out, cursor::MoveTo(0, 0)).unwrap();
                        println!("{}{:?} to move", self.board, self.turn);

                        if won {
                            println!("{:?} wins!", self.turn.other_faction());

                            println!("Press any key to quit");

                            while let Ok(x) = event::read() {
                                if let Event::Key(_) = x {
                                    break;
                                }
                            }

                            break;
                        }
                    }
                }
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Right),
                    column: _,
                    row: _,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if self.selected.is_some() {
                        self.selected = None;

                        self.legal_moves = BitArray::new();

                        execute!(out, cursor::MoveTo(0, 0)).unwrap();
                        println!("{}", self.board);
                    }
                }
                _ => {}
            }
        }

        execute!(out, LeaveAlternateScreen).unwrap();
    }
}
