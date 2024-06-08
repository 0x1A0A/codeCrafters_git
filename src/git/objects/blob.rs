use std::io::Read;

#[derive(Debug, PartialEq, Eq)]
pub struct Blob(pub Vec<u8>);

pub fn parse(stream: &mut impl Read) -> std::io::Result<Blob> {
    let mut data = Vec::new();
    stream.read_to_end(&mut data)?;
    Ok(Blob(data))
}
