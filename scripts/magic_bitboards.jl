using Printf

const tower_bits::UInt128 = 0x01004000000000001000000000000401

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

    moves
end

function make_vertical_moves_mask(i)
    moves = UInt128(0)

    for k in 0:10
        moves |= UInt128(1) << (k * 11)
    end

    moves &= ~(UInt128(1) << (i * 11))

    moves
end

function make_horizontal_moves_mask(j)
    moves = UInt128(0)

    for k in 0:10
        moves |= UInt128(1) << k
    end

    moves &= ~(UInt128(1) << j)

    moves
end

function make_up_moves_mask(i)
    moves = UInt128(0)

    for k in 0:i-1
        moves |= UInt128(1) << (k * 11)
    end

    moves
end

function make_down_moves_mask(i)
    moves = UInt128(0)

    for k in i+1:10
        moves |= UInt128(1) << (k * 11)
    end

    moves
end

function bitmap_to_u128(m)
    reinterpret(UInt128, m.chunks)[1]
end

function make_obstructor(mask, n)
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

    obstructions = make_obstructor(mask, n)

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

    m
end

function make_actual_vertical_moves(i, n)
    mask = make_vertical_moves_mask(i)

    obstructions = make_obstructor(mask, n)

    m = UInt128(0)

    for k in i+1:10
        bit = UInt128(1) << (k * 11)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    for k in i-1:-1:0
        bit = UInt128(1) << (k * 11)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    m
end

function make_actual_horizontal_moves(j, n)
    mask = make_horizontal_moves_mask(j)

    obstructions = make_obstructor(mask, n)

    m = UInt128(0)

    for k in j+1:10
        bit = UInt128(1) << k

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    for k in j-1:-1:0
        bit = UInt128(1) << k

        if (obstructions & bit) != 0
            break
        end

        m |= bit
    end

    m
end

function make_actual_up_moves(i, n)
    mask = make_vertical_moves_mask(i)

    obstructions = make_obstructor(mask, n)

    m = UInt128(0)

    for k in i-1:-1:0
        bit = UInt128(1) << (k * 11)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    m
end

function make_actual_down_moves(i, n)
    mask = make_vertical_moves_mask(i)

    obstructions = make_obstructor(mask, n)

    m = UInt128(0)

    for k in i+1:10
        bit = UInt128(1) << (k * 11)

        if obstructions & bit != 0
            break
        end

        m |= bit
    end

    m
end

function obstruction_to_ind(obstruction, multiplier)
    unsafe_trunc(UInt64, (obstruction * multiplier) >> 64)
end

function look_for_collisions(lookup, moves, unshifted_inds, bitmask)
    best_cap = typemax(UInt64)
    best_s = 65
    best_n = 0

    for s in 0:64
        empty!(lookup)

        working = true

        for (m, ui) in zip(moves, unshifted_inds)
            i = (ui >> s) & bitmask

            if !haskey(lookup, i)
                lookup[i] = m
            elseif lookup[i] != m
                working = false
                break
            end
        end

        if working
            cap = maximum(keys(lookup))
            if cap < best_cap
                best_cap = cap
                best_s = s
                best_n = length(lookup)
            end
        end
    end

    (best_s, best_cap, best_n)
end

function test_magic_number(lookup, ind_buf, obstructors, moves, multiplier, bitmask)
    empty!(ind_buf)
    for o in obstructors
        push!(ind_buf, obstruction_to_ind(o, multiplier))
    end

    look_for_collisions(lookup, moves, ind_buf, bitmask)
end

