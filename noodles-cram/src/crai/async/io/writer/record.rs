use tokio::io::{self, AsyncWrite, AsyncWriteExt};

use crate::crai::Record;

pub(super) async fn write_record<W>(writer: &mut W, record: &Record) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    const LINE_FEED: u8 = b'\n';

    writer.write_all(record.to_string().as_bytes()).await?;
    writer.write_all(&[LINE_FEED]).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_record() -> Result<(), Box<dyn std::error::Error>> {
        use noodles_core::Position;

        let record = Record::new(Some(0), Position::new(10946), 6765, 17711, 233, 317811);

        let mut buf = Vec::new();
        write_record(&mut buf, &record).await?;

        let expected = b"0\t10946\t6765\t17711\t233\t317811\n";
        assert_eq!(buf, expected);

        Ok(())
    }
}
