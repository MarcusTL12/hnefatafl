use std::{env::args, time::Instant};

use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const BASE_MASK: u128 = 0x00004008010020040080100200400801;

fn make_moves_mask(i: u8) -> u128 {
    BASE_MASK & !(1 << (11 * i))
}

fn make_obstructor(mut mask: u128, n: u16) -> u128 {
    let mut m = 0;

    let mut i = 0;
    let mut j = 0;

    while mask != 0 {
        let tz = mask.trailing_zeros();
        mask >>= tz + 1;
        j += tz;

        if (n & (1 << i)) != 0 {
            m |= 1 << j;
        }

        i += 1;
        j += 1;
    }

    m
}

fn make_actual_moves(i: u8, n: u16) -> u128 {
    let mask = make_moves_mask(i);
    let obstructor = make_obstructor(mask, n);

    let mut m = 0;

    for k in (i + 1)..11 {
        let bit = 1 << (11 * k);

        if obstructor & bit != 0 {
            break;
        }

        m |= bit;
    }

    for k in (0..i).rev() {
        let bit = 1 << (11 * k);

        if obstructor & bit != 0 {
            break;
        }

        m |= bit;
    }

    m
}

fn make_all_obstructors(i: u8) -> [u128; 1024] {
    let mut obstructors = [0; 1024];

    let m = make_moves_mask(i);

    for (n, o) in obstructors.iter_mut().enumerate() {
        *o = make_obstructor(m, n as u16);
    }

    obstructors
}

fn make_all_moves(i: u8) -> [u128; 1024] {
    let mut moves = [0; 1024];

    for (n, m) in moves.iter_mut().enumerate() {
        *m = make_actual_moves(i, n as u16);
    }

    moves
}

fn obstruction_to_ind(obstruction: u128, multiplier: u128) -> u64 {
    ((obstruction * multiplier) >> 64) as u64
}

fn test_magic_number<const BITMASK: usize, const NCOMBS: usize>(
    obstructors: &[u128; 1024],
    moves: &[u128; 1024],
    multiplier: u128,
) -> Option<u32> {
    let unshifted_inds = obstructors.map(|o| obstruction_to_ind(o, multiplier));

    'outer: for s in 0..64 {
        let mut lookup = [!0; NCOMBS];

        for (&m, ui) in moves.iter().zip(unshifted_inds) {
            let i = (ui >> s) as usize & BITMASK;

            if lookup[i] == !0 {
                lookup[i] = m;
            } else if lookup[i] != m {
                continue 'outer;
            }
        }

        return Some(s);
    }

    None
}

fn main() {
    let mut args = args();
    args.next();

    let command = args.next().unwrap();

    match command.as_str() {
        "check-11" => {
            let i = args.next().unwrap().parse().unwrap();
            let m = u128::from_str_radix(&args.next().unwrap(), 16).unwrap();

            let obstructors = make_all_obstructors(i);
            let moves = make_all_moves(i);

            if let Some(s) = test_magic_number::<0b111_1111_1111, 2048>(
                &obstructors,
                &moves,
                m,
            ) {
                println!("Number is magic with s = {s}");
            } else {
                println!("Number is not magic");
            }
        }
        "search" => {
            let bits: u8 = args.next().unwrap().parse().unwrap();
            let i = args.next().unwrap().parse().unwrap();
            let n: usize = args.next().unwrap().parse().unwrap();

            let obstructors = make_all_obstructors(i);
            let moves = make_all_moves(i);

            let f = match bits {
                10 => test_magic_number::<0b11_1111_1111, 1024>,
                11 => test_magic_number::<0b111_1111_1111, 2048>,
                12 => test_magic_number::<0b1111_1111_1111, 4096>,
                _ => unimplemented!(),
            };

            let t = Instant::now();

            let ans = (0..n).into_par_iter().find_map_any(|k| {
                let m = random();

                f(&obstructors, &moves, m).map(|s| (m, s, k))
            });

            let t = t.elapsed();

            if let Some((m, s, k)) = ans {
                println!(
                    "Found {bits} bit magic number for index {i} after {k} trials:
    m = 0x{m:x}, s = {s}"
                );
            } else {
                println!("Did not find working magic number!");
            }

            println!("Time elapsed: {t:.2?}");
        }
        _ => println!("Unimplemented command!"),
    }
}
