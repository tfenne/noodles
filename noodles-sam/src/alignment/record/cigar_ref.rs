#![expect(dead_code)]

use std::mem;

use super::Cigar;

const FOUR_BYTE_PACKED_CHUNK_SIZE: usize = mem::size_of::<u32>();

enum CigarRef<'a> {
    FourBytePacked(&'a [u8]),
    Cigar(Box<dyn Cigar + 'a>),
}

impl CigarRef<'_> {
    fn is_empty(&self) -> bool {
        match self {
            Self::FourBytePacked(src) => src.is_empty(),
            Self::Cigar(cigar) => cigar.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::FourBytePacked(src) => src.len() / FOUR_BYTE_PACKED_CHUNK_SIZE,
            Self::Cigar(cigar) => cigar.len(),
        }
    }
}