function look_for_magic_number(i, j, n_trials, best_cap=typemax(UInt64))
    mask = make_possible_moves_mask(i, j)

    cap = count_ones(mask)

    obstructors = [make_obstructor(mask, n) for n in 0:2^cap-1]
    moves = [make_actual_moves(i, j, n) for n in 0:2^cap-1]

    best_c = Threads.Atomic{UInt64}(best_cap)

    nth = Threads.nthreads()

    history = [Tuple{UInt128,Int,UInt64,Int}[] for _ in 1:nth]

    Threads.@threads for id in 1:nth
        lookup = Dict{UInt64,UInt128}()
        ind_buf = UInt64[]

        for _ in id:nth:n_trials
            m = rand(UInt128)

            s, c, n = test_magic_number(lookup, ind_buf, obstructors, moves, m, ~UInt64(0))

            if c < best_c[]
                size_mb = 16 * c / 1024^2
                efficiency = (n / c) * 100

                println("Found new m = $m, s = $s")
                println("    size = $size_mb MiB")
                @printf "    efficiency = %.2f %c\n\n" efficiency '%'

                Threads.atomic_min!(best_c, c)

                push!(history[id], (m, s, c, n))
            end
        end
    end

    for h in history
        for (m, s, c, n) in h
            if c == best_c[]
                return (m, s, c, n)
            end
        end
    end
end

function look_for_vertical_magic_number(i, bitmask, n_trials, best_cap=typemax(UInt64))
    mask = make_vertical_moves_mask(i)

    cap = count_ones(mask)

    obstructors = [make_obstructor(mask, n) for n in 0:2^cap-1]
    moves = [make_actual_vertical_moves(i, n) for n in 0:2^cap-1]

    best_c = Threads.Atomic{UInt64}(best_cap)

    nth = Threads.nthreads()

    history = [Tuple{UInt128,Int,UInt64,Int}[] for _ in 1:nth]

    Threads.@threads for id in 1:nth
        lookup = Dict{UInt64,UInt128}()
        ind_buf = UInt64[]

        for _ in id:nth:n_trials
            m = rand(UInt128)

            s, c, n = test_magic_number(lookup, ind_buf, obstructors, moves, m, bitmask)

            if c < best_c[]
                size_mb = 16 * (c + 1) / 1024
                efficiency = (n / (c + 1)) * 100

                println("Found new m = $m, s = $s")
                println("    size = $size_mb kiB")
                @printf "    efficiency = %.2f %c\n\n" efficiency '%'

                Threads.atomic_min!(best_c, c)

                push!(history[id], (m, s, c, n))
            end
        end
    end

    for h in history
        for (m, s, c, n) in h
            if c == best_c[]
                return (m, s, Int(c + 1) * 16 / 1024, n)
            end
        end
    end
end

# Scoreboard vertical:
#  0 => 0x501e1970a031effc2a07048e4fdb9fff, 37, 10 bit
#  1 => 0x751bfde58826f7fdffaffbbfdff77fff, 37, 10 bit
#  2 => 0xa04c5c46b05e0d6b298eeb8e10c1effe, 38, 10 bit
#  3 => 0x2ae249b8c9983da0cdffe930c6c00b32, 38, 11 bit
#  4 => 0xf9e9accbfff623354b06ca7bfff9542f, 37, 11 bit
#  5 => 0xc068f9868341f917eda8040f97dac16c, 38, 11 bit
#  6 => 0xa634d2305654600820312cc6083ea3c1, 36, 11 bit
#  7 => 0x3d4f0e3637a1ab59743b9d175dd17146, 37, 11 bit
#  8 => 0x9eafffde40e34b972146f8fffa64e154, 39, 10 bit
#  9 => 0xa2227ffd6fb1f5d0112bada224aad389, 37, 10 bit
# 10 => 0x76d837fff936da3d90a3a7af54efb580, 33, 10 bit

magic_numbers = [
    0x501e1970a031effc2a07048e4fdb9fff
    0x751bfde58826f7fdffaffbbfdff77fff
    0xa04c5c46b05e0d6b298eeb8e10c1effe
    0x2ae249b8c9983da0cdffe930c6c00b32
    0xf9e9accbfff623354b06ca7bfff9542f
    0xc068f9868341f917eda8040f97dac16c
    0xa634d2305654600820312cc6083ea3c1
    0x3d4f0e3637a1ab59743b9d175dd17146
    0x9eafffde40e34b972146f8fffa64e154
    0xa2227ffd6fb1f5d0112bada224aad389
    0x76d837fff936da3d90a3a7af54efb580
]

magic_shifts = [37, 37, 38, 38, 37, 38, 36, 37, 39, 37, 33]

