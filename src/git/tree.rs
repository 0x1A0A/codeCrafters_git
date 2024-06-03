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
        match take(&mut reader) {
            Some(item) => tree.push(item),
            None => break,
        }
    }

    Ok(tree)
}

pub fn take(reader: &mut impl BufRead) -> Option<TreeItem> {
    let mut data = Vec::new();

    if let Ok(consume) = reader.read_until(0x00, &mut data) {
        if consume == 0 {
            return None;
        }
    }

    let mode_name = CStr::from_bytes_with_nul(&data).unwrap();
    let (mode, name) = mode_name.to_str().unwrap().split_once(' ').unwrap();

    let mode = u32::from_str_radix(mode, 8).unwrap();
    let name = name.to_string();

    let mut hash = [0; 20];
    if let Err(_) = reader.read_exact(&mut hash) {
        panic!("Invlalid Tree format!");
    }

    Some(TreeItem { mode, name, hash })
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    #[test]
    fn parse_tree_valid() {
        let mut data = Vec::new();
        data.append(&mut Vec::from("40000 test.txt\0"));
        data.append(&mut hex::decode("aca49a24ef448129fc42e2fb0de2f95f0096d09c").unwrap());
        data.append(&mut Vec::from("100644 README.MD\0"));
        data.append(&mut hex::decode("aca49a24ef448129fc42e2fb0de2f95f0096d09c").unwrap());
        let mut reader = BufReader::new(data.as_slice());

        let result = super::take(&mut reader);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(format!("{:o}", result.mode), "40000");
        assert_eq!(result.name, "test.txt");
        let _ = super::take(&mut reader);
        let result = super::take(&mut reader);
        assert!(result.is_none());
    }

    #[test]
    fn pares_correct_size() {
        let mut data = Vec::new();
        data.append(&mut Vec::from("40000 test.txt\0"));
        data.append(&mut hex::decode("aca49a24ef448129fc42e2fb0de2f95f0096d09c").unwrap());
        data.append(&mut Vec::from("100644 README.MD\0"));
        data.append(&mut hex::decode("aca49a24ef448129fc42e2fb0de2f95f0096d09c").unwrap());
        let mut reader = BufReader::new(data.as_slice());

        let result = super::parse(&mut reader);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
