use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkDay {
    pub sessions: Vec<Session>,
    pub active_session: Option<usize>,
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub start: DateTime<Local>,
    pub end: Option<DateTime<Local>>,
}
