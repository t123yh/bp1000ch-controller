mod ups;
mod config;
mod status_record;
mod handler;

use config::get_config;
use status_record::{try_read_status, ShutdownStatus, boot_time};
use handler::run_cmd;

fn main() {
    let conf = get_config();
    let status = try_read_status(&conf.status_file);
    let mut force_reboot = false;
    match status.shutdown {
        ShutdownStatus::ShutdownPending { time: t } => {
            if t > boot_time() {
                println!("Sending cmd!");
                let result = ups::ups_shutdown(&conf.serial, conf.shutdown_delay_minutes, conf.shutdown_recovery_minutes);
                if !result.is_ok() {
                    force_reboot = true;
                }
            }
        }
        _ => ()
    }

    if force_reboot {
        run_cmd(&conf.force_reboot_command);
    }
}