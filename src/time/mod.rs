use std::fmt;

#[derive(Debug, Copy, Clone, Default)]
pub struct TimingInfo {
    pub uptime_seconds: f32,
    pub current_frame_index: u32,
    pub frames_per_second: f32,
    pub unused_seconds: f32,
    pub unused_milliseconds: f32,
    pub milliseconds_slept: f32,
    pub seconds_since_last_update: f32,
}

impl fmt::Display for TimingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TimingInfo (uptime_seconds={}, current_frame_index={}, frames_per_second={})",
            self.uptime_seconds, self.current_frame_index, self.frames_per_second
        )
    }
}
