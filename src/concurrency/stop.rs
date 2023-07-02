use super::work::Work;

use std::convert::From;

#[derive(Debug)]
// stop? stop what?
pub enum Stop {
    Stop,
}

impl From<Stop> for Work<()> {
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::Stop => Work::Stop,
        }
    }
}
