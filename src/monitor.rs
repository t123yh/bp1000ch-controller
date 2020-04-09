pub mod ups;
pub mod config;
pub mod status_record;
pub mod handler;

use status_record::{ShutdownStatus, PowerStatus, write_status, try_read_status, system_time, boot_time};
use handler::{send_message, run_cmd};
use std::error::Error;
use crate::config::get_config;

fn format_ups_status(ups_data: &ups::UpsStatusInfo) -> String {
    return format!("输入电压 {}V，输出电压 {}V，电池电压 {}V，负载 {}%", ups_data.input_voltage, ups_data.output_voltage, ups_data.battery_voltage, ups_data.load_percentage)
}

fn main() {
    let conf = get_config();

    let mut graceful_shutdown = false;

    let mut status = try_read_status(&conf.status_file);
    let old_counter = status.low_battery_counter;
    status.low_battery_counter = 0;

    let is_shutdown_pending = match status.shutdown {
        ShutdownStatus::ShutdownPending {time: t} => {
            if t < boot_time() { // The shutdown script may have failed to execute?
                status.shutdown = ShutdownStatus::Normal;
                send_message(&conf.print_log_command, "系统已从掉电中恢复。").unwrap();
                false
            } else {
                true
            }
        },
        _ => false
    };

    if !is_shutdown_pending {
        match ups::ups_query_status(&conf.serial) {
            Err(err) => {
                if status.power != PowerStatus::CommunicationFailed {
                    send_message(&conf.print_log_command, &("UPS 通讯失败 ".to_owned() + err.description())).unwrap();
                    status.power = PowerStatus::CommunicationFailed;
                }
            },
            Ok(ups_data) => {
                println!("{:?}", ups_data);
                if status.power == PowerStatus::CommunicationFailed {
                    send_message(&conf.print_log_command, "UPS 通讯恢复。").unwrap();
                    status.power = PowerStatus::Unknown;
                }

                if ups_data.mains_fail {
                    if status.power != PowerStatus::BatteryRunning {
                        send_message(&conf.print_log_command, &format!("市电中断，UPS 现在由电池供电。{}。", format_ups_status(&ups_data))).unwrap();
                    }

                    status.power = PowerStatus::BatteryRunning;

                    if ups_data.battery_voltage < conf.battery_low_threshold || ups_data.battery_low {
                        status.low_battery_counter = old_counter + 1;

                        if status.low_battery_counter > conf.battery_low_debounce
                            || ups_data.battery_voltage < conf.battery_critical_threshold
                            || ups_data.battery_low {
                            send_message(&conf.print_log_command,
                                         &format!("UPS 电池电量低，执行关机命令。{}。", format_ups_status(&ups_data))).unwrap();
                            status.shutdown = ShutdownStatus::ShutdownPending { time: system_time() };
                            graceful_shutdown = true;
                        }
                    }
                } else {
                    if status.power != PowerStatus::MainsRunning {
                        send_message(&conf.print_log_command,
                                     &format!("UPS 现在由市电供电。{}。", format_ups_status(&ups_data))).unwrap();
                    }
                    status.power = PowerStatus::MainsRunning;
                }
            }
        }
    }

    write_status(&conf.status_file, &status).unwrap();

    if graceful_shutdown {
        run_cmd(&conf.graceful_shutdown_command).unwrap();
    }
}
