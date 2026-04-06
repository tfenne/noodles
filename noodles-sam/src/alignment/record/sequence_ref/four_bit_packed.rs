mod iter;

use self::iter::Iter;

pub(super) struct FourBitPacked<'a> {
    src: &'a [u8],
    base_count: usize,
}

impl<'a> FourBitPacked<'a> {
    pub(super) fn is_empty(&self) -> bool {
        self.base_count == 0
    }

    pub(super) fn len(&self) -> usize {
        self.base_count
    }

    pub(super) fn get(&self, i: usize) -> Option<u8> {
        if i < self.base_count
            && let Some(&n) = self.src.get(i / 2)
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

    pub(super) fn iter(&self) -> Iter<'a> {
        Iter::new(self.src, self.base_count)
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
