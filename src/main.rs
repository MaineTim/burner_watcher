extern crate gpio_cdev;

mod events;

use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use gpio_cdev::*;
use toml::Value;

struct Burner {
    chip: String, // The gpiochip device (e.g. /dev/gpiochip0)
    line: u32,    // The offset of the GPIO line for the provided chip
}

/// split_once splits a string on ":" one time.
fn split_once(in_string: &str) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, ':');
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}

fn get_config(filepath: &str) -> Value {
    let config_string = fs::read_to_string(filepath).expect("Unable to read config file");
    let config = config_string.parse::<Value>().unwrap();
    return config;
}

/// do_test takes a command string in the format "HIGH:1000-LOW:2000-HIGH:4000",
/// action:duration-action:duration... It simulates those events and calls
/// process_event on each.
fn do_test(commands: String, dbase_filename: &str) -> errors::Result<()> {
    let mut burner_status = events::BurnerStatus {
        start_time: Utc::now(),
        end_time: Utc::now(),
        firing: false,
    };
    for command in commands.split("-") {
        let (action, delay) = split_once(command);
        let l_timestamp = Utc::now();
        let evt_pin_state = match action {
            "HIGH" => events::LineState::High,
            _ => events::LineState::Low,
        };
        let event_status = events::EventStatus {
            timestamp: l_timestamp.timestamp_nanos() as u64,
            pin_state: evt_pin_state,
        };
        burner_status = events::process_event(burner_status, &event_status, &dbase_filename);
        let duration = delay.parse::<u64>().unwrap();
        thread::sleep(Duration::from_millis(duration));
    }

    Ok(())
}

fn do_main(burner: Burner, dbase_filename: &str) -> errors::Result<()> {
    let mut burner_status = events::BurnerStatus {
        start_time: Utc::now(),
        end_time: Utc::now(),
        firing: false,
    };
    let mut chip = Chip::new(burner.chip)?;
    let line = chip.get_line(burner.line)?;
    for event in line.events(LineRequestFlags::INPUT, EventRequestFlags::BOTH_EDGES, "gpioevents")? {
        let evt = event?;
        let evt_pin_state = match evt.event_type() {
            EventType::RisingEdge => events::LineState::High,
            EventType::FallingEdge => events::LineState::Low,
        };
        let event_status = events::EventStatus {
            timestamp: evt.timestamp(),
            pin_state: evt_pin_state,
        };
        burner_status = events::process_event(burner_status, &event_status, &dbase_filename);
    }

    Ok(())
}

fn main() {
    let burner = Burner {
        chip: String::from("/dev/gpiochip0"),
        line: 23,
    };

    // If we have no CL arguments, then do_main(),
    // otherwise do_test() with the first argument.

    let args: Vec<String> = env::args().collect();
    env_logger::init();
    let config = get_config("burner_watcher.toml");
    let dbase_filename = format!("{}{}", config["DBs"]["dbasepath"].as_str().unwrap(),
        config["DBs"]["burnerlogfile"].as_str().unwrap());

    match args.len() {
        1 => {
            match do_main(burner, &dbase_filename) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            };
        }
        _ => {
            let commands = &args[1];
            match do_test(commands.to_string(), &dbase_filename) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            };
        }
    }
}
