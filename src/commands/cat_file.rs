use core::panic;

use crate::objects::{Kind, Object};

#[derive(Debug)]
pub struct Options {
    pub pretty: bool,
}

pub fn invoke(hash: &str, options: Options) {
    let mut data = match Object::read(hash) {
        Ok(ok) => ok,
        Err(e) => panic!("{}", e),
    };

    if options.pretty == false {
        panic!("we now only support pretty print");
    }

    match data.kind {
        Kind::Blob => {
            let _ = std::io::copy(&mut data.content, &mut std::io::stdout());
        }
        k => panic!("unknow how to print this kind {k}"),
    }
}
