use common::find_hidraw_device_name;
use evdev::EventSummary::{Key, Misc};
use evdev::{Device, KeyCode, MiscCode};
use std::process::ExitCode;
use common::led_config::LedConfig;

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

    let mut device = match Device::open(DEVICE) {
        Ok(device) => device,
        Err(err) => {
            eprintln!("Failed to open the keyboard input device: {}", err);
            return ExitCode::from(1);
        }
    };

    let mut is_pressed = false;

    loop {
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
}
