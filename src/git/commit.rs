use core::panic;
use std::io::Read;

#[derive(Debug, Default)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub date: u64,
    pub zone: String,
}

#[derive(Debug)]
pub struct Commit {
    pub tree: String,
    pub parent: Vec<String>,
    pub author: Author,
    pub committer: Author,
    pub message: String,
}

pub fn parse(stream: &mut impl Read) -> std::io::Result<Commit> {
    let mut content = String::new();
    stream.read_to_string(&mut content)?;

    let mut tree = String::new();
    let mut parent: Vec<String> = Vec::new();
    let mut author: Author = Default::default();
    let mut committer: Author = Default::default();
    let mut message = String::new();

    for line in content.lines().filter(|x| !x.trim().is_empty()) {
        let split = line.split(' ').collect::<Vec<_>>();
        let Some((head, body)) = split.split_first() else {
            panic!("incorrect lines -- {line}")
        };

        match *head {
            "tree" => {
                tree = body.first().unwrap().to_string();
            }
            "parent" => {
                let hash = body.first().unwrap().to_string();
                parent.push(hash);
            }
            "author" => {}
            "committer" => {}
            _ => {
                message.push_str(&line);
            }
        };
    }

    Ok(Commit {
        tree,
        parent,
        author,
        committer,
        message,
    })
}
