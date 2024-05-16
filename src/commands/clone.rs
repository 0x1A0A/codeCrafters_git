use reqwest::blocking::Client;
use std::{io::Read, path::PathBuf};

#[derive(Debug)]
pub struct Options {
    pub dir: Option<PathBuf>,
}

pub fn invoke(url: &str, options: Options) {
    let client = Client::builder().build().unwrap();

    let mut resp = client
        .get(format!("{url}/info/refs"))
        .query(&[("service", "git-upload-pack")])
        .send()
        .unwrap();

    let mut state = 0;

    loop {
        let mut length: [u8; 4] = [0; 4];
        resp.read_exact(&mut length).unwrap();

        let s = String::from_utf8(length.to_vec()).unwrap();
        let length = usize::from_str_radix(&s, 16).unwrap();

        if length == 0 {
            state += 1
        }

        print!("{s} {length} ");
        if length != 0 {
            let mut buf = vec![0; length - 4];
            resp.read_exact(&mut buf).unwrap();
            let value = String::from_utf8(buf).unwrap();
            print!("{value}");
        }

        if state == 2 {
            break;
        }
    }
}

struct PktLine {
    length: usize,
    value: [u8],
}
