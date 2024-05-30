use std::fs;

pub fn invoke() {
    fs::create_dir_all(".git/objects").unwrap();
    fs::create_dir_all(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();

    println!("Initialized git directory")
}
