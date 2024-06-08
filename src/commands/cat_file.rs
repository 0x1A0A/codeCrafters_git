use core::panic;

use crate::git::objects::Object;

#[derive(Debug)]
pub struct Options {
    pub pretty: bool,
}

pub fn invoke(hash: &str, options: Options) {
    let data = match Object::read_from_hash(hash) {
        Ok(ok) => ok,
        Err(e) => panic!("{}", e),
    };

    if options.pretty == false {
        panic!("we now only support pretty print");
    }

    data.cat();
}
