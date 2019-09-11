use chrono::prelude::*;
use rusqlite::{params, Connection, NO_PARAMS};

#[derive(Debug)]
pub enum LineState {
    Low,
    High,
}

#[derive(Debug)]
pub struct BurnerStatus {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub firing: bool,
}

pub struct EventStatus {
    pub timestamp: u64,
    pub pin_state: LineState,
}

fn save_to_dbase(burner_status: &BurnerStatus, dbase_filename: &str) {
    let datetime = String::from(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
    let conn = Connection::open(dbase_filename).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS runtimes(
        StartTime TEXT,
        EndTime TEXT,
        InsertionTime TEXT
    )",
        NO_PARAMS,
    )
    .unwrap();
    conn.execute(
        "INSERT INTO runtimes (
        StartTime,
        EndTime,
        InsertionTime
	) values(?1, ?2, ?3)",
        params![
            burner_status.start_time.to_rfc3339_opts(SecondsFormat::Secs, true),
            burner_status.end_time.to_rfc3339_opts(SecondsFormat::Secs, true),
            datetime
        ],
    )
    .unwrap();
    conn.close().unwrap();
    log::info!("Burn logged.")
}

/// process_event takes the current burner status, and a new event. If the event is HIGH (firing),
/// save the state (start_time) return. If the event is LOW (off), check to see if the time since
/// last HIGH was more than 5 seconds. If it is, call save_to_dbase.
pub fn process_event(burner_status: BurnerStatus, event_status: &EventStatus, dbase_filename: &str,) -> BurnerStatus {
    let event_time: DateTime<Utc> = Utc::now();
    let mut new_burner_status = BurnerStatus {
        start_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        end_time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        firing: false,
    };
    let firing_state = match event_status.pin_state {
        LineState::High => true,
        LineState::Low => false,
    };
    if firing_state {
        new_burner_status.start_time = event_time;
        new_burner_status.end_time = event_time;
        new_burner_status.firing = true;
    } else {
        new_burner_status.start_time = burner_status.start_time;
        new_burner_status.end_time = event_time;
        if new_burner_status
            .end_time
            .signed_duration_since(new_burner_status.start_time)
            .num_seconds()
            > 5
        {
            save_to_dbase(&new_burner_status, &dbase_filename);
        };
    }
    log::info!("{:?}", new_burner_status);
    return new_burner_status;
}
