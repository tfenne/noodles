use noodles_core::Position;

use crate::crai::{
    Record,
    record::{Field, ParseError},
};

const FIELD_DELIMITER: char = '\t';
const MAX_FIELDS: usize = 6;

pub(crate) fn parse_record(s: &str) -> Result<Record, ParseError> {
    const UNMAPPED: i32 = -1;

    let mut fields = s.splitn(MAX_FIELDS, FIELD_DELIMITER);

    let reference_sequence_id =
        parse_i32(&mut fields, Field::ReferenceSequenceId).and_then(|n| match n {
            UNMAPPED => Ok(None),
            _ => usize::try_from(n)
                .map(Some)
                .map_err(ParseError::InvalidReferenceSequenceId),
        })?;

    let alignment_start = parse_position(&mut fields, Field::AlignmentStart)?;
    let alignment_span = parse_span(&mut fields, Field::AlignmentSpan)?;
    let offset = parse_u64(&mut fields, Field::Offset)?;
    let landmark = parse_u64(&mut fields, Field::Landmark)?;
    let slice_length = parse_u64(&mut fields, Field::SliceLength)?;

    Ok(Record::new(
        reference_sequence_id,
        alignment_start,
        alignment_span,
        offset,
        landmark,
        slice_length,
    ))
}

fn parse_i32<'a, I>(fields: &mut I, field: Field) -> Result<i32, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    fields
        .next()
        .ok_or(ParseError::Missing(field))
        .and_then(|s| s.parse().map_err(|e| ParseError::Invalid(field, e)))
}

fn parse_u64<'a, I>(fields: &mut I, field: Field) -> Result<u64, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    fields
        .next()
        .ok_or(ParseError::Missing(field))
        .and_then(|s| s.parse().map_err(|e| ParseError::Invalid(field, e)))
}

fn parse_position<'a, I>(fields: &mut I, field: Field) -> Result<Option<Position>, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    fields
        .next()
        .ok_or(ParseError::Missing(field))
        .and_then(|s| s.parse().map_err(|e| ParseError::Invalid(field, e)))
        .map(Position::new)
}

fn parse_span<'a, I>(fields: &mut I, field: Field) -> Result<usize, ParseError>
where
    I: Iterator<Item = &'a str>,
{
    fields
        .next()
        .ok_or(ParseError::Missing(field))
        .and_then(|s| s.parse().map_err(|e| ParseError::Invalid(field, e)))
}
