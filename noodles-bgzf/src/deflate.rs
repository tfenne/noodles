use std::io;

use flate2::Crc;

#[cfg(feature = "libdeflate")]
pub(crate) fn decode(src: &[u8], dst: &mut [u8]) -> io::Result<()> {
    use libdeflater::Decompressor;

    let mut decoder = Decompressor::new();

    decoder
        .deflate_decompress(src, dst)
        .map(|_| ())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(not(feature = "libdeflate"))]
pub(crate) fn decode(src: &[u8], dst: &mut [u8]) -> io::Result<()> {
    use zlib_rs::{Inflate, InflateFlush, Status};

    const HAS_ZLIB_HEADER: bool = false;
    const WINDOW_BITS: u8 = 15;

    let mut decoder = Inflate::new(HAS_ZLIB_HEADER, WINDOW_BITS);

    let status = decoder
        .decompress(src, dst, InflateFlush::Finish)
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

    if status == Status::StreamEnd {
        Ok(())
    } else {
        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}

#[cfg(feature = "libdeflate")]
pub(crate) fn encode(
    src: &[u8],
    compression_level: libdeflater::CompressionLvl,
    dst: &mut Vec<u8>,
) -> io::Result<u32> {
    use libdeflater::Compressor;

    let mut encoder = Compressor::new(compression_level);

    let max_len = encoder.deflate_compress_bound(src.len());
    dst.resize(max_len, 0);

    let len = encoder
        .deflate_compress(src, dst)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    dst.truncate(len);

    let mut crc = Crc::new();
    crc.update(src);

    Ok(crc.sum())
}

#[cfg(not(feature = "libdeflate"))]
pub(crate) fn encode(
    src: &[u8],
    compression_level: flate2::Compression,
    dst: &mut Vec<u8>,
) -> io::Result<u32> {
    use std::io::Write;

    use flate2::write::DeflateEncoder;

    dst.clear();

    let mut encoder = DeflateEncoder::new(dst, compression_level);
    encoder.write_all(src)?;
    encoder.finish()?;

    let mut crc = Crc::new();
    crc.update(src);

    Ok(crc.sum())
}
