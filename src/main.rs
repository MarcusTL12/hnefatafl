#![feature(iter_array_chunks)]

use std::{io::stdout, thread::sleep, time::Duration};

use bitarray::BitArray;
use board::{to_linind, BoardState, Faction, HighlightedBoardState};
use crossterm::{
    ExecutableCommand, cursor,
    event::{
        self, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

mod board;

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

fn main() {
    let mut board = BoardState::standard_setup();

    let mut selected = None;
    let mut legal_moves = BitArray::new();

    let mut turn = Faction::Black;

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

                if board.get_2d(coord).and_then(|x| x.try_into().ok())
                    == Some(turn)
                {
                    selected = Some(coord);

                    legal_moves = board.moves_from(coord);

                    execute!(out, cursor::MoveTo(0, 0)).unwrap();
                    println!("{}", HighlightedBoardState(board, legal_moves));
                } else if legal_moves[to_linind(coord).unwrap()] {
                    let won = board.do_move(selected.unwrap(), coord);

                    turn = turn.other_faction();

                    execute!(out, cursor::MoveTo(0, 0)).unwrap();
                    println!("{board}");

                    if won {
                        println!("{:?} wins!", turn.other_faction());
                        sleep(Duration::from_secs(2));
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
                if selected.is_some() {
                    selected = None;

                    legal_moves = BitArray::new();

                    execute!(out, cursor::MoveTo(0, 0)).unwrap();
                    println!("{board}");
                }
            }
            _ => {}
        }
    }

    stdout().execute(LeaveAlternateScreen).unwrap();
}
