use chrono::prelude::*;

#[derive(Debug)]
pub enum LineState {
    Low,
    High,
}

pub struct BurnerStatus {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub firing: bool,
}

pub struct EventStatus {
    pub timestamp: u64,
    pub pin_state: LineState,
}

pub fn process_event(burner_status: BurnerStatus, event_status: EventStatus) -> BurnerStatus {
    let event_time: DateTime<Utc> = Utc::now();
    let firing_state = match event_status.pin_state {
        LineState::High => true,
        LineState::Low => false,
    };
    println!("{:?}", event_status.pin_state);
    let new_burner_status = BurnerStatus {
        start_time: event_time,
        end_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        firing: firing_state,
    };
    return new_burner_status;
}
