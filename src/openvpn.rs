use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use std::process::Stdio;
use tempfile::NamedTempFile;

pub async fn start(config: &NamedTempFile, credential: &NamedTempFile) {
    let mut cmd = Command::new("sudo");
    cmd.args(&[
        "openvpn",
        "--config",
        config.path().to_str().unwrap(),
        "--auth-user-pass",
        credential.path().to_str().unwrap(),
    ]);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn command");

    // let mut stdin = child
    //     .stdin
    //     .take()
    //     .expect("child did not have a handle to stdin");

    let mut stdout_reader = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut stderr_reader = BufReader::new(child.stderr.take().unwrap()).lines();

    info!("OpenVPN PID: {}", child.id().unwrap().to_string());

    // tokio::spawn(async move {
    //     stdin
    //         .write("echo nya && echo owo && exit\n".as_bytes())
    //         .await
    //         .unwrap();
    // });
    tokio::spawn(async move {
        loop {
            let stdout_line = stdout_reader.next_line();
            let stderr_line = stderr_reader.next_line();
            tokio::select! {
                stdout = stdout_line => {
                    if let Some(line) = stdout.unwrap() {
                        info!("OpenVPN stdout: {}", line);
                    }
                }
                stderr = stderr_line => {
                    if let Some(line) = stderr.unwrap() {
                        error!("OpenVPN stderr: {}", line);
                    }
                }
            }
        }
    });
    tokio::spawn(async move {
        info!("Waiting for child process...");
        match child.wait().await {
            Ok(status) => {
                info!("Child process exited with status: {}", status);
            }
            Err(e) => {
                error!("Child process exited with error: {}", e);
            }
        }
    })
    .await
    .unwrap();
}
