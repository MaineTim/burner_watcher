extern crate gpio_cdev;

mod events;

use chrono::prelude::*;
use gpio_cdev::*;
use std::env;
use std::thread;
use std::time::Duration;

struct Burner {
    chip: String, // The gpiochip device (e.g. /dev/gpiochip0)
    line: u32,    // The offset of the GPIO line for the provided chip
}

fn split_once(in_string: &str) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, ':');
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}

/// do_test takes a command string in the format "HIGH:1000-LOW:2000-HIGH:4000",
/// action:duration-action:duration... It simulates those events and calls
/// process_event on each.
fn do_test(commands: String) -> errors::Result<()> {
    let mut burner_status = events::BurnerStatus {
        start_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        end_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
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
        burner_status = events::process_event(burner_status, event_status);
        println!("{:?}", burner_status.firing);
        let duration = delay.parse::<u64>().unwrap();
        thread::sleep(Duration::from_millis(duration));
    }

    Ok(())
}

fn do_main(burner: Burner) -> errors::Result<()> {
    let mut burner_status = events::BurnerStatus {
        start_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        end_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        firing: false,
    };
    let mut chip = Chip::new(burner.chip)?;
    let line = chip.get_line(burner.line)?;
    for event in line.events(
        LineRequestFlags::INPUT,
        EventRequestFlags::BOTH_EDGES,
        "gpioevents",
    )? {
        let evt = event?;
        let evt_pin_state = match evt.event_type() {
            EventType::RisingEdge => events::LineState::High,
            EventType::FallingEdge => events::LineState::Low,
        };
        let event_status = events::EventStatus {
            timestamp: evt.timestamp(),
            pin_state: evt_pin_state,
        };
        burner_status = events::process_event(burner_status, event_status);
        println!("{:?}", burner_status.firing);
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

    match args.len() {
        1 => {
            match do_main(burner) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            };
        }
        _ => {
            let commands = &args[1];
            match do_test(commands.to_string()) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            };
        }
    }
}
