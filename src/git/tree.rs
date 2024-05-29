use std::{
    ffi::CStr,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug)]
pub struct TreeItem {
    pub mode: u32,
    pub name: String,
    pub hash: [u8; 20],
}

pub fn parse(stream: &mut impl Read) -> std::io::Result<Vec<TreeItem>> {
    let mut reader = BufReader::new(stream);

    let mut tree = Vec::new();

    loop {
        let mut data = Vec::new();
        let consume = reader.read_until(0x00, &mut data)?;

        if consume == 0 {
            break;
        }
        let mode_name = CStr::from_bytes_with_nul(&data).unwrap();
        let (mode, name) = mode_name.to_str().unwrap().split_once(' ').unwrap();
        let mode = u32::from_str_radix(mode, 8).unwrap();
        let name = name.to_string();
        let mut hash = [0; 20];
        reader.read_exact(&mut hash)?;
        tree.push(TreeItem { mode, name, hash });
    }

    Ok(tree)
}
