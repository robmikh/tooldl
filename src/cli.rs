use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

/// A utility to download/update tools.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// GitHub username to use for requests
    #[clap(short, long)]
    pub user: String,

    /// GitHub token to use for requests
    #[clap(short, long)]
    pub token: Option<String>,

    /// Path to operate in
    #[clap(short, long)]
    pub path: Option<String>,
}

impl Args {
    pub fn path(&self) -> PathBuf {
        if let Some(path) = &self.path {
            PathBuf::from_str(path).unwrap()
        } else {
            std::env::current_dir().unwrap()
        }
    }
}
