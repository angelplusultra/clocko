use std::{
    collections::HashMap,
    fs::File,
    io::{ BufReader, BufWriter},
};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

fn main() {
    let mut app = App::new();

    app.end_active_session();
}

#[derive(Serialize, Deserialize)]
struct App {
    data: HashMap<NaiveDate, WorkDay>,
}

impl App {
    pub fn new() -> Self {
        let today = Utc::now().date_naive();
        let mut app;

        // Try to read data from data.json
        if let Ok(_) = File::open("data.json") {
            app = App::read_data();
        } else {
            app = App {
                data: HashMap::new(),
            };
        }

        // Ensure that today's WorkDay exists in app.data
        if !app.data.contains_key(&today) {
            app.data.insert(
                today,
                WorkDay {
                    active_session: None,
                    date: today.clone(),
                    sessions: Vec::new(),
                },
            );
            app.write_data();
        }

        app
    }
    pub fn init(&self) {
        loop {
            print!("Enter command (start, end, status, quit): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "start" => self.create_session(),
                "end" => self.end_active_session(),
                "quit" => break,
                _ => println!("Unknown command"),
            }
        }
    }

    pub fn create_session(&mut self) {
        let work_day = self.data.get_mut(&Utc::now().date_naive()).unwrap();

        if work_day.active_session.is_some() {
            println!("A session is already active");
        } else {
            let sesh = Session {
                start: Utc::now(),
                end: None,
            };

            work_day.sessions.push(sesh);
            work_day.active_session = Some(work_day.sessions.len() - 1);

            self.write_data();
        }
    }

    pub fn end_active_session(&mut self) {
        let work_day = self.data.get_mut(&Utc::now().date_naive()).unwrap();

        if let Some(idx) = work_day.active_session {
            work_day.sessions[idx].end = Some(Utc::now());
            work_day.active_session = None;

            self.write_data();
        }
    }

    fn read_data() -> App {
        let file = File::open("data.json").expect("Error reading data file");
        let reader = BufReader::new(file);

        let data: App =
            serde_json::from_reader(reader).expect("Errro converting the buffer into json");

        data
    }

    fn write_data(&self) {
        if let Ok(file) = File::create("data.json") {
            let writer = BufWriter::new(file);

            serde_json::to_writer(writer, &self);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WorkDay {
    sessions: Vec<Session>,
    active_session: Option<usize>,
    date: NaiveDate,
}

#[derive(Serialize, Deserialize)]
struct Session {
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
}
