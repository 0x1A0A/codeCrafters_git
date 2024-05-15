use core::panic;
use std::{
    ffi::CStr,
    io::{BufRead, Read},
};

use crate::objects::{Kind, Object};

#[derive(Debug)]
pub struct Options {
    pub name_only: bool,
}

pub fn invoke(tree_hash: &str, options: Options) {
    let data = match Object::read(tree_hash) {
        Ok(ok) => ok,
        Err(e) => panic!("{}", e),
    };

    let mut content = if data.kind == Kind::Tree {
        data.content
    } else {
        panic!("work on tree");
    };

    let mut objects: Vec<(String, String, [u8; 20])> = Vec::new();
    let mut size = 0;

    loop {
        let mut buf = Vec::new();
        size += content.read_until(0, &mut buf).unwrap();

        let mode_name = CStr::from_bytes_until_nul(&buf).unwrap();
        let mode_name = mode_name.to_str().unwrap();

        let Some((mode, name)) = mode_name.split_once(' ') else {
            panic!("wrong file format");
        };

        let mut hash = [0; 20];
        content.read_exact(&mut hash).unwrap();
        size += 20;
        objects.push((mode.to_string(), name.to_string(), hash));

        if size == data.expected_size {
            break;
        }
    }

    for i in objects.iter() {
        let out;

        if options.name_only == true {
            out = format!("{}", i.1);
        } else {
            out = format!("{:0>6} {} {} {}", i.0, "kind", hex::encode(i.2), i.1);
        }

        println!("{}", out);
    }
}
