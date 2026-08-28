use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod build;
mod categories;
mod comments;
mod content;
mod date_time;
mod dev;
mod git;
mod init;
mod metadata;
mod output;
mod parser;
mod render;
mod search;
mod templates;
mod theme;
mod url_path;

const GENERATOR: &str = concat!("Inkcairn ", env!("CARGO_PKG_VERSION"));

#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new site
    Init {
        #[arg(value_name = "DIRECTORY", default_value = ".")]
        directory: PathBuf,
    },

    /// Generate the static site
    Build {
        /// Build with uncommitted, untracked, or non-Git content
        #[arg(long)]
        allow_dirty: bool,

        /// Include drafts
        #[arg(long)]
        include_drafts: bool,

        #[arg(value_name = "DIRECTORY", default_value = ".")]
        directory: PathBuf,
    },

    /// Preview and reload the site while writing
    Dev {
        /// Use a specific port instead of finding an available one
        #[arg(long)]
        port: Option<u16>,

        #[arg(value_name = "DIRECTORY", default_value = ".")]
        directory: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Init { directory } => {
            let root = init::run(&directory)?;
            println!("Initialized site in {}", root.display());
        }
        Command::Build {
            allow_dirty,
            include_drafts,
            directory,
        } => {
            let output = build::build(&directory, allow_dirty, include_drafts)?;
            println!("Generated site in {}", output.display());
        }
        Command::Dev { port, directory } => dev::run(&directory, port).await?,
    }

    Ok(())
}
