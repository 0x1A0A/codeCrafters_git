use core::panic;
use reqwest::blocking::Client;
use sha1::{Digest, Sha1};
use std::{
    collections::{self, VecDeque},
    fs,
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use crate::git::{
    commit,
    packfile::{headers, read_object, ObjType},
    tree,
};

use super::git_init;

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

    git_init::invoke();

    let Some((first, elements)) = pack_line[1..].split_first() else {
        panic!("invalid size for discover!");
    };

    let hash = &first.value[..40];
    let hash = String::from_utf8(hash.to_vec()).unwrap();
    let ref_head = hash.clone();

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
                std::io::copy(&mut buf[1..].to_vec().as_slice(), &mut std::io::stdout());
            }
        }

        if state == 2 {
            break;
        }
    }

    for pkt in elements {
        let value = String::from_utf8(pkt.value.to_vec()).unwrap();
        println!("{}", value);
        let Some((hash, path)) = value.split_once(' ') else {
            panic!("unknow format of pkt-line");
        };

        let _ = fs::create_dir_all(".git/refs/heads");
        let mut hash = hash.to_string();
        hash.push('\n');
        let _ = fs::write(format!(".git/{}", path.trim()), &hash);
    }

    let mut stream = std::io::Cursor::new(packfile.clone());
    let (_, _, entries) = headers(&mut stream).unwrap();

    let mut map = collections::HashMap::new();

    for i in 0..entries {
        let (content, content_type) = read_object(&mut stream, None).unwrap();
        let mut hasher = Sha1::new();
        let header = format!("{} {}\0", content_type, content.len());
        hasher.update(header);
        hasher.update(content.clone());
        let hash = hasher.finalize();
        let hash = hex::encode(hash);
        map.insert(hash, (content, content_type));
    }

    let head = map.get(&ref_head).unwrap();
    let commit = commit::parse(&mut head.0.as_slice()).unwrap();

    let mut queue: VecDeque<(String, PathBuf)> = VecDeque::new();
    queue.push_back((commit.tree, ".".into()));

    loop {
        if queue.is_empty() {
            break;
        }
        let (hash, path) = queue.pop_front().unwrap();
        let tree = map.get(&hash).unwrap();
        let tree = tree::parse(&mut tree.0.as_slice()).unwrap();
        fs::create_dir_all(path.clone()).unwrap();

        for item in tree {
            let hash = hex::encode(item.hash);
            let (content, obj_type) = map.get(&hash).unwrap();
            let mut path = path.clone();
            path.push(item.name);
            match obj_type {
                ObjType::Tree => {
                    queue.push_back((hash, path));
                }
                ObjType::Blob => {
                    let mut file = fs::File::create(path.clone()).unwrap();
                    let mut perm = file.metadata().unwrap().permissions();
                    perm.set_mode(item.mode);
                    file.write(content);
                    file.set_permissions(perm);
                }
                _ => unimplemented!(),
            }
        }
    }

    let mut hash = [0; 20];
    stream.read_exact(&mut hash).unwrap();

    fs::create_dir_all(format!(".git/objects/pack"));
    fs::write(
        format!(".git/objects/pack/pack_{}.pack", hex::encode(hash)),
        packfile,
    )
    .unwrap();
}

#[allow(unused)]
struct PktLine {
    length: usize,
    value: Vec<u8>,
}
