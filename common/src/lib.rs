mod error;
pub mod led_config;

use crate::error::Error;
use crate::error::Error::{HidrawDeviceNotFound, Problem};
use nix::libc::{close, open, O_RDWR};
use nix::{ioctl_read_buf, ioctl_write_buf};
use std::ffi::{CString};
use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::{fs, io};

ioctl_read_buf!(hid_get_raw_name, b'H', 4, u8);
ioctl_write_buf!(hid_send_feature_report, b'H', 6, u8);

pub fn find_hidraw_device_name() -> Result<String, Error> {
    for dir in fs::read_dir("/sys/class/hidraw")? {
        let dir_entry = dir?;
        let uevent_file = dir_entry.path().join("device/uevent");
        let file = File::open(uevent_file)?;
        let lines = io::BufReader::new(file).lines();
        for line_result in lines {
            let line = line_result?;
            if line == "HID_NAME=ITE33D1:00 048D:837A" {
                // return this device immediately
                let device_name = dir_entry.file_name();
                return Ok(device_name.into_string().unwrap());
            }
        }
    }

    Err(HidrawDeviceNotFound)
}

fn turn_backlight_on_or_off<P: AsRef<Path>>(device_name: P, on: bool) -> Result<(), Error> {
    unsafe {
        let dev_name_path = PathBuf::from("/dev").join(device_name);
        let c_str = CString::new(dev_name_path.to_str().unwrap())?;
        let fd = open(c_str.as_ptr(), O_RDWR);

        if fd == -1 {
            let error_message = format!("Failed to open {}", dev_name_path.as_path().display());
            return Err(Problem(error_message));
        }

        let mut raw_name_buf: [u8; 256] = [0; 256];

        hid_get_raw_name(fd, &mut raw_name_buf).expect("Failed to read raw name");

        // First byte - 204
        // Second byte - 0
        // Third byte - block style
        // Fourth byte - block color [0-19]
        // Fifth byte - brightness [0-5]
        // Sixth byte - block number [0-3]

        const STYLE_ALWAYS_ON: u8 = 3;
        const COLOR_RED: u8 = 0;
        const MIN_BRIGHTNESS: u8 = 0;
        const MAX_BRIGHTNESS: u8 = 5;

        let brightness = if on { MAX_BRIGHTNESS } else { MIN_BRIGHTNESS };

        for block in 0..4 {
            let data = [204, 0, STYLE_ALWAYS_ON, COLOR_RED, brightness, block];
            hid_send_feature_report(fd, &data)?;
        }

        let data = [204, 9];
        hid_send_feature_report(fd, &data)?;
        close(fd);
    }

    Ok(())
}

pub fn turn_backlight_on<P: AsRef<Path>>(device_name: P) -> Result<(), Error> {
    turn_backlight_on_or_off(device_name, true)
}

pub fn turn_backlight_off<P: AsRef<Path>>(device_name: P) -> Result<(), Error> {
    turn_backlight_on_or_off(device_name, false)
}
