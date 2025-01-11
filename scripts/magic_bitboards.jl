
if !@isdefined tower
    global tower = falses(11, 11)

    tower[1, 1] = true
    tower[end, 1] = true
    tower[1, end] = true
    tower[end, end] = true
    tower[6, 6] = true

    global tower_bits::UInt128 = reinterpret(UInt128, tower.chunks)[1]
end

@inline function to2d(i, j)
    i + j * 11
end

function make_possible_moves_mask(i, j)
    moves = UInt128(0)

    for k in 0:10
        moves |= UInt128(1) << to2d(k, j)
        moves |= UInt128(1) << to2d(i, k)
    end

    moves &= ~(UInt128(1) << to2d(i, j))

    moves & ~tower_bits
end

function bitmap_to_u128(m)
    reinterpret(UInt128, m.chunks)[1]
end

function map_num_to_mask(mask, n)
    m = UInt128(0)

    i = 0
    j = 0

    while mask != 0
        tz = trailing_zeros(mask)
        mask >>= tz + 1
        j += tz

        if (n & (1 << i)) != 0
            m |= UInt128(1) << j
        end

        i += 1
        j += 1
    end

    m
end

function make_actual_moves(i, j, n)
    mask = make_possible_moves_mask(i, j)

    obstructions = map_num_to_mask(mask, n)

    @show obstructions

    m = UInt128(0)

    for k in i+1:10
        bit = UInt128(1) << to2d(k, j)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    for k in i-1:-1:0
        bit = UInt128(1) << to2d(k, j)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    for k in j+1:10
        bit = UInt128(1) << to2d(i, k)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    for k in j-1:-1:0
        bit = UInt128(1) << to2d(i, k)

        if (obstructions & bit) != 0
            break
        end

        m |= bit
    end

    m & ~tower_bits
end
