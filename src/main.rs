extern crate gpio_cdev;

mod events;

use gpio_cdev::*;
use chrono::prelude::*;

struct Burner {
    /// The gpiochip device (e.g. /dev/gpiochip0)
    chip: String,
    /// The offset of the GPIO line for the provided chip
    line: u32,
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
        let mut evt_pin_state = events::LineState::Low;
        match evt.event_type() {
            EventType::RisingEdge => {
                evt_pin_state = events::LineState::High;
            }
            EventType::FallingEdge => {
                evt_pin_state = events::LineState::Low;
            }
        }
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
    match do_main(burner) {
        Ok(()) => {}
        Err(e) => {
            println!("Error: {:?}", e);
        }
    };
}
