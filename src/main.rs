mod commands;
mod objects;

use clap::Parser;
use commands::{Args, Cli};

fn main() {
    let args = Args::parse();

    match args.command {
        Cli::Init => commands::git_init::invoke(),
        Cli::CatFile { pretty, object } => {
            let options = commands::cat_file::Options { pretty };
            commands::cat_file::invoke(&object, options);
        }
        Cli::HashObject { write, file } => {
            let options = commands::hash_object::Options { write };
            commands::hash_object::invoke(file, options);
        }
        Cli::LsTree {
            name_only,
            tree_hash,
        } => {
            let options = commands::ls_tree::Options { name_only };
            commands::ls_tree::invoke(&tree_hash, options);
        }
    }
}
