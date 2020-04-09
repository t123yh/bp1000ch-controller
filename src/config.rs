use serde::Deserialize;
use std::vec::Vec;
use std::{fs, env};

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    pub serial: String,
    pub status_file: String,
    pub battery_low_threshold: f32,
    pub battery_low_debounce: usize,
    pub battery_critical_threshold: f32,
    pub shutdown_delay_minutes: f32,
    pub shutdown_recovery_minutes: f32,

    pub print_log_command: Vec<String>,
    pub graceful_shutdown_command: Vec<String>,
    pub force_reboot_command: Vec<String>,
}

pub fn get_config() -> Config {
    let args: Vec<String> = env::args().collect();
    let conf_path = &args[1];
    let conf = fs::read_to_string(conf_path).unwrap();
    toml::from_str(&conf).unwrap()
}