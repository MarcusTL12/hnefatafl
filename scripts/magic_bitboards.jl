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

    moves & ~tower_bits
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

    m & ~tower_bits
end

function obstruction_to_ind(obstruction, multiplier)
    unsafe_trunc(UInt64, (obstruction * multiplier) >> 64)
end

function look_for_collisions(lookup, moves, unshifted_inds)
    best_cap = typemax(UInt64)
    best_s = 65
    best_n = 0

    for s in 39:64
        empty!(lookup)

        working = true

        for (m, ui) in zip(moves, unshifted_inds)
            i = ui >> s

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
        else
            break
        end
    end

    (best_s, best_cap, best_n)
end

function test_magic_number(lookup, ind_buf, obstructors, moves, multiplier)
    empty!(ind_buf)
    for o in obstructors
        push!(ind_buf, obstruction_to_ind(o, multiplier))
    end

    look_for_collisions(lookup, moves, ind_buf)
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

            s, c, n = test_magic_number(lookup, ind_buf, obstructors, moves, m)

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
