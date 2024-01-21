#[derive(Debug, Copy, Clone, Default)]
pub struct TimingInfo {
    pub uptime_seconds: f32,
    pub frames_per_second: f32,
    pub unused_seconds: f32,
    pub unused_milliseconds: f32,
    pub milliseconds_slept: f32,
    pub seconds_since_last_update: f32,
}
