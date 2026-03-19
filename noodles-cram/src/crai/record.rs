//! CRAM index record and fields.

use std::{fmt, io, str::FromStr};

use noodles_core::Position;

/// A CRAM index record.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Record {
    reference_sequence_id: Option<usize>,
    alignment_start: Option<Position>,
    alignment_span: usize,
    offset: u64,
    landmark: u64,
    slice_length: u64,
}

impl Record {
    /// Creates a CRAM index record.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    /// ```
    pub fn new(
        reference_sequence_id: Option<usize>,
        alignment_start: Option<Position>,
        alignment_span: usize,
        offset: u64,
        landmark: u64,
        slice_length: u64,
    ) -> Self {
        Self {
            reference_sequence_id,
            alignment_start,
            alignment_span,
            offset,
            landmark,
            slice_length,
        }
    }

    /// Returns the reference sequence ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    ///
    /// assert_eq!(record.reference_sequence_id(), Some(0));
    /// ```
    pub fn reference_sequence_id(&self) -> Option<usize> {
        self.reference_sequence_id
    }

    /// Returns the alignment start.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    ///
    /// assert_eq!(record.alignment_start(), Position::new(10946));
    /// ```
    pub fn alignment_start(&self) -> Option<Position> {
        self.alignment_start
    }

    /// Returns the alignment span.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    ///
    /// assert_eq!(record.alignment_span(), 6765);
    /// ```
    pub fn alignment_span(&self) -> usize {
        self.alignment_span
    }

    /// Returns the offset of the container from the start of the stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    ///
    /// assert_eq!(record.offset(), 17711);
    /// ```
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns the offset of the slice from the start of the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    ///
    /// assert_eq!(record.landmark(), 233);
    /// ```
    pub fn landmark(&self) -> u64 {
        self.landmark
    }

    /// Returns the size of the slice in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_core::Position;
    /// use noodles_cram::crai;
    ///
    /// let record = crai::Record::new(
    ///     Some(0),
    ///     Position::new(10946),
    ///     6765,
    ///     17711,
    ///     233,
    ///     317811,
    /// );
    ///
    /// assert_eq!(record.slice_length(), 317811);
    /// ```
    pub fn slice_length(&self) -> u64 {
        self.slice_length
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const UNMAPPED: i32 = -1;

        if let Some(id) = self.reference_sequence_id() {
            write!(f, "{id}\t")?;
        } else {
            write!(f, "{UNMAPPED}\t")?;
        };

        let alignment_start = self.alignment_start().map(usize::from).unwrap_or_default();

        write!(
            f,
            "{}\t{}\t{}\t{}\t{}",
            alignment_start, self.alignment_span, self.offset, self.landmark, self.slice_length
        )
    }
}

impl FromStr for Record {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        super::io::reader::parse_record(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let record = Record::new(None, Position::new(10946), 6765, 17711, 233, 317811);
        let actual = record.to_string();
        let expected = "-1\t10946\t6765\t17711\t233\t317811";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_from_str() -> Result<(), Box<dyn std::error::Error>> {
        let actual: Record = "0\t10946\t6765\t17711\t233\t317811".parse()?;

        let expected = Record {
            reference_sequence_id: Some(0),
            alignment_start: Position::new(10946),
            alignment_span: 6765,
            offset: 17711,
            landmark: 233,
            slice_length: 317811,
        };

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_from_str_with_invalid_records() {
        assert!(matches!(
            "0\t10946".parse::<Record>(),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        assert!(matches!(
            "0\t10946\tnoodles".parse::<Record>(),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));

        assert!(matches!(
            "-8\t10946\t6765\t17711\t233\t317811".parse::<Record>(),
            Err(e) if e.kind() == io::ErrorKind::InvalidData
        ));
    }
}
