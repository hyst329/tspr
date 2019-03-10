pub struct TimeInterval(u64, u64);
pub struct RangeInterval(u64, u64);

pub enum Interval {
    Time(TimeInterval),
    Range(RangeInterval),
}

pub struct Window(u64);
