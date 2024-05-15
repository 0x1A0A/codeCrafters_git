use core::panic;
use std::{
    error::Error,
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read},
    usize,
};

use flate2::read::ZlibDecoder;

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub struct Object<R> {
    pub kind: Kind,
    pub expected_size: usize,
    pub content: R,
}

impl Object<()> {
    pub fn read(hash: &str) -> Result<Object<impl BufRead>, Box<dyn Error>> {
        let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
        let data = fs::File::open(path)?;

        let d = ZlibDecoder::new(data);
        let mut buff = Vec::new();
        let mut d = BufReader::new(d);

        d.read_until(0, &mut buff).unwrap();

        let header = CStr::from_bytes_until_nul(&buff)?;
        let header = header.to_str()?;

        let Some((kind, size)) = header.split_once(' ') else {
            panic!("Unexpected header format!");
        };

        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            k => panic!("Unknow object type {k}"),
        };

        let expected_size = size.parse::<usize>().unwrap();
        let content = d.take(expected_size as u64);

        Ok(Object {
            kind,
            expected_size,
            content,
        })
    }
}
