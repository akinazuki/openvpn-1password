use log::{debug, error};
use reqwest::Url;
use tokio::process::Command;

use crate::read_models::Root;

#[derive(Debug)]
pub enum OnePassword {
    Result(Root),
    Error(String),
    Raw(String),
}
pub async fn onepassword_read(target: &str) -> OnePassword {
    let parsed_url = Url::parse(target).unwrap();
    let vault = parsed_url.host_str().unwrap();
    let items = parsed_url
        .path()
        .trim_start_matches('/')
        .split("/")
        .collect::<Vec<&str>>();
    let (item, field) = if items.len() == 2 {
        (items[0], Some(items[1]))
    } else {
        (items[0], None)
    };
    debug!("Vault: {}, Item: {}, Field: {}", vault, item, field.is_some());
    let executor = if field.is_some() {
      Command::new("op")
        .args(&["read", target])
        .output()
        .await
    } else {
      Command::new("op")
        .args(&["item", "get", item, "--vault", vault, "--format", "json"])
        .output()
        .await
    };
    let executor = match executor {
        Ok(v) => v,
        Err(e) => {
            error!(
                "Read 1Password failed with error code: {} , {}",
                e.raw_os_error().unwrap_or(0),
                e
            );
            panic!();
        }
    };
    let (stdout, stderr) = (executor.stdout, executor.stderr);
    if field.is_some() {
        return OnePassword::Raw(String::from_utf8(stdout).unwrap());
    }
    match serde_json::from_str(&String::from_utf8(stdout.clone()).unwrap()) {
        Ok(v) => OnePassword::Result(v),
        Err(e) => {
            error!("Read 1Password failed: {}", e);
            error!(
                " -> 1Password stdout: {}",
                String::from_utf8(stdout).unwrap()
            );
            error!(
                " -> 1Password stderr: {}",
                String::from_utf8(stderr).unwrap()
            );
            OnePassword::Error(e.to_string())
        }
    }
}
