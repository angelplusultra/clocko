use chrono::{Local, NaiveDate, Utc};
use dialoguer::{FuzzySelect, Select};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
};

use crate::{
    structs::{Session, WorkDay},
    utils::clear_console,
};

#[derive(Serialize, Deserialize)]
pub struct App {
    pub data: HashMap<NaiveDate, WorkDay>,
    pub today: NaiveDate,
}

impl App {
    // WARNING: Not entirely sure this is working correctly, it might be overwriting days???? idk.
    // maybe its fine
    // TODO: Need to save file in the home directory of user e.g ~/.clocko/data.json instead of local
    //to wherever this program executes from.
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
                    date: today,
                    sessions: Vec::new(),
                },
            );
            app.write_data();
        }

        app
    }
    // INFO: Main app loop
    pub fn init(&mut self) {
        clear_console();
        loop {
            // check if an active session exists
            let has_active_session = self.data.get(&self.today).unwrap().active_session.is_some();

            // Filter selection options
            let opts = vec![
                (0, "Clock In"),
                (1, "Clock Out"),
                (2, "View Current Session"),
                (3, "Get Total Working Time"),
                (4, "Select Day"),
                (5, "Select Week"),
                (6, "Exit"),
            ]
            .into_iter()
            .filter(|&selection| match selection.0 {
                0 if has_active_session => false,
                1 if !has_active_session => false,
                2 if !has_active_session => false, // Only show if an active session exists
                3 if has_active_session => false,
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
            match answer {
                // INFO: Clock In
                0 => match self.create_session() {
                    Ok(sesh) => {
                        let local_time = sesh.start.with_timezone(&Local);
                        let formatted_time = local_time.format("%I:%M%p").to_string();

                        println!("Session started at {}", formatted_time);
                        self.write_data();
                    }
                    Err(msg) => {
                        println!("Error: {msg}");
                    }
                },
                // INFO: Clock Out
                1 => match self.end_active_session() {
                    Ok(sesh) => {
                        let duration = sesh.end.unwrap() - sesh.start;
                        println!(
                            "Hours: {}, Minutes: {}",
                            duration.num_hours(),
                            duration.num_minutes() % 60
                        );
                    }
                    Err(msg) => {
                        println!("Error: {msg}");
                    }
                },
                // INFO: Get Active Session
                2 => {
                    let sesh = self.get_active_session();

                    let duration = Utc::now() - sesh.start;

                    println!(
                        "Hours: {} Minutes: {}",
                        duration.num_hours(),
                        duration.num_minutes() % 60
                    );
                }
                // INFO: Get Total Time
                3 => {
                    let todays_sessions = &self.data.get(&self.today).unwrap().sessions;

                    let mut total_minutes = 0;
                    for sesh in todays_sessions.iter() {
                        let duration = sesh.end.unwrap() - sesh.start;

                        total_minutes += duration.num_minutes();
                    }

                    println!(
                        "Hours {} Minutes {}",
                        total_minutes / 60,
                        total_minutes % 60
                    );
                }
                // INFO: Select Day
                4 => {
                    let all_days = &self
                        .data
                        .iter()
                        .map(|(key, _)| key)
                        .collect::<Vec<&NaiveDate>>();

                    let selection = FuzzySelect::new()
                        .with_prompt("Select a Work Day")
                        .items(&all_days)
                        .interact()
                        .unwrap();
                },
                5 => {


                }
                // INFO: Exit
                6 => {
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
    }

    fn get_active_session(&self) -> &Session {
        let idx = self.data.get(&self.today).unwrap().active_session.unwrap();

        let active_session = &self.data.get(&self.today).unwrap().sessions[idx];

        active_session
    }
    pub fn create_session(&mut self) -> Result<&Session, &'static str> {
        let work_day = self.data.get_mut(&self.today).unwrap();

        if work_day.active_session.is_some() {
            println!("A session is already active");
            return Err("Cannot create a session while theres an active session");
        } else {
            let sesh = Session {
                start: Utc::now(),
                end: None,
            };

            work_day.sessions.push(sesh);
            work_day.active_session = Some(work_day.sessions.len() - 1);

            self.write_data();

            let sesh_ref = self.data.get(&self.today).unwrap().sessions.last().unwrap();

            return Ok(sesh_ref);
        }
    }

    pub fn end_active_session(&mut self) -> Result<&Session, &'static str> {
        let work_day = self.data.get_mut(&Utc::now().date_naive()).unwrap();

        if let Some(idx) = work_day.active_session {
            work_day.sessions[idx].end = Some(Utc::now());
            work_day.active_session = None;

            self.write_data();

            let sesh_ref = self.data.get(&self.today).unwrap().sessions.last().unwrap();

            return Ok(sesh_ref);
        }

        return Err("No active session");
    }

    fn read_data() -> App {
        let file = File::open("data.json").expect("Error reading data file");
        let reader = BufReader::new(file);

        let data: App =
            serde_json::from_reader(reader).expect("Error converting the buffer into json");

        data
    }

    fn write_data(&self) {
        if let Ok(file) = File::create("data.json") {
            let writer = BufWriter::new(file);

            let _ = serde_json::to_writer(writer, &self);
        }
    }
}
