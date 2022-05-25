mod cli;
mod github;

use clap::Parser;
use reqwest::{Client, header::{HeaderMap, AUTHORIZATION, HeaderValue, USER_AGENT}};

use crate::{cli::Args, github::ReleasesLatest};

pub const SERVICE_NAME: &str = "tooldl";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let token = get_token(&args)?;

    let latest = get_latest_release_async(&token, "robmikh", "Win32CaptureSample").await?;
    println!("{:#?}", latest);

    Ok(())
}

fn get_token(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    let keyring_entry = keyring::Entry::new(SERVICE_NAME, &args.user);
    let token = if let Some(token) = &args.token {
        keyring_entry.set_password(&token)?;
        token.clone()
    } else {
        let result = keyring_entry.get_password();
        match result {
            Ok(token) => token,
            Err(error) => {
                match error {
                    keyring::Error::NoEntry => {
                        eprintln!("No token found! Please supply one with '--token' so that it can be saved for future use.");
                        std::process::exit(1);
                    },
                    _ => {
                        Err(error)?;
                    }
                }
                panic!()
            },
        }
    };
    Ok(token)
}

async fn get_latest_release_async(token: &str, owner: &str, repo: &str) -> Result<ReleasesLatest, Box<dyn std::error::Error>> {
    let request_url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    //println!("{}", request_url);

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("token {}", &token))?);
    headers.insert(USER_AGENT, HeaderValue::from_str(SERVICE_NAME)?);

    let response = Client::new()
        .get(&request_url)
        .headers(headers)
        .send().await?;
    //println!("{:#?}", response);

    let latest = response.json::<ReleasesLatest>().await?;  
    Ok(latest)
}
