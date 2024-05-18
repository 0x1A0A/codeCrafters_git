use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use sha1::Digest;
use sha1::Sha1;

use flate2::write::ZlibEncoder;
use flate2::Compression;

#[derive(Debug)]
pub struct Options {
    pub write: bool,
}

pub fn invoke(path: PathBuf, options: Options) -> String {
    let content = fs::read_to_string(path).unwrap();
    let data = format!("blob {}\0{}", content.len(), content);

    let mut hasher = Sha1::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();
    let hash = hex::encode(hash);
 
    if options.write == true {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(data.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();

        let _ = fs::create_dir(format!(".git/objects/{}", &hash[..2]));
        let _ = fs::write(
            format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
            compressed,
        );
    }

    println!("{}", hash);

    hash
}
