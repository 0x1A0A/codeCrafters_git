use flate2::read::ZlibDecoder;
use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
};

use super::helpers;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    offset: usize,
    cache: &HashMap<usize, (Vec<u8>, ObjType)>,
) -> std::io::Result<(Vec<u8>, ObjType)> {
    stream.seek(SeekFrom::Start(offset as u64))?;

    let size_and_type = helpers::read_size(stream)?;
    let (size, t) = extract_size_and_type(size_and_type);

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

            let base_offset = offset - negativeoffset;
            let cached = cache.get(&base_offset);

            let (base, base_type) = match cached {
                Some(c) => {
                    let (a, t) = c;
                    (a.clone(), t.clone())
                }
                None => read_object(stream, base_offset, cache)?,
            };

            let mut content = Vec::new();

            stream.seek(SeekFrom::Start(current))?;
            let consume = decompress(stream, &mut content)?;
            stream.seek(SeekFrom::Start(current + consume as u64))?;
            let content = process_delta(&mut content, &base)?;

            (content, base_type)
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

fn process_delta(delta: &mut Vec<u8>, base: &Vec<u8>) -> std::io::Result<Vec<u8>> {
    let mut reader = delta.as_slice();
    let _ = helpers::read_size(&mut reader)?;
    let content_size = helpers::read_size(&mut reader)?;
    let mut content: Vec<u8> = Vec::new();

    loop {
        if content.len() == content_size {
            break;
        }

        let [instruction] = helpers::read_bytes::<1>(&mut reader)?;
        if instruction & 0x80 != 0 {
            let (offset, size) = read_offset_and_size(&mut reader, instruction)?;
            let copy = &base[offset..offset + size];
            let mut source = Vec::from(copy);
            content.append(&mut source);
        } else {
            let size = instruction & 0x7f;
            let mut source = vec![0; size as usize];
            source.reserve_exact(size as usize);
            reader.read_exact(&mut source)?;
            content.append(&mut source);
        }
    }

    Ok(content)
}

fn read_offset_and_size(stream: &mut impl Read, flag: u8) -> std::io::Result<(usize, usize)> {
    let offset_flag = flag & 0xf;
    let mut offset: usize = 0;
    for i in 0..4 {
        if offset_flag & (1 << i) != 0 {
            let [buf] = helpers::read_bytes::<1>(stream)?;
            offset |= (buf as usize) << i * 8;
        }
    }

    let size_flag = (flag & 0x70) >> 4;
    let mut size = 0;
    for i in 0..3 {
        if size_flag & (1 << i) != 0 {
            let [buf] = helpers::read_bytes::<1>(stream)?;
            size |= (buf as usize) << i * 8;
        }
    }

    Ok((offset, size))
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

    #[test]
    fn read_offset_and_size() {
        let data = vec![0x10, 0x80, 0xff];
        let (offset, size) =
            super::read_offset_and_size(&mut data.as_slice(), 0b1_010_0101).unwrap();

        assert_eq!(offset, 0x800010);
        assert_eq!(size, 0xff00);
    }
}
