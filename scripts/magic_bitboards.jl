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
#  0 => 0x260cd366bff7bfa0001986996c81fff7, 41, 11 bit, 31.98 kiB
#  1 => 0xc286186d1f485cfcefa06c16766e28e4, 37, 12 bit, 63.82 kiB
#  2 => 0x034e85551b77bd36e10e3fc06451ccb5, 39, 12 bit, 63.84 kiB
#  3 => 0xfd216858c8fdbe5c123f6178eb812f90, 39, 12 bit, 63.78 kiB
#  4 => 0x6713254cf439424ab8f812b4d59a009e, 36, 12 bit, 63.88 kiB
#  5 => 0x7e79d26f61b4fa51e6dd119fe5162958, 38, 12 bit, 63.91 kiB
#  6 => 0xb7c14fde474c01ce4792f8f4141094fc, 37, 12 bit, 63.86 kiB
#  7 => 0xdef3e227db21f66a367630ff83b33ec5, 40, 12 bit, 63.83 kiB
#  8 => 0xa580f97b505d5753421b5e4a403c29f8, 38, 12 bit, 63.94 kiB
#  9 => 0xddcd03fb2c8be68eaf1e30257b9db83f, 35, 12 bit, 63.92 kiB
# 10 => 0xbbcc2dd321f5c278990ec2886b804766, 25, 12 bit, 63.86 kiB

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
