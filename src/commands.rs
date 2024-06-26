pub mod cat_file;
pub mod commit_tree;
pub mod git_init;
pub mod hash_object;
pub mod ls_tree;
pub mod write_tree;
pub mod clone;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    WriteTree {},
    CommitTree {
        tree_hash: String,

        #[clap(short = 'p')]
        parent: Option<String>,

        #[clap(short = 'm')]
        message: String,
    },
    Clone {
        source: String,
        dir: Option<PathBuf>,
    },
}

#[macro_export]
macro_rules! invoke {
    ($command: ident, $hash: expr, $($args: ident),*) => {
        {
            let options = commands::$command::Options { $( $args ),* };
            commands::$command::invoke($hash, options);
        }
    };
    ($command: ident) => {
        {
            commands::$command::invoke();
        }
    };
}
