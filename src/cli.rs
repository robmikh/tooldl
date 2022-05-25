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
}