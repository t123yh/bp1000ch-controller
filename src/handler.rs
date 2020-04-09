use std::process::{Command, Stdio};
use std::io::Write;
use std::fs::File;
use std::env;
use std::path::PathBuf;
use crate::status_record::system_time;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

fn write_tmp_file(content: &str) -> std::io::Result<PathBuf> {
    let mut dir = env::temp_dir();
    let rstr: String = thread_rng().sample_iter(&Alphanumeric).take(5).collect();
    dir.push(format!("upsmsg-{}-{}.txt", rstr, system_time()));
    File::create(dir.clone())?.write_all(content.as_bytes())?;
    return Ok(dir);
}

pub fn send_message(cmd: &Vec<String>, msg: &str) -> std::io::Result<()> {
    println!("Sending message!");

    let tmp_path = write_tmp_file(msg).unwrap().into_os_string().into_string().unwrap();
    let mut que: Vec<&String> = cmd.iter().skip(1).collect();
    que.push(&tmp_path);
    let mut child = Command::new(cmd[0].as_str())
        .args(que)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    child.stdin.as_mut()
        .unwrap_or_else(|| { panic!("Failed to grab stdin from child.") })
        .write_all(msg.as_bytes())?;

    child.wait()?;

    return Ok(());
}

pub fn run_cmd(cmd: &Vec<String>) -> std::io::Result<()> {
    Command::new(cmd[0].as_str())
        .args(cmd.iter().skip(1)).spawn()?;

    return Ok(());
}

