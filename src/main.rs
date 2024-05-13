#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::BufReader;
use std::io::Read;

use flate2::bufread::ZlibDecoder;

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
            let reader = BufReader::new(data);
            let mut d = ZlibDecoder::new(reader);
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

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => {
            git_init();
            println!("Initialized git directory")
        }
        "cat-file" => cat_file(args),
        _ => println!("unknown command: {}", args[1]),
    }
}
