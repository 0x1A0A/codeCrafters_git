use chrono::Local;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::prelude::*;
use std::fs;

use sha1::{Digest, Sha1};
use crate::objects::Kind;
use flate2::{write::ZlibEncoder, Compression};

#[derive(Debug)]
pub struct Options {
    pub message: String,
    pub parent: Option<String>,
}

pub fn invoke(hash: &str, options: Options) {
    let name = "code";
    let email = "crafter@your.git";

    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let unix_time_second = unix_time.as_secs();

    let time_zone = Local::now();
    let tz = time_zone.format("%z");

    let tree = format!("tree {}", hash);
    let author = format!("author {name} <{email}> {unix_time_second} {tz}");
    let commiiter = format!("committer {name} <{email}> {unix_time_second} {tz}");

    let mut content = format!("{tree}");
    if let Some(parent) = options.parent {
        content = format!("{content}\nparent {parent}");
    }

    content = format!("{content}\n{author}\n{commiiter}\n\n{}\n", options.message);

    let header = format!("{} {}\0", Kind::Commit, content.len());

    let mut hasher = Sha1::new();
    hasher.update(header.clone());
    hasher.update(content.clone());

    let hash = hasher.finalize();
    let hash = hex::encode(hash);


    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(header.as_bytes()).unwrap();
    e.write_all(content.as_bytes()).unwrap();
    let compressed = e.finish().unwrap();

    let _ = fs::create_dir(format!(".git/objects/{}", &hash[..2]));
    fs::write(
        format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
        compressed,
    )
    .unwrap();

    println!("{hash}");
}
