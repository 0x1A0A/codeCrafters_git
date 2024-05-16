use core::panic;
use std::io::prelude::*;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

use crate::objects::Kind;

pub fn invoke() {
    let content = create_tree(".".into());

    let mut hasher = Sha1::new();
    hasher.update(content.clone());
    let hash = hasher.finalize();
    let hash = hex::encode(hash);

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&content).unwrap();
    let compressed = e.finish().unwrap();

    let _ = fs::create_dir(format!(".git/objects/{}", &hash[..2]));
    fs::write(
        format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
        compressed,
    )
    .unwrap();

    println!("{}", hash);
}

#[allow(dead_code)]
struct Tree {
    mode: String,
    kind: Kind,
    name: String,
    content: Vec<u8>,
}

fn create_tree(path: PathBuf) -> Vec<u8> {
    let ignore = vec![".git"];

    let entries = match fs::read_dir(path) {
        Ok(dir) => dir,
        Err(err) => panic!("{}", err),
    };

    let mut data: Vec<Tree> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => panic!("{}", err),
        };

        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();

        if ignore.iter().any(|x| x.eq(&name)) {
            continue;
        }

        let metadata = fs::metadata(&path).unwrap();
        let mode = format!("{:o}", metadata.permissions().mode());
        let (content, kind) = match &path.is_dir() {
            false => (blob(path), Kind::Blob),
            true => (create_tree(path), Kind::Tree),
        };

        data.push(Tree {
            mode,
            kind,
            name,
            content,
        });
    }

    let mut content: Vec<u8> = Vec::new();
    for d in data.iter() {
        let mut hasher = Sha1::new();
        hasher.update(d.content.clone());
        let hash = hasher.finalize();
        let mut hash = Vec::from(hash.as_slice());

        let formated = format!("{} {}\0", d.mode, d.name);
        let mut bytes = Vec::from(formated);
        content.append(&mut bytes);
        content.append(&mut hash);
    }

    let header = format!("{} {}\0", Kind::Tree, content.len());
    let mut data = Vec::from(header);

    data.append(&mut content);

    data
}

fn blob(path: PathBuf) -> Vec<u8> {
    let mut content = fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    content.read_to_end(&mut buf).unwrap();

    let header = format!("{} {}\0", Kind::Blob, buf.len());
    let mut data = Vec::from(header);

    data.append(&mut buf);

    data
}
