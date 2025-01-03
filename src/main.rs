#![feature(iter_array_chunks)]

use std::fmt::Display;

use bitarray::BitArray;

const N: usize = 256usize.div_ceil(usize::BITS as usize);

struct BoardState(pub BitArray<N>);

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first_row = true;

        for c in self
            .0
            .into_iter()
            .array_chunks::<2>()
            .take(121)
            .array_chunks::<11>()
        {
            let mut first_col = true;

            if first_row {
                writeln!(f, "╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗")?;
            } else {
                writeln!(f, "╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───╢")?;
            }

            for x in c {
                if first_col {
                    write!(f, "║")?;
                } else {
                    write!(f, "│")?;
                }

                write!(f, "{}", match x {
                    [false, false] => "   ",
                    [true, false] => " ♛ ",
                    [false, true] => " ◯ ",
                    [true, true] => " ⬤ ",
                })?;

                first_col = false;
            }

            writeln!(f, "║")?;

            first_row = false;
        }

        writeln!(f, "╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝")
    }
}

fn main() {
    let mut state = BoardState(BitArray::new());

    state.0.set(6, true);
    state.0.set(7, false);

    state.0.set(14, false);
    state.0.set(15, true);

    state.0.set(50, true);
    state.0.set(51, true);

    println!("{state}");
}
