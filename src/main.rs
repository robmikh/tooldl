mod cli;
mod github;

use clap::Parser;
use reqwest::{Client, header::{HeaderMap, AUTHORIZATION, HeaderValue, USER_AGENT}};

use crate::{cli::Args, github::ReleasesLatest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let service = "tooldl";
    let username = args.user;
    let keyring_entry = keyring::Entry::new(&service, &username);
    let token = if let Some(token) = args.token {
        keyring_entry.set_password(&token)?;
        token
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

    let request_url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest",
                              owner = "robmikh",
                              repo = "Win32CaptureSample");
    println!("{}", request_url);

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("token {}", &token))?);
    headers.insert(USER_AGENT, HeaderValue::from_str(service)?);

    let response = Client::new()
        .get(&request_url)
        .headers(headers)
        .send().await?;
    println!("{:#?}", response);
    //let text = response.text().await?;
    //println!("{:#?}", text);
    let latest = response.json::<ReleasesLatest>().await?;
    println!("{:#?}", latest);
    Ok(())
}
