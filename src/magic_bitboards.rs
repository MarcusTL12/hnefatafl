use std::mem;

use bitarray::BitArray;

use crate::board::M;

const BITMASK: usize = 0b111_1111_1111;

const MAGIC_NUMBERS: [u128; 11] = [
    0x501e1970a031effc2a07048e4fdb9fff,
    0x751bfde58826f7fdffaffbbfdff77fff,
    0xa04c5c46b05e0d6b298eeb8e10c1effe,
    0x2ae249b8c9983da0cdffe930c6c00b32,
    0xf9e9accbfff623354b06ca7bfff9542f,
    0xc068f9868341f917eda8040f97dac16c,
    0xa634d2305654600820312cc6083ea3c1,
    0x3d4f0e3637a1ab59743b9d175dd17146,
    0x9eafffde40e34b972146f8fffa64e154,
    0xa2227ffd6fb1f5d0112bada224aad389,
    0x76d837fff936da3d90a3a7af54efb580,
];

const MAGIC_SHIFTS: [u8; 11] = [37, 37, 38, 38, 37, 38, 36, 37, 39, 37, 33];

const VERTICAL_MASK: u128 = 0x00004008010020040080100200400801;

static MAGIC_LOOKUP: [[u128; 2048]; 11] =
    unsafe { mem::transmute(*include_bytes!("../res/11bit_magic_lookup.dat")) };

static HORIZONTAL_LOOKUP: [[u16; 2048]; 11] =
    unsafe { mem::transmute(*include_bytes!("../res/horizontal_lookup.dat")) };

fn get_vertical_moves(obstructors: BitArray<M>, i: u16, j: u16) -> BitArray<M> {
    let obstructors: u128 = unsafe { mem::transmute(obstructors) };

    let o = ((obstructors >> j) & VERTICAL_MASK) & !(1 << (11 * i));

    let ind = ((((o * MAGIC_NUMBERS[i as usize]) >> 64) as u64)
        >> MAGIC_SHIFTS[i as usize]) as usize
        & BITMASK;

    unsafe { mem::transmute(MAGIC_LOOKUP[i as usize][ind] << j) }
}

fn get_horizontal_moves(
    obstructors: BitArray<M>,
    i: u16,
    j: u16,
) -> BitArray<M> {
    let obstructors: u128 = unsafe { mem::transmute(obstructors) };

    let o = ((obstructors >> (11 * i)) as usize) & 0b111_1111_1111 & !(1 << j);

    unsafe {
        mem::transmute((HORIZONTAL_LOOKUP[j as usize][o] as u128) << (11 * i))
    }
}
