use chrono::Local;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::objects::Kind;

#[derive(Debug)]
pub struct Options {
    pub message: String,
    pub parent: Option<String>,
}

pub fn invoke(hash: &str, options: Options) {
    let name = "code";
    let email = "crafter@your.git";

    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let unix_time_second = unix_time.as_secs();

    let time_zone = Local::now();
    let tz = time_zone.format("%z");

    let tree = format!("tree {}", hash);
    let author = format!("author {name} <{email}> {unix_time_second} {tz}");
    let commiiter = format!("committer {name} <{email}> {unix_time_second} {tz}");

    let mut content = format!("{tree}");
    if let Some(parent) = options.parent {
        content = format!("{content}\nparent {parent}");
    }

    content = format!("{content}\n{author}\n{commiiter}\n\n{}", options.message);

    let header = format!("{} {}\0", Kind::Commit, content.len());
    println!("{header}{content}");
}
