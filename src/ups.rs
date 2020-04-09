use serialport::{SerialPortSettings, DataBits, FlowControl, Parity, StopBits};
use std::io::{BufReader, BufRead, Error, ErrorKind};
use std::time::Duration;
use std::str;

fn ups_send(port: &str, cmd: &str, need_response: bool) -> std::io::Result<Option<String>> {
    let terminator = '\r' as u8;

    let mut serial = serialport::open_with_settings(port, &SerialPortSettings {
        baud_rate: 2400,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(500),
    })?;
    let serial_read = serial.try_clone()?;

    let mut reader = BufReader::new(serial_read);
    let mut buf: Vec<u8> = Vec::new();

    serial.write(&[terminator])?;
    reader.read_until(terminator, &mut buf)?;
    assert!(buf.last() == Some(&terminator), "UPS did not respond correctly");

    serial.write(cmd.as_bytes())?;
    serial.write(&[terminator])?;
    if need_response {
        buf.clear();
        reader.read_until(terminator, &mut buf)?;
        let result = String::from_utf8_lossy(&buf).trim_matches('\r').to_string();
        return Ok(Some(result));
    } else {
        return Ok(None);
    }
}

pub fn ups_query(port: &str, cmd: &str) -> std::io::Result<String> {
    Ok(ups_send(port, cmd, true)?.unwrap())
}

pub fn ups_cmd(port: &str, cmd: &str) -> std::io::Result<()> {
    ups_send(port, cmd, false)?;
    Ok(())
}

#[derive(Debug)]
pub struct UpsStatusInfo {
    pub input_voltage: f32,
    pub output_voltage: f32,
    pub load_percentage: u8,
    pub battery_voltage: f32,
    pub beeper_active: bool,
    pub mains_fail: bool,
    pub battery_low: bool,
    pub buck_boost_active: bool,
    pub fault: bool,
    pub line_interactive: bool,
    pub self_test_running: bool,
    pub shutdown_active: bool,
}

pub fn ups_query_status(port: &str) -> std::io::Result<UpsStatusInfo> {
    let result_str = ups_query(port, "QS")?;
    if !result_str.starts_with("(") {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid UPS response to QS."));
    }
    let fields: Vec<&str> = result_str.trim_start_matches("(").split(" ").collect();
    let f = |idx: usize| -> f32 {
        fields[idx].parse().unwrap()
    };
    let u = |idx: usize| -> u8 {
        fields[idx].parse().unwrap()
    };
    let bits: Vec<char> = fields[7].chars().collect();
    let b = |idx: usize| -> bool {
        bits[7 - idx] == '1'
    };

    return Ok(UpsStatusInfo {
        input_voltage: f(0),
        output_voltage: f(2),
        load_percentage: u(3),
        battery_voltage: f(5),
        mains_fail: b(7),
        battery_low: b(6),
        buck_boost_active: b(5),
        fault: b(4),
        line_interactive: b(3),
        self_test_running: b(2),
        shutdown_active: b(1),
        beeper_active: b(0),
    });
}

pub fn ups_shutdown(port: &str, delay_minutes: f32, recover_minutes: f32) -> std::io::Result<()> {
    let delay_time = (delay_minutes * 10.0).round() as i32;
    let delay_str = match delay_time {
        0 => "00".to_owned(),
        1..=9 => format!(".{:1}", delay_time),
        10..=99 => format!("{:02}", delay_time / 10),
        _ => "99".to_owned()
    };

    let recover_str = format!("{:04}", recover_minutes);
    let cmd_str = format!("S{}R{}", delay_str, recover_str);
    ups_cmd(port, &cmd_str)
}

pub fn ups_cancel_shutdown(port: &str) -> std::io::Result<()> {
    ups_cmd(port, "C")
}
