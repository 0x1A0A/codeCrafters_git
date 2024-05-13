use std::env;
use std::fs;
use std::io::prelude::*;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;
use sha1::Sha1;

fn git_init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
}

fn cat_file(args: Vec<String>) {
    let _option = &args[2];
    let object = &args[3];
    let prefix = &object[0..2];

    let value = fs::File::open(format!(".git/objects/{}/{}", prefix, &object[2..]));
    match value {
        Ok(data) => {
            let mut d = ZlibDecoder::new(data);
            let mut s = String::new();
            d.read_to_string(&mut s).unwrap();

            if let Some(i) = s.find("\u{000}") {
                print!("{}", &s[i + 1..])
            }
        }
        Err(e) => {
            println!("{}", e)
        }
    }
}

fn hash_object(args: Vec<String>) {
    let _option = &args[2];
    let file = &args[3];
    let content = fs::read_to_string(file).unwrap();
    let data = format!("blob {}\0{}", content.len(), content);

    let mut hash = Sha1::new();
    hash.update(data.as_bytes());

    let result = hash.finalize();
    let hash_str = result
        .iter()
        .map(|e| format!("{:02x}", e))
        .collect::<Vec<_>>()
        .join("");

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    e.write_all(data.as_bytes()).unwrap();

    let compressed = e.finish().unwrap();

    let prefix = &hash_str[0..2];

    let _ = fs::create_dir(format!(".git/objects/{}", &prefix));

    let _ = fs::write(
        format!(".git/objects/{}/{}", prefix, &hash_str[2..]),
        compressed,
    );

    print!("{}", hash_str)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => {
            git_init();
            println!("Initialized git directory")
        }
        "cat-file" => cat_file(args),
        "hash-object" => hash_object(args),
        _ => println!("unknown command: {}", args[1]),
    }
}
