use std::fmt;

#[derive(Clone, Copy)]
pub enum TimeRange {
    Live,
    Playback { start: u64, end: u64 },
}

impl From<TimeRange> for (u64, u64) {
    fn from(time_range: TimeRange) -> (u64, u64) {
        match time_range {
            TimeRange::Live => (0, 0),
            TimeRange::Playback { start, end } => (start, end),
        }
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimeRange::Live => write!(f, "live"),
            TimeRange::Playback { start, end } => write!(f, "from {} to {}", start, end),
        }
    }
}
