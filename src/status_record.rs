use serde::{Serialize, Deserialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum PowerStatus {
    Unknown,
    MainsRunning,
    BatteryRunning,
    Failure,
    CommunicationFailed,
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
#[serde(tag = "status")]
pub enum ShutdownStatus {
    Normal,
    ShutdownPending { time: u64 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Status {
    pub power: PowerStatus,
    pub low_battery_counter: usize,
    pub shutdown: ShutdownStatus,
}

fn read_status(path: &str) -> std::io::Result<Status> {
    let content = fs::read_to_string(path)?;
    let conf: Status = toml::from_str(content.as_str())?;
    return Ok(conf);
}

pub fn try_read_status(path: &str) -> Status {
    match read_status(path) {
        Ok(result) => result,
        Err(_) => Status {
            power: PowerStatus::Unknown,
            shutdown: ShutdownStatus::Normal,
            low_battery_counter: 0,
        }
    }
}

pub fn write_status(path: &str, content: &Status) -> std::io::Result<()> {
    let str = toml::to_string(content).unwrap(); // Shouldn't have an error here
    fs::write(path, str.as_str())
}

pub fn system_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn uptime() -> u64 {
    fs::read_to_string("/proc/uptime").unwrap().split_whitespace().nth(0).unwrap().parse::<f64>().unwrap() as u64
}

pub fn boot_time() -> u64 {
    system_time() - uptime()
}
