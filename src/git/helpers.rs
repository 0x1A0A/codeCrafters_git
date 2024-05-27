use std::io::Read;

pub fn read_bytes<const N: usize>(stream: &mut impl Read) -> std::io::Result<[u8; N]> {
    let mut bytes = [0; N];
    stream.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn read_u32(stream: &mut impl Read) -> std::io::Result<u32> {
    let bytes = read_bytes(stream)?;
    Ok(u32::from_be_bytes(bytes))
}

pub fn read_encoded_size(stream: &mut impl Read) -> std::io::Result<(u8, bool)> {
    let [buf] = read_bytes(stream)?;
    let value = buf & 0x7f;
    let more = buf & 0x80 != 0;

    Ok((value, more))
}

pub fn read_size(stream: &mut impl Read) -> std::io::Result<usize> {
    let mut size = 0;
    let mut shift = 0;

    loop {
        let (value, more) = read_encoded_size(stream)?;
        size |= (value as usize) << shift;
        shift += 7;
        if !more {
            break;
        }
    }

    Ok(size)
}

pub fn read_offset(stream: &mut impl Read) -> std::io::Result<usize> {
    let mut offset = 0;

    loop {
        let (value, more) = read_encoded_size(stream)?;
        offset <<= 7;
        offset |= value as usize;
        if !more {
            break;
        }
        offset += 1;
    }

    Ok(offset)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    #[test]
    fn read_from_stream() {
        let data = b"test data here";
        let mut reader = BufReader::new(data.as_slice());

        let first = super::read_bytes::<1>(&mut reader).unwrap();
        assert_eq!(&first, b"t");

        let data: [u8; 4] = super::read_bytes(&mut reader).unwrap();
        assert_eq!(&data, b"est ");
    }

    #[test]
    fn read_u32_from_stream() {
        let data = [0x11, 0x22, 0x33, 0x44];
        let mut reader = BufReader::new(data.as_slice());

        let value = super::read_u32(&mut reader).unwrap();

        assert_eq!(value, 0x11223344);
    }

    #[test]
    fn read_encoded_size() {
        let data = [0x87, 0x92, 0x33, 0x44];
        let mut reader = BufReader::new(data.as_slice());

        let value = super::read_encoded_size(&mut reader).unwrap();
        assert_eq!(value.0, 0x07);
        assert_eq!(value.1, true);

        let value = super::read_encoded_size(&mut reader).unwrap();
        assert_eq!(value.0, 0x12);
        assert_eq!(value.1, true);

        let value = super::read_encoded_size(&mut reader).unwrap();
        assert_eq!(value.0, 0x33);
        assert_eq!(value.1, false);
    }

    #[test]
    fn read_variable_size() {
        let data = [0b10010110, 0b10111000, 0b01100001, 0x44];
        let mut reader = BufReader::new(data.as_slice());
        let size = super::read_size(&mut reader).unwrap();

        assert_eq!(size, 0b110_0001_011_1000_001_0110); // 0x185c16)
    }

    #[test]
    fn read_offset() {
        let data = [0b10010110, 0b10111000, 0b01100001, 0x44];
        let mut reader = BufReader::new(data.as_slice());
        let offset = super::read_offset(&mut reader).unwrap();

        assert_eq!(offset, 0b001_0110_011_1000_110_0001 + (1 << 7) + (1 << 14));
    }
}
