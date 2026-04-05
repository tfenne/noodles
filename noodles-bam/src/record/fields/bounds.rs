use std::ops::Range;

pub const ALIGNMENT_START_RANGE: Range<usize> = 4..8;
pub const NAME_LENGTH_INDEX: usize = 8;
pub const MAPPING_QUALITY_INDEX: usize = 9;
pub const CIGAR_OP_COUNT_RANGE: Range<usize> = 12..14;
pub const FLAGS_RANGE: Range<usize> = 14..16;
pub const READ_LENGTH_RANGE: Range<usize> = 16..20;
pub const MATE_REFERENCE_SEQUENCE_ID_RANGE: Range<usize> = 20..24;
pub const MATE_ALIGNMENT_START_RANGE: Range<usize> = 24..28;
pub const TEMPLATE_LENGTH_RANGE: Range<usize> = 28..32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bounds {
    pub name_end: usize,
    pub cigar_end: usize,
    pub sequence_end: usize,
    pub quality_scores_end: usize,
}
