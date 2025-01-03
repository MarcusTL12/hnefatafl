# Hnefatafl
Hnefatafl testing

## Notes

### State size and storage format:

Board size 11x11 = 121 squares
1 bit board fits in u128

4 square states:
    Empty
    White
    Black
    King

Fits in 2 bits per square:
Board state fits in 2 u128 or 4 u64

### Number of total game states:

Number of black pieces: 6 * 4 = 24
Number of white pieces: 12
King: 1

States of full number of pieces:
(121 1) * (120 12) * (108 24) = 821774405941582612288082094367000888473000
= 8.21e41
(unreasonable ammount to store)
