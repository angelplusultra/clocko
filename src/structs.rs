use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct WorkDay {
    pub sessions: Vec<Session>,
    pub active_session: Option<usize>,
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}
