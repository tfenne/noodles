use std::io::{self, Write};

use crate::crai::Record;

pub(super) fn write_record<W>(writer: &mut W, record: &Record) -> io::Result<()>
where
    W: Write,
{
    writeln!(writer, "{record}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_record() -> Result<(), Box<dyn std::error::Error>> {
        use noodles_core::Position;

        let record = Record::new(Some(0), Position::new(10946), 6765, 17711, 233, 317811);

        let mut buf = Vec::new();
        write_record(&mut buf, &record)?;

        let expected = b"0\t10946\t6765\t17711\t233\t317811\n";
        assert_eq!(buf, expected);

        Ok(())
    }
}
