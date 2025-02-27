use common::find_hidraw_device_name;
use common::led_config::LedConfig;
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match LedConfig::load() {
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

    let turn_on_off_result = if config.is_on {
        common::turn_backlight_on(device_name)
    } else {
        common::turn_backlight_off(device_name)
    };

    match turn_on_off_result {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Failed to turn the lights: {}", err);
            return ExitCode::from(1);
        }
    };

    ExitCode::SUCCESS
}
