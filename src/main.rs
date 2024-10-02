use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    process::Command,
};

use chrono::{DateTime, NaiveDate, Utc};
use dialoguer::Select;
use serde::{Deserialize, Serialize};

fn main() {
    let mut app = App::new();

    app.init();
}

#[derive(Serialize, Deserialize)]
struct App {
    data: HashMap<NaiveDate, WorkDay>,
    today: NaiveDate,
}

impl App {
    fn clear_console(&self) {
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", "cls"])
                .status()
                .expect("failed to clear console");
        } else {
            Command::new("clear")
                .status()
                .expect("failed to clear console");
        }
    }

    pub fn new() -> Self {
        let today = Utc::now().date_naive();
        let mut app;

        // Try to read data from data.json
        if let Ok(_) = File::open("data.json") {
            app = App::read_data();
        } else {
            app = App {
                today: today.clone(),
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
    // Main CLI Loop
    pub fn init(&mut self) {
        self.clear_console();
        loop {
            // check if an active session exists
            let has_active_session = self.data.get(&self.today).unwrap().active_session.is_some();
            if has_active_session {
                println!("You have an active session");
            }

            // Filter selection options
            let opts = vec![
                (0, "Clock In"),
                (1, "Clock Out"),
                (2, "View Current Session"),
                (3, "Get Total Working Time"),
                (4, "Exit"),
            ]
            .into_iter()
            .filter(|&selection| match selection.0 {
                0 if has_active_session => false,
                1 if !has_active_session => false,
                2 if !has_active_session => false, // Only show if an active session exists
                _ => true,
            })
            .collect::<Vec<(i32, &str)>>();


            let opts_formatted = opts.iter().map(|&(_, v)| v).collect::<Vec<&str>>();
            // Display select menu
            let selection = Select::new()
                .with_prompt("What do you choose?")
                .items(&opts_formatted)
                .interact()
                .unwrap();

            let (answer, _) = opts[selection];
            /* Selection Resolvers */

            // Clock in
            if answer == 0 {
                if has_active_session {
                    println!(
                        "You cannot clock in while there is an active session for the current day"
                    );
                    continue;
                }

                let sesh = self.create_session().unwrap();

                println!("Session started at {}", sesh.start);
                self.write_data();
            }

            // Clock out
            if answer == 1 {
                if !has_active_session {
                    println!("You cannot clock out when there is no active session.");
                    continue;
                }

                // End session and get clone of session
                let sesh = self.end_active_session().unwrap();

                // Display duration of time to user Hours: {} Minutes: {}
                println!("Hours: {}, Minutes: {}", sesh.start, sesh.end.unwrap());
            }

            // Get total working time for today
            if answer == 2 {}

            // View Current Session
            if answer == 3 {}
            if answer == 4 {
                break;
            }
        }
    }

    pub fn create_session(&mut self) -> Option<Session> {
        let work_day = self.data.get_mut(&Utc::now().date_naive()).unwrap();

        if work_day.active_session.is_some() {
            println!("A session is already active");
            return None;
        } else {
            let sesh = Session {
                start: Utc::now(),
                end: None,
            };

            work_day.sessions.push(sesh.clone());
            work_day.active_session = Some(work_day.sessions.len() - 1);

            self.write_data();

            return Some(sesh);
        }
    }

    pub fn end_active_session(&mut self) -> Option<Session> {
        let work_day = self.data.get_mut(&Utc::now().date_naive()).unwrap();

        if let Some(idx) = work_day.active_session {
            work_day.sessions[idx].end = Some(Utc::now());
            work_day.active_session = None;

            self.write_data();

            return Some(self.data.get(&self.today).unwrap().sessions[idx].clone());
        }

        return None;
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

            let _ = serde_json::to_writer(writer, &self);
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct WorkDay {
    sessions: Vec<Session>,
    active_session: Option<usize>,
    date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone)]
struct Session {
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
}
