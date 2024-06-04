use std::{collections::VecDeque, io::Read};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub date: u64,
    pub zone: String,
}

impl From<Vec<&str>> for Author {
    fn from(value: Vec<&str>) -> Self {
        Self {
            name: value[0].to_string(),
            email: value[1].to_string(),
            date: u64::from_str_radix(value[2], 10).unwrap(),
            zone: value[3].to_string(),
        }
    }
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

    let mut queue = content.lines().collect::<VecDeque<_>>();
    let mut line = queue.pop_front().unwrap();
    let (head, body) = line.split_once(' ').unwrap();
    assert_eq!(head, "tree");
    let tree = body.to_string();

    let mut parent: Vec<String> = Vec::new();
    loop {
        line = queue.pop_front().unwrap();
        let (head, body) = line.split_once(' ').unwrap();
        if head != "parent" {
            break;
        }
        parent.push(body.to_string());
    }

    let (head, body) = line.split_once(' ').unwrap();
    assert_eq!(head, "author");

    let mut split = body.split(" ").collect::<Vec<_>>();
    assert_eq!(split.len(), 4);
    split[1] = split[1].trim_start_matches('<').trim_end_matches('>');
    let author: Author = From::from(split.clone());

    line = queue.pop_front().unwrap();
    let (head, body) = line.split_once(' ').unwrap();
    assert_eq!(head, "committer");
    let mut split = body.split(" ").collect::<Vec<_>>();
    assert_eq!(split.len(), 4);
    split[1] = split[1].trim_start_matches('<').trim_end_matches('>');
    let committer: Author = From::from(split.clone());

    let _ = queue.pop_front();

    let message = Vec::from(queue).join("");

    Ok(Commit {
        tree,
        parent,
        author,
        committer,
        message,
    })
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::git::commit::Author;

    const COMMIT: &str = "tree 11144a9d4ce9ddea810a3d8b74abbd912e5028b1
parent e1b03b60755972a80dfa8cb02326087d8b38b852
author user <email1994@domain.com> 1717431836 +0700
committer user <email1994@domain.com> 1717431836 +0700

test: Tree parsing
";
    #[test]
    fn commit_parse_with_all() {
        let mut stream = BufReader::new(COMMIT.as_bytes());
        let commit = super::parse(&mut stream);

        assert!(commit.is_ok());

        let commit = commit.unwrap();
        assert_eq!(commit.tree, "11144a9d4ce9ddea810a3d8b74abbd912e5028b1");
        assert_eq!(commit.parent[0], "e1b03b60755972a80dfa8cb02326087d8b38b852");
        assert_eq!(commit.message, "test: Tree parsing");
        assert_eq!(
            commit.author,
            Author {
                name: "user".to_string(),
                email: "email1994@domain.com".to_string(),
                date: 1717431836,
                zone: "+0700".to_string()
            }
        );
    }
}
