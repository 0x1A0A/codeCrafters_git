use flate2::read::ZlibDecoder;
use std::io::{Read, Seek, SeekFrom};

use super::helpers;

#[derive(Debug, PartialEq, Eq)]
pub enum ObjType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl std::fmt::Display for ObjType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjType::Blob => write!(f, "blob"),
            ObjType::Tree => write!(f, "tree"),
            ObjType::Commit => write!(f, "commit"),
            ObjType::Tag => write!(f, "tag"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PackObjType {
    Base(ObjType),
    OFS,
    REF,
}

pub fn read_object(
    stream: &mut (impl Seek + Read),
    offset: Option<usize>,
) -> std::io::Result<(Vec<u8>, ObjType)> {
    let offset = match offset {
        Some(v) => v,
        None => stream.stream_position()? as usize,
    };

    stream.seek(SeekFrom::Start(offset as u64))?;

    let size_and_type = helpers::read_size(stream)?;
    let (_, t) = extract_size_and_type(size_and_type);

    let t = match t {
        0b001 => PackObjType::Base(ObjType::Commit),
        0b010 => PackObjType::Base(ObjType::Tree),
        0b011 => PackObjType::Base(ObjType::Blob),
        0b100 => PackObjType::Base(ObjType::Tag),
        0b110 => PackObjType::OFS,
        0b111 => PackObjType::REF,
        x => panic!("{x:03b} is invalid type."),
    };

    let (content, content_type) = match t {
        PackObjType::Base(base) => {
            let current = stream.stream_position()?;
            let mut content = Vec::new();
            let consume = decompress(stream, &mut content)?;

            stream.seek(SeekFrom::Start(current + consume as u64))?;

            (content, base)
        }
        PackObjType::OFS => {
            let negativeoffset = helpers::read_offset(stream)?;
            let current = stream.stream_position()?;

            let (base, base_type) = read_object(stream, Some(offset - negativeoffset))?;
            let mut content = Vec::new();

            stream.seek(SeekFrom::Start(current))?;
            let consume = decompress(stream, &mut content)?;
            stream.seek(SeekFrom::Start(current + consume as u64))?;

            (base, base_type)
        }
        PackObjType::REF => {
            unimplemented!()
        }
    };

    Ok((content, content_type))
}

pub fn headers(stream: &mut impl Read) -> std::io::Result<([u8; 4], u32, u32)> {
    let header = helpers::read_bytes(stream)?;
    let version = helpers::read_u32(stream)?;
    let entries = helpers::read_u32(stream)?;
    assert_eq!(&header, b"PACK");
    assert_ne!(version, 0);
    assert_ne!(entries, 0);

    Ok((header, version, entries))
}

fn extract_size_and_type(size_and_type: usize) -> (usize, u8) {
    let t = (size_and_type & 0x70) >> 4;
    let size = ((size_and_type >> 3) & !0xf) | (size_and_type & 0xf);
    (size, t as u8)
}

fn decompress(stream: &mut impl Read, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
    let mut z = ZlibDecoder::new(stream);
    z.read_to_end(buffer)?;

    Ok(z.total_in() as usize)
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

    #[test]
    fn extract_file_type_and_size() {
        let (size, obj_type) = super::extract_size_and_type(0b110_101_0110);
        assert_eq!(size, 0b110_0110);
        assert_eq!(obj_type, 0b101);
    }
}
