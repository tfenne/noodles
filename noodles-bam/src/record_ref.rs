#![allow(dead_code)]

use std::{io, mem};

use bstr::{BStr, ByteSlice};
use noodles_core::Position;
use noodles_sam::alignment::record::Flags;

use super::record::{
    fields::{bounds, get_position, get_reference_sequence_id},
    try_to_position, try_to_reference_sequence_id,
};

struct RecordRef<'a>(&'a [u8]);

impl RecordRef<'_> {
    fn reference_sequence_id(&self) -> Option<io::Result<usize>> {
        // SAFETY: `self.head.len() >= mem::size_of::<i32>()`.
        let src = self.0.first_chunk().unwrap();
        get_reference_sequence_id(*src).map(try_to_reference_sequence_id)
    }

    fn alignment_start(&self) -> Option<io::Result<Position>> {
        let src = &self.0[bounds::ALIGNMENT_START_RANGE];
        // SAFETY: `src.len() == mem::size_of::<i32>()`.
        get_position(src.try_into().unwrap()).map(try_to_position)
    }

    fn name_length(&self) -> usize {
        let n = &self.0[bounds::NAME_LENGTH_INDEX];
        usize::from(*n)
    }

    fn mapping_quality(&self) -> Option<u8> {
        const MISSING: u8 = 255;

        match self.0[bounds::MAPPING_QUALITY_INDEX] {
            MISSING => None,
            n => Some(n),
        }
    }

    fn cigar_op_count(&self) -> usize {
        let src = &self.0[bounds::CIGAR_OP_COUNT_RANGE];
        // SAFETY: `src.len() == mem::size_of::<u16>()`.
        usize::from(u16::from_le_bytes(src.try_into().unwrap()))
    }

    fn flags(&self) -> Flags {
        let src = &self.0[bounds::FLAGS_RANGE];
        // SAFETY: `src.len() == mem::size_of::<u16>()`.
        let n = u16::from_le_bytes(src.try_into().unwrap());
        Flags::from(n)
    }

    fn base_count(&self) -> usize {
        let src = &self.0[bounds::READ_LENGTH_RANGE];
        // SAFETY: `src.len() == mem::size_of::<u32>()`.
        let n = u32::from_le_bytes(src.try_into().unwrap());
        usize::try_from(n).unwrap()
    }

    fn mate_reference_sequence_id(&self) -> Option<io::Result<usize>> {
        let src = &self.0[bounds::MATE_REFERENCE_SEQUENCE_ID_RANGE];
        // SAFETY: `src.len() == mem::size_of::<i32>()`.
        get_reference_sequence_id(src.try_into().unwrap()).map(try_to_reference_sequence_id)
    }

    fn mate_alignment_start(&self) -> Option<io::Result<Position>> {
        let src = &self.0[bounds::MATE_ALIGNMENT_START_RANGE];
        // SAFETY: `src.len() == mem::size_of::<i32>()`.
        get_position(src.try_into().unwrap()).map(try_to_position)
    }

    fn template_length(&self) -> i32 {
        let src = &self.0[bounds::TEMPLATE_LENGTH_RANGE];
        // SAFETY: `src.len() == mem::size_of::<i32>()`.
        i32::from_le_bytes(src.try_into().unwrap())
    }

    fn name(&self) -> Option<&BStr> {
        const NUL: u8 = 0x00;
        const MISSING: &[u8] = &[b'*', NUL];

        let read_name_len = self.name_length();
        let start = bounds::TEMPLATE_LENGTH_RANGE.end;
        let end = start + read_name_len;

        match &self.0[start..end] {
            MISSING => None,
            buf => Some(buf.strip_suffix(&[NUL]).unwrap_or(buf).as_bstr()),
        }
    }

    fn sequence(&self) -> &[u8] {
        let start = bounds::TEMPLATE_LENGTH_RANGE.end
            + self.name_length()
            + (self.cigar_op_count() * mem::size_of::<u32>());

        let sequence_len = self.base_count().div_ceil(2);
        let end = start + sequence_len;

        &self.0[start..end]
    }

    fn quality_scores(&self) -> &[u8] {
        let base_count = self.base_count();

        let start = bounds::TEMPLATE_LENGTH_RANGE.end
            + self.name_length()
            + (self.cigar_op_count() * mem::size_of::<u32>())
            + base_count.div_ceil(2);

        let end = start + base_count;

        &self.0[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fields() -> io::Result<()> {
        const SRC: &[u8; 44] = &[
            0xff, 0xff, 0xff, 0xff, // ref_id = -1
            0xff, 0xff, 0xff, 0xff, // pos = -1
            0x02, // l_read_name = 2
            0xff, // mapq = 255
            0x48, 0x12, // bin = 4680
            0x01, 0x00, // n_cigar_op = 1
            0x04, 0x00, // flag = 4
            0x04, 0x00, 0x00, 0x00, // l_seq = 0
            0xff, 0xff, 0xff, 0xff, // next_ref_id = -1
            0xff, 0xff, 0xff, 0xff, // next_pos = -1
            0x00, 0x00, 0x00, 0x00, // tlen = 0
            b'*', 0x00, // read_name = "*\x00"
            0x40, 0x00, 0x00, 0x00, // cigar = 4M
            0x12, 0x48, // sequence = ACGT
            b'N', b'D', b'L', b'S', // quality scores
        ];

        let record = RecordRef(SRC);

        assert!(record.reference_sequence_id().transpose()?.is_none());
        assert!(record.alignment_start().transpose()?.is_none());
        assert!(record.mapping_quality().is_none());
        assert_eq!(record.flags(), Flags::UNMAPPED);
        assert!(record.mate_reference_sequence_id().transpose()?.is_none());
        assert!(record.mate_alignment_start().transpose()?.is_none());
        assert_eq!(record.template_length(), 0);
        assert!(record.name().is_none());
        assert_eq!(record.sequence(), &[0x12, 0x48]);
        assert_eq!(record.quality_scores(), b"NDLS");

        Ok(())
    }
}
