mod commands;
mod objects;

use clap::Parser;
use commands::{Args, Cli};

fn main() {
    let args = Args::parse();

    match args.command {
        Cli::Init => commands::git_init::invoke(),
        Cli::CatFile { pretty, object } => invoke!(cat_file, &object, pretty),
        Cli::HashObject { write, file } => invoke!(hash_object, file, write),
        Cli::LsTree {
            name_only,
            tree_hash,
        } => invoke!(ls_tree, &tree_hash, name_only),
    }
}
