//! BAM record fields.

pub(crate) mod bounds;

use std::{io, mem};

use self::bounds::Bounds;

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Fields {
    pub(crate) buf: Vec<u8>,
    pub(crate) bounds: Bounds,
}

impl Fields {
    pub(crate) fn index(&mut self) -> io::Result<()> {
        index(&self.buf[..], &mut self.bounds)
    }
}

impl Default for Fields {
    fn default() -> Self {
        let buf = vec![
            0xff, 0xff, 0xff, 0xff, // ref_id = -1
            0xff, 0xff, 0xff, 0xff, // pos = -1
            0x02, // l_read_name = 2
            0xff, // mapq = 255
            0x48, 0x12, // bin = 4680
            0x00, 0x00, // n_cigar_op = 0
            0x04, 0x00, // flag = 4
            0x00, 0x00, 0x00, 0x00, // l_seq = 0
            0xff, 0xff, 0xff, 0xff, // next_ref_id = -1
            0xff, 0xff, 0xff, 0xff, // next_pos = -1
            0x00, 0x00, 0x00, 0x00, // tlen = 0
            b'*', 0x00, // read_name = "*\x00"
        ];

        let bounds = Bounds {
            name_end: buf.len(),
            cigar_end: buf.len(),
            sequence_end: buf.len(),
            quality_scores_end: buf.len(),
        };

        Self { buf, bounds }
    }
}

impl TryFrom<Vec<u8>> for Fields {
    type Error = io::Error;

    fn try_from(buf: Vec<u8>) -> Result<Self, Self::Error> {
        let mut fields = Self {
            buf,
            bounds: Bounds {
                name_end: 0,
                cigar_end: 0,
                sequence_end: 0,
                quality_scores_end: 0,
            },
        };

        fields.index()?;

        Ok(fields)
    }
}

pub(crate) fn get_reference_sequence_id(src: [u8; 4]) -> Option<i32> {
    const UNMAPPED: i32 = -1;

    match i32::from_le_bytes(src) {
        UNMAPPED => None,
        n => Some(n),
    }
}

pub(crate) fn get_position(src: [u8; 4]) -> Option<i32> {
    const MISSING: i32 = -1;

    match i32::from_le_bytes(src) {
        MISSING => None,
        n => Some(n),
    }
}

fn index(buf: &[u8], bounds: &mut Bounds) -> io::Result<()> {
    const MIN_BUF_LENGTH: usize = bounds::TEMPLATE_LENGTH_RANGE.end;

    if buf.len() < MIN_BUF_LENGTH {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    }

    let read_name_len = usize::from(buf[bounds::NAME_LENGTH_INDEX]);
    bounds.name_end = bounds::TEMPLATE_LENGTH_RANGE.end + read_name_len;

    let src = &buf[bounds::CIGAR_OP_COUNT_RANGE];
    // SAFETY: `src` is 2 bytes.
    let cigar_op_count = usize::from(u16::from_le_bytes(src.try_into().unwrap()));
    let cigar_len = mem::size_of::<u32>() * cigar_op_count;
    bounds.cigar_end = bounds.name_end + cigar_len;

    let src = &buf[bounds::READ_LENGTH_RANGE];
    // SAFETY: `src` is 4 bytes.
    let base_count = usize::try_from(u32::from_le_bytes(src.try_into().unwrap()))
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let sequence_len = base_count.div_ceil(2);
    bounds.sequence_end = bounds.cigar_end + sequence_len;

    bounds.quality_scores_end = bounds.sequence_end + base_count;

    if buf.len() < bounds.quality_scores_end {
        Err(io::Error::from(io::ErrorKind::UnexpectedEof))
    } else {
        Ok(())
    }
}
