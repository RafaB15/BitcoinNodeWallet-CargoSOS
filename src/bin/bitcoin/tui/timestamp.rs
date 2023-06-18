use chrono::Utc;

use std::convert::From;

const DAY: usize = 86_400;
const WEEK: usize = 604_800;
const MONTH: usize = 2_629_743;
const YEAR: usize = 31_556_926;

#[derive(Debug, Clone, Copy)]
pub enum Timestamp {
    Day,
    Week,
    Month,
    Year,
}

impl Timestamp {
    pub fn get_timestamps_from_now(&self) -> usize {
        let now = Utc::now();
        let now = now.timestamp() as usize;
        let delta_time: usize = (*self).into();
        now - delta_time
    }
}

impl From<Timestamp> for usize {
    fn from(value: Timestamp) -> Self {
        match value {
            Timestamp::Day => DAY,
            Timestamp::Week => WEEK,
            Timestamp::Month => MONTH,
            Timestamp::Year => YEAR,
        }
    }
}