use std::io::{Read, Seek, SeekFrom};

use super::helpers;

fn headers(stream: &mut impl Read) -> std::io::Result<([u8; 4], u32, u32)> {
    let header = helpers::read_bytes(stream)?;
    let version = helpers::read_u32(stream)?;
    let entries = helpers::read_u32(stream)?;
    assert_eq!(&header, b"PACK");
    assert_ne!(version, 0);
    assert_ne!(entries, 0);

    Ok((header, version, entries))
}

fn read_object(stream: &mut (impl Seek + Read), pos: usize) -> std::io::Result<()> {
    stream.seek(SeekFrom::Start(pos as u64))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::headers;

    #[test]
    fn valid_packfile_header() {
        let data = [*b"PACK", *b"\x00\x00\x00\x02", *b"\x00\x00\x00\x02"].concat();

        let mut reader = BufReader::new(data.as_slice());
        assert!(headers(&mut reader).is_ok());
    }

    #[test]
    #[should_panic]
    fn invalid_packfile_header() {
        let data = [*b"HEAD", *b"\x00\x00\x00\x02", *b"\x00\x00\x00\x02"].concat();

        let mut reader = BufReader::new(data.as_slice());
        let _ = headers(&mut reader);
    }
}
