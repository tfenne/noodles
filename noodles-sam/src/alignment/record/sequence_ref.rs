#![expect(dead_code)]

mod four_bit_packed;

enum SequenceRef<'a> {
    FourBitPacked(&'a [u8], usize),
    Raw(&'a [u8]),
}

impl SequenceRef<'_> {
    fn is_empty(&self) -> bool {
        match self {
            Self::FourBitPacked(_, base_count) => *base_count == 0,
            Self::Raw(src) => src.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::FourBitPacked(_, base_count) => *base_count,
            Self::Raw(src) => src.len(),
        }
    }

    fn get(&self, i: usize) -> Option<u8> {
        match self {
            Self::FourBitPacked(src, base_count) => {
                if i < *base_count
                    && let Some(&n) = src.get(i / 2)
                {
                    if i.is_multiple_of(2) {
                        Some(decode_base(n >> 4))
                    } else {
                        Some(decode_base(n))
                    }
                } else {
                    None
                }
            }
            Self::Raw(src) => src.get(i).copied(),
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        match self {
            Self::FourBitPacked(src, base_count) => {
                Box::new(four_bit_packed::Iter::new(src, *base_count))
            }
            Self::Raw(src) => Box::new(src.iter().copied()),
        }
    }
}

fn decode_base(n: u8) -> u8 {
    const BASE_LUT: &[u8; 16] = b"=ACMGRSVTWYHKDBN";
    BASE_LUT[usize::from(n & 0x0f)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base() {
        const EXPECTED_BASES: &[u8; 16] = b"=ACMGRSVTWYHKDBN";

        for (n, &expected) in (0..15).zip(EXPECTED_BASES) {
            assert_eq!(decode_base(n), expected);
        }

        assert_eq!(decode_base(16), b'=');
        assert_eq!(decode_base(255), b'N');
    }
}
