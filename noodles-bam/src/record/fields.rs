//! BAM record fields.

pub(crate) mod bounds;

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
