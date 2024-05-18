use core::panic;
use reqwest::blocking::Client;
#[allow(unused)]
use sha1::{Digest, Sha1};
use std::{fs, io::Read, path::PathBuf};

#[derive(Debug)]
pub struct Options {
    pub dir: Option<PathBuf>,
}

#[allow(unused)]
pub fn invoke(url: &str, options: Options) {
    let client = Client::builder().build().unwrap();

    let mut resp = client
        .get(format!("{url}/info/refs"))
        .query(&[("service", "git-upload-pack")])
        .send()
        .unwrap();

    let mut state = 0;

    let mut pack_line: Vec<PktLine> = Vec::new();

    loop {
        let mut length: [u8; 4] = [0; 4];
        resp.read_exact(&mut length).unwrap();

        let s = String::from_utf8(length.to_vec()).unwrap();
        let length = usize::from_str_radix(&s, 16).unwrap();

        if length == 0 {
            state += 1
        }

        if length != 0 {
            let mut buf = vec![0; length - 4];
            resp.read_exact(&mut buf).unwrap();
            pack_line.push(PktLine {
                length,
                value: buf.to_vec(),
            });
        }

        if state == 2 {
            break;
        }
    }

    // we might need to init a git first
    // the first response is use to request a pack upload
    let Some((first, elements)) = pack_line[1..].split_first() else {
        panic!("invalid size for discover!");
    };

    let hash = &first.value[..40];
    let hash = String::from_utf8(hash.to_vec()).unwrap();

    let body = format!("0054want {hash} multi_ack side-band-64k ofs-delta\n00000009done\n");
    let request = Vec::from(body.clone());

    let rq = client
        .post(format!("{url}/git-upload-pack"))
        .header("Accept", "application/x-git-upload-pack-result")
        .header("Content-Type", "application/x-git-upload-pack-request")
        .body(request);

    let mut resp = rq.send().unwrap();

    state = 1;

    let mut packfile = Vec::new();

    loop {
        let mut length: [u8; 4] = [0; 4];
        resp.read_exact(&mut length).unwrap();

        let s = String::from_utf8(length.to_vec()).unwrap();
        let length = usize::from_str_radix(&s, 16).unwrap();

        if length == 0 {
            state += 1
        }

        if length != 0 {
            let mut buf = vec![0; length - 4];
            resp.read_exact(&mut buf).unwrap();
            if buf.starts_with(&[0b1]) {
                packfile.append(&mut buf[1..].to_vec());
            }
            
            if buf.starts_with(&[2]) {
                std::io::copy(&mut buf[1..].to_vec().as_slice(),&mut std::io::stdout());
            }
        }

        if state == 2 {
            break;
        }
    }

    for pkt in elements {
        let value = String::from_utf8(pkt.value.to_vec()).unwrap();
        let Some((hash, path)) = value.split_once(' ') else {
            panic!("unknow format of pkt-line");
        };

        let _ = fs::create_dir_all(".git/refs/heads");
        let _ = fs::write(format!(".git/{}", path.trim()), hash);
    }

    crate::git::packfile_parse(&packfile);

    let packsum = &packfile[packfile.len() - 20..];
    let packsum = hex::encode(packsum);

    fs::write(format!("pack_{packsum}.pack"), packfile).unwrap();
}

#[allow(unused)]
struct PktLine {
    length: usize,
    value: Vec<u8>,
}
