use common::find_hidraw_device_name;
use common::led_config::LedConfig;
use evdev::EventSummary::{Key, Misc};
use evdev::{Device, KeyCode, MiscCode};
use nix::poll::{poll, PollFd, PollFlags};
use signal_hook::consts::SIGTERM;
use std::os::fd::{AsRawFd, BorrowedFd};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use nix::errno::Errno;

const FN_SPACE: i32 = 0xc0050;
const DEVICE: &str = "/dev/input/by-id/usb-ITE_Tech._Inc._ITE_Device_8910_-event-kbd";

fn main() -> ExitCode {
    let mut config = match LedConfig::load() {
        Ok(led_config) => led_config,
        Err(err) => {
            eprintln!("Failed to load LED config: {}", err);
            return ExitCode::from(1);
        }
    };

    let device_name_result = find_hidraw_device_name();
    let device_name = match device_name_result {
        Ok(dev) => dev,
        Err(err) => {
            eprintln!("Failed to find the keyboard device: {}", err);
            return ExitCode::from(1);
        }
    };

    let mut is_on = config.is_on;

    let stop_flag = Arc::new(AtomicBool::new(false));
    match signal_hook::flag::register(SIGTERM, Arc::clone(&stop_flag)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to register signal handler: {}", err);
            return ExitCode::from(1);
        }
    };

    let mut device = match Device::open(DEVICE) {
        Ok(device) => device,
        Err(err) => {
            eprintln!("Failed to open the keyboard input device: {}", err);
            return ExitCode::from(1);
        }
    };

    let raw_fd = device.as_raw_fd();
    let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
    let mut poll_fd = [PollFd::new(fd, PollFlags::POLLIN)];

    let mut is_pressed = false;

    while !stop_flag.load(Ordering::Relaxed) {
        match poll(&mut poll_fd, 100u16) {
            Ok(0) => {
                continue;
            }
            Ok(_) => {}
            Err(Errno::EINTR) => {
                println!("Interrupt detected");
                break;
            }
            Err(err) => {
                let error_description = err.desc();
                eprintln!("Failed to poll for device events: {}", error_description);
                return ExitCode::from(1);
            }
        }

        for event in device.fetch_events().unwrap() {
            let mut should_switch = false;
            match event.destructure() {
                Misc(_, MiscCode::MSC_SCAN, FN_SPACE) => {
                    if !is_pressed {
                        is_pressed = true;
                    }
                }
                Key(_, KeyCode::KEY_UNKNOWN, 1) if is_pressed => {
                    should_switch = true;
                }
                _ => {
                    is_pressed = false;
                }
            }

            if should_switch {
                let turn_on_off_result = if is_on {
                    is_on = false;
                    println!("Turning the LEDs off");
                    common::turn_backlight_off(&device_name)
                } else {
                    is_on = true;
                    println!("Turning the LEDs on");
                    common::turn_backlight_on(&device_name)
                };

                config.is_on = is_on;
                match config.save() {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Failed to save updated config: {}", err);
                        return ExitCode::from(1);
                    }
                }

                match turn_on_off_result {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Failed to switch LEDs: {}", err);
                        return ExitCode::from(1);
                    }
                }
            }
        }
    }

    println!("Exiting gracefully");
    ExitCode::SUCCESS
}