function populate_magic_lookup(i, bitmask, multiplier, s)
    mask = make_vertical_moves_mask(i)

    cap = count_ones(mask)

    obstructors = [make_obstructor(mask, n) for n in 0:2^cap-1]
    moves = [make_actual_vertical_moves(i, n) for n in 0:2^cap-1]

    n_lookup = bitmask + 1

    lookup = zeros(UInt128, n_lookup)

    for (m, o) in zip(moves, obstructors)
        i = (obstruction_to_ind(o, multiplier) >> s) & bitmask

        lookup[begin+i] = m
    end

    lookup
end

function make_magic_lookup_file()
    filename = "res/11bit_magic_lookup.dat"
    bitmask = 0b111_1111_1111

    lookup = UInt128[]

    for i in 0:10
        append!(lookup, populate_magic_lookup(i, bitmask,
            magic_numbers[begin+1], magic_shifts[begin+i]))
    end

    open(filename, "w") do io
        write(io, lookup)
    end
end

function populate_horizontal_lookup(j)
    mask = make_horizontal_moves_mask(j)

    cap = count_ones(mask)

    obstructors = [make_obstructor(mask, n) for n in 0:2^cap-1]
    moves = [make_actual_horizontal_moves(j, n) for n in 0:2^cap-1]

    n_lookup = 2^11

    lookup = zeros(UInt16, n_lookup)

    for (m, o) in zip(moves, obstructors)
        i = Int(o)
        lookup[begin+i] = UInt16(m)
    end

    lookup
end

function make_horizontal_lookup_file()
    filename = "res/horizontal_lookup.dat"

    lookup = UInt16[]

    for j in 0:10
        append!(lookup, populate_horizontal_lookup(j))
    end

    open(filename, "w") do io
        write(io, lookup)
    end
end

function look_for_up_magic_number(i, bitmask, n_trials, best_cap=typemax(UInt64))
    mask = make_up_moves_mask(i)

    cap = count_ones(mask)

    obstructors = [make_obstructor(mask, n) for n in 0:2^cap-1]
    moves = [make_actual_up_moves(i, n) for n in 0:2^cap-1]

    best_c = Threads.Atomic{UInt64}(best_cap)

    nth = Threads.nthreads()

    history = [Tuple{UInt128,Int,UInt64,Int}[] for _ in 1:nth]

    Threads.@threads for id in 1:nth
        lookup = Dict{UInt64,UInt128}()
        ind_buf = UInt64[]

        for _ in id:nth:n_trials
            m = rand(UInt128)

            s, c, n = test_magic_number(lookup, ind_buf, obstructors, moves, m, bitmask)

            if c < best_c[]
                size_b = 16 * (c + 1)
                efficiency = (n / (c + 1)) * 100

                println("Found new m = $m, s = $s")
                println("    size = $size_b Bytes")
                @printf "    efficiency = %.2f %c\n\n" efficiency '%'

                Threads.atomic_min!(best_c, c)

                push!(history[id], (m, s, c, n))
            end
        end
    end

    for h in history
        for (m, s, c, n) in h
            if c == best_c[]
                return (m, s, Int(c + 1) * 16, n)
            end
        end
    end
end

# Scoreboard "up":
# 0 => none
# 1 => obstructor directly
# 2 => 0xbd1bc995c9666c4a629358503bb6f0b5, s = 1,  bits = 2,   64 bytes
# 3 => 0xe382a155ebb143971528d4fb931aa092, s = 5,  bits = 3,  112 bytes
# 4 => 0x16ff9c973fccd1ab255dfa8dba7fad61, s = 27, bits = 3,  128 bytes
# 5 => 0x294907387ff05d85f8d50d3cbf0dd731, s = 23, bits = 4,  256 bytes
# 6 => 0x4314b658f05851ff9fff3fbf3554309f, s = 4,  bits = 5,  512 bytes
# 7 => 0x168c5f7ff1101f3694f5d699dadaae23, s = 33, bits = 6, 1024 bytes
# 8 =>
# 9 =>
# 10 =>
