use super::error_tui::ErrorTUI;

use chrono::Utc;

use std::{
    convert::{From, TryFrom},
    fmt::Display,
};

const DAY: usize = 86_400;
const WEEK: usize = 604_800;
const MONTH: usize = 2_629_743;
const YEAR: usize = 31_556_926;

const DAY_OPTION: char = '1';
const WEEK_OPTION: char = '2';
const MONTH_OPTION: char = '3';
const YEAR_OPTION: char = '4';

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

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timestamp::Day => write!(f, "Day"),
            Timestamp::Week => write!(f, "Week"),
            Timestamp::Month => write!(f, "Month"),
            Timestamp::Year => write!(f, "Year"),
        }
    }
}

impl From<Timestamp> for char {
    fn from(value: Timestamp) -> Self {
        match value {
            Timestamp::Day => DAY_OPTION,
            Timestamp::Week => WEEK_OPTION,
            Timestamp::Month => MONTH_OPTION,
            Timestamp::Year => YEAR_OPTION,
        }
    }
}

impl TryFrom<&str> for Timestamp {
    type Error = ErrorTUI;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value: char = match value.chars().next() {
            Some(value) => value,
            _ => return Err(ErrorTUI::InvalidMenuOption),
        };

        match value {
            DAY_OPTION => Ok(Timestamp::Day),
            WEEK_OPTION => Ok(Timestamp::Week),
            MONTH_OPTION => Ok(Timestamp::Month),
            YEAR_OPTION => Ok(Timestamp::Year),
            _ => Err(ErrorTUI::InvalidMenuOption),
        }
    }
}
