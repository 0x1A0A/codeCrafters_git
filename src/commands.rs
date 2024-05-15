pub mod cat_file;
pub mod git_init;
pub mod hash_object;
pub mod ls_tree;

use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Cli,
}

#[derive(Debug, Subcommand)]
pub enum Cli {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty: bool,
        object: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
}
