mod onepassword;
mod read_models;
mod openvpn;
use onepassword::{onepassword_read, OnePassword};
use std::{collections::HashMap, env, io::Write, process::exit};

use dotenv::dotenv;
use log::{error};
use tempfile::NamedTempFile;
use tokio::process::Command;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env::set_var("RUST_LOG", "openvpn_1password");
    env_logger::init();
    if !check_environment().await {
        return;
    };
    let mut args = env::args();
    let origin_path = args.nth(1).unwrap_or_else(|| {
        println!("Usage: {} <path>", env::args().next().unwrap());
        exit(0);
    });

    let res = onepassword_read(&origin_path).await;
    match res {
        OnePassword::Result(v) => {
            let needle_fields = vec!["username", "password", "host", "port"];
            let empty_str = String::new();
            let mut fields: HashMap<String, String> = HashMap::new();
            if v.fields.len() == 0 {
                error!("No fields found");
                exit(2);
            }
            for field in v.fields {
                if needle_fields.contains(&field.label.as_str()) {
                    let value = field.value.unwrap_or(String::from(""));
                    fields.insert(field.label, value);
                }
            }
            let ovpn_file_name = if let Some(files) = v.files {
                let ovpn_file_name = files
                    .iter()
                    .filter(|file| file.name.ends_with(".ovpn"))
                    .next()
                    .unwrap()
                    .name
                    .clone();
                Some(ovpn_file_name)
            } else {
                None
            };
            if ovpn_file_name.is_none() {
                error!("No openvpn configuration file found");
                exit(2);
            }
            let ovpn_config_file = match onepassword_read(format!("{}/{}", origin_path, ovpn_file_name.unwrap()).as_str()).await {
                OnePassword::Raw(v) => Some(v),
                _ => None,
            };
            let (username, password, _host, _port) = (
                fields.get("username").unwrap_or(&empty_str),
                fields.get("password").unwrap_or(&empty_str),
                fields.get("host").unwrap_or(&empty_str),
                fields.get("port").unwrap_or(&empty_str),
            );
            let temp_credentials =
                create_temp_pipe_file(format!("{}\n{}", username, password).as_str());
            let temp_ovpn_config = create_temp_pipe_file(ovpn_config_file.unwrap().as_str());
            openvpn::start(&temp_ovpn_config, &temp_credentials).await;
        }
        OnePassword::Error(_e) => panic!(),
        _ => panic!(),
    }
}
fn create_temp_pipe_file(text: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(text.as_bytes()).unwrap();
    return file;
}
async fn check_environment() -> bool {
    let commands = vec!["op", "openvpn"];
    for command in commands {
        if !check_command_exist(command).await {
            error!("Command {} not found", command);
            return false;
        }
    }
    true
}
async fn check_command_exist(command: &str) -> bool {
    let executor = Command::new("which").arg(command).output().await;
    let executor = match executor {
        Ok(v) => v,
        Err(e) => {
            error!(
                "Check command exist failed with error code: {} , {}",
                e.raw_os_error().unwrap_or(0),
                e
            );
            return false;
        }
    };
    let (stdout, stderr) = (executor.stdout, executor.stderr);
    if stdout.len() == 0 {
        error!(
            "Check command exist failed {}",
            String::from_utf8(stderr).unwrap()
        );
        false
    } else {
        true
    }
}
