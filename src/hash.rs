const CHUNK_SIZE: usize = core::mem::size_of::<u64>() * 4;
const PRIME_1: u64 = 0x9E3779B185EBCA87;
const PRIME_2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME_3: u64 = 0x165667B19E3779F9;
const PRIME_4: u64 = 0x85EBCA77C2B2AE63;
const PRIME_5: u64 = 0x27D4EB2F165667C5;

const fn read_u32(input: &[u8], cursor: usize) -> u32 {
    input[cursor] as u32
        | (input[cursor + 1] as u32) << 8
        | (input[cursor + 2] as u32) << 16
        | (input[cursor + 3] as u32) << 24
}

const fn read_u64(input: &[u8], cursor: usize) -> u64 {
    input[cursor] as u64
        | (input[cursor + 1] as u64) << 8
        | (input[cursor + 2] as u64) << 16
        | (input[cursor + 3] as u64) << 24
        | (input[cursor + 4] as u64) << 32
        | (input[cursor + 5] as u64) << 40
        | (input[cursor + 6] as u64) << 48
        | (input[cursor + 7] as u64) << 56
}

const fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME_2))
        .rotate_left(31)
        .wrapping_mul(PRIME_1)
}

const fn merge_round(mut acc: u64, val: u64) -> u64 {
    acc ^= round(0, val);
    acc.wrapping_mul(PRIME_1).wrapping_add(PRIME_4)
}

const fn avalanche(mut input: u64) -> u64 {
    input ^= input >> 33;
    input = input.wrapping_mul(PRIME_2);
    input ^= input >> 29;
    input = input.wrapping_mul(PRIME_3);
    input ^= input >> 32;
    input
}

const fn finalize(mut input: u64, data: &[u8], mut cursor: usize, end: usize) -> u64 {
    let mut len = end - cursor;

    while len >= 8 {
        input ^= round(0, read_u64(data, cursor));
        cursor += core::mem::size_of::<u64>();
        len -= core::mem::size_of::<u64>();
        input = input
            .rotate_left(27)
            .wrapping_mul(PRIME_1)
            .wrapping_add(PRIME_4);
    }

    if len >= 4 {
        input ^= (read_u32(data, cursor) as u64).wrapping_mul(PRIME_1);
        cursor += core::mem::size_of::<u32>();
        len -= core::mem::size_of::<u32>();
        input = input
            .rotate_left(23)
            .wrapping_mul(PRIME_2)
            .wrapping_add(PRIME_3);
    }

    while len > 0 {
        input ^= (data[cursor] as u64).wrapping_mul(PRIME_5);
        cursor += core::mem::size_of::<u8>();
        len -= core::mem::size_of::<u8>();
        input = input.rotate_left(11).wrapping_mul(PRIME_1);
    }

    avalanche(input)
}

pub(crate) const fn xxh64(input: &[u8], seed: u64) -> u64 {
    xxh64_range(input, 0, input.len(), seed)
}

pub(crate) const fn xxh64_range(input: &[u8], offset: usize, len: usize, seed: u64) -> u64 {
    let input_len = len as u64;
    let mut cursor = offset;
    let end = offset + len;
    let mut result;

    if len >= CHUNK_SIZE {
        let mut v1 = seed.wrapping_add(PRIME_1).wrapping_add(PRIME_2);
        let mut v2 = seed.wrapping_add(PRIME_2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME_1);

        loop {
            v1 = round(v1, read_u64(input, cursor));
            cursor += core::mem::size_of::<u64>();
            v2 = round(v2, read_u64(input, cursor));
            cursor += core::mem::size_of::<u64>();
            v3 = round(v3, read_u64(input, cursor));
            cursor += core::mem::size_of::<u64>();
            v4 = round(v4, read_u64(input, cursor));
            cursor += core::mem::size_of::<u64>();

            if end - cursor < CHUNK_SIZE {
                break;
            }
        }

        result = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));

        result = merge_round(result, v1);
        result = merge_round(result, v2);
        result = merge_round(result, v3);
        result = merge_round(result, v4);
    } else {
        result = seed.wrapping_add(PRIME_5);
    }

    result = result.wrapping_add(input_len);

    finalize(result, input, cursor, end)
}

#[cfg(test)]
mod tests {
    use super::xxh64;

    #[test]
    fn matches_xxh64_empty_vector() {
        assert_eq!(xxh64(b"", 0), 0xef46_db37_51d8_e999);
    }
}
