use blob::Blob;
use commit::Commit;
use tree::TreeItem;

use flate2::read::ZlibDecoder;
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader},
};

pub mod blob;
pub mod commit;
pub mod tree;

#[derive(Debug, PartialEq, Eq)]
pub enum Object {
    Blob(Blob),
    Tree(Vec<TreeItem>),
    Commit(Commit),
}

impl Object {
    pub fn read_from_hash(hash: &str) -> std::io::Result<Object> {
        let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
        let data = fs::File::open(path)?;

        let d = ZlibDecoder::new(data);
        let mut buff = Vec::new();
        let mut d = BufReader::new(d);

        d.read_until(0, &mut buff).unwrap();

        let header = CStr::from_bytes_until_nul(&buff).unwrap();
        let header = header.to_str().unwrap();

        let Some((kind, _)) = header.split_once(' ') else {
            panic!("Unexpected header format!");
        };

        match kind {
            "blob" => {
                let obj = blob::parse(&mut d)?;
                Ok(Object::Blob(obj))
            }
            "tree" => {
                let obj = tree::parse(&mut d)?;
                Ok(Object::Tree(obj))
            }
            "commit" => {
                let obj = commit::parse(&mut d)?;
                Ok(Object::Commit(obj))
            }
            k => panic!("Unknow object type {k}"),
        }
    }

    pub fn cat(&self) {
        match self {
            Object::Blob(obj) => {
                let data = &obj.0;
                match String::from_utf8(data.to_vec()) {
                    Ok(s) => print!("{s}"),
                    Err(e) => print!("{e}"),
                };
            }
            Object::Tree(obj) => {
                for t in obj.iter() {
                    let hash = hex::encode(t.hash);
                    let kind = match Self::read_from_hash(&hash).unwrap() {
                        Self::Blob(_) => "blob",
                        Self::Tree(_) => "tree",
                        Self::Commit(_) => "commit",
                    };

                    println!("{:06o} {kind} {} {}", t.mode, hash, t.name);
                }
            }
            Object::Commit(obj) => {
                println!("tree {}", obj.tree);
                for parent in obj.parents.iter() {
                    println!("parent {}", parent);
                }

                let author = &obj.author;
                println!(
                    "author {} <{}> {} {}",
                    author.name, author.email, author.date, author.zone
                );

                let committer = &obj.committer;
                println!(
                    "committer {} <{}> {} {}",
                    committer.name, committer.email, committer.date, committer.zone
                );
            }
        }
    }
}
