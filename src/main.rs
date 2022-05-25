mod cli;
mod github;
mod zip_utils;

use std::{fs::File, io::Cursor, path::Path};

use clap::Parser;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
    Client,
};
use zip_utils::extract_to_directory;

use crate::{cli::Args, github::ReleasesLatest};

pub const SERVICE_NAME: &str = "tooldl";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let token = get_token(&args)?;

    let tools_folder = args.path();
    let tools_registry = {
        let mut tools_registry = tools_folder.clone();
        tools_registry.push("tools.txt");
        tools_registry
    };
    let tool_repos = get_tools(&tools_registry);

    let temp_folder = {
        let mut temp_folder = std::env::temp_dir();
        temp_folder.push("tooldl");
        temp_folder
    };
    //println!("{:?}", &temp_folder);
    if !temp_folder.exists() {
        std::fs::create_dir(&temp_folder)?;
    }

    for (owner, repo) in tool_repos {
        println!("Processing {owner}/{repo}...");
        let latest = get_latest_release_async(&token, &owner, &repo).await?;
        //println!("{:#?}", latest);
        let mut tool_path = {
            let mut tool_path = tools_folder.clone();
            tool_path.push(&repo);
            tool_path
        };
        if !tool_path.exists() {
            std::fs::create_dir(&tool_path)?;
        }

        tool_path.push("info.txt");
        // If info.txt exists, check to see if it has a matching version.
        // If it does, we don't need to download a new copy.
        if tool_path.exists() {
            let version = std::fs::read_to_string(&tool_path)?;
            if version.starts_with(&latest.tag_name) {
                println!(
                    "  Skipping, latest version ({}) already installed.",
                    &latest.tag_name
                );
                continue;
            }
        }
        std::fs::write(&tool_path, &latest.tag_name)?;

        println!("  Updating to {}", &latest.tag_name);
        for asset in latest.assets {
            let url = &asset.browser_download_url;
            let name = &asset.name;

            if !name.ends_with(".zip") {
                continue;
            }

            let arch = if name.contains("x64") || name.contains("x86_64") {
                "x64"
            } else if name.contains("ARM64") || name.contains("aarch64") {
                "ARM64"
            } else {
                continue;
            };

            tool_path.set_file_name(arch);
            if !tool_path.exists() {
                std::fs::create_dir(&tool_path)?;
            }

            // Download the release and unzip it
            let zip_file = download_file_async(url, &temp_folder, name).await?;
            extract_to_directory(zip_file, &tool_path)?;
        }
    }

    // Delete temp folder
    std::fs::remove_dir_all(temp_folder)?;

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
                    }
                    _ => {
                        Err(error)?;
                    }
                }
                panic!()
            }
        }
    };
    Ok(token)
}

async fn get_latest_release_async(
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<ReleasesLatest, Box<dyn std::error::Error>> {
    let request_url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    //println!("{}", request_url);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", &token))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_str(SERVICE_NAME)?);

    let response = Client::new()
        .get(&request_url)
        .headers(headers)
        .send()
        .await?;
    //println!("{:#?}", response);

    let latest = response.json::<ReleasesLatest>().await?;
    Ok(latest)
}

fn get_tools<P: AsRef<Path>>(path: P) -> Vec<(String, String)> {
    let path = path.as_ref();

    if !path.exists() {
        eprintln!("Tools registry not found! Make sure there is a 'tools.txt' file in the specified directory.");
        std::process::exit(1);
    }

    let mut tools = Vec::new();
    let string = std::fs::read_to_string(path).unwrap();
    let lines = string.lines();
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let (owner, repo) = line
            .split_once('/')
            .expect("Invalid format for repo entry in tools registry!");
        tools.push((owner.to_owned(), repo.to_owned()));
    }
    tools
}

async fn download_file_async<P: AsRef<Path>>(
    url: &str,
    folder: P,
    file_name: &str,
) -> Result<File, Box<dyn std::error::Error>> {
    let mut path = folder.as_ref().to_owned();
    path.push(file_name);

    {
        let response = reqwest::get(url).await?;
        let mut file = File::create(&path)?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
    }

    let file = File::open(path)?;
    Ok(file)
}
