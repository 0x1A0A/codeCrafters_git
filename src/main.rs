use std::env;
use std::fs;
use std::io::prelude::*;
use std::usize;

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

fn extract_header(_t: u8, buffer: &[u8]) -> (String, usize, usize) {
    let mut i = 0;

    let mut stage = 0;
    let mut object_type = String::new();
    let mut file_size: usize = 0;

    loop {
        match buffer[i] {
            0 => {
                break;
            }
            32 => {
                stage = 1;
            }
            c => {
                if stage == 1 {
                    file_size <<= 8;
                    file_size |= c as usize;
                } else {
                    object_type.push(char::from(c));
                }
            }
        }
        i += 1;
    }

    (object_type, file_size, i)
}

fn extract_tree_content(buffer: &[u8]) -> (String, String, Vec<u8>, usize) {
    let mut i = 0;
    loop {
        if buffer[i] == 32 {
            break;
        }
        i += 1;
    }
    let mode_byte = &buffer[0..i];
    let mut name = String::new();
    i += 1;

    loop {
        if buffer[i] == 0 {
            break;
        }

        name.push(char::from(buffer[i]));
        i += 1;
    }

    i += 1;

    let mode = String::from_utf8(mode_byte.to_vec()).unwrap();

    let hash = Vec::from(&buffer[i..i + 20]);

    (mode, name, hash, i + 20)
}

fn ls_tree(args: Vec<String>) {
    let _options = &args[1..args.len() - 1];
    let file = &args[args.len() - 1];
    let value = fs::File::open(format!(".git/objects/{}/{}", &file[0..2], &file[2..]));
    match value {
        Ok(data) => {
            let mut d = ZlibDecoder::new(data);
            let mut s: Vec<u8> = Vec::new();

            d.read_to_end(&mut s).unwrap();

            let (object_type, _, mut index) = extract_header(0, &s);
            index += 1;

            loop {
                let buffer = &s[index..];
                if buffer.len() == 0 {
                    break;
                }

                let (mode, name, hash, s) = extract_tree_content(&s[index..]);
                index += s;

                let hash_str = hash
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<Vec<_>>()
                    .join("");

                if _options.iter().any(|x| x.eq("--name-only")) {
                    println!("{name}");
                } else {
                    println!("{:0>6} {object_type} {hash_str} {name}", mode.trim());
                }
            }
        }
        Err(e) => {
            println!("{}", e)
        }
    }
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
        "ls-tree" => ls_tree(args),
        _ => println!("unknown command: {}", args[1]),
    }
}
