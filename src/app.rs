use chrono::{Datelike, Duration, Local, NaiveDate};
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
        let today = Local::now().date_naive();
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
                (2, "View Current Session's Total Working Time"),
                (3, "View Today's Total Working Time"),
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
                // INFO: View Active Session's Total Working Time
                2 => {
                    let sesh = self.get_active_session();

                    let duration = Local::now() - sesh.start;

                    println!(
                        "Hours: {} Minutes: {}",
                        duration.num_hours(),
                        duration.num_minutes() % 60
                    );
                }
                // INFO: View Today's Total Working Time
                3 => {
                    let total_minutes = self.get_total_minutes_from_day(&self.today);

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

                    let answer = all_days[selection];

                    let total_minutes = self.get_total_minutes_from_day(answer);

                    println!(
                        "Hours {} Minutes {}",
                        total_minutes / 60,
                        total_minutes % 60
                    );
                }
                // INFO: Select Week
                // WARNING: Not tested
                5 => {
                    // 1. Instantiate a hash map to contain a week string (2024-09-02 - 2024-10-05)
                    //    as key and all of its associated work sessions as a value Vec<&Session>
                    let mut week_hash = HashMap::<String, Vec<&Session>>::new();

                    // 2. filter out sessionless days
                    let sessionful_days = self
                        .data
                        .iter()
                        .filter(|&(_, day)| !day.sessions.is_empty())
                        .collect::<HashMap<&NaiveDate, &WorkDay>>();

                    // 3. Loop through every day (max 365)
                    for (date, work_day) in sessionful_days {

                        // 4. Compute the week range
                        let weekday = date.weekday();

                        let days_to_sunday = weekday.num_days_from_sunday();

                        let sunday = *date - Duration::days(days_to_sunday as i64);
                        let saturday = sunday + Duration::days(6);

                        let week_range = format!("{} - {}", sunday, saturday);

                        


                        // 5. Check hash if week range string exists, if so push all sessions of work
                        // day into vector, else create a new vector, push all sessions into it and
                        // insert into hash
                        if let Some(sessions) = week_hash.get_mut(&week_range) {
                            for sesh in work_day.sessions.iter() {
                                sessions.push(sesh);
                            }
                        } else {
                            let mut week_sessions = Vec::<&Session>::new();

                            for sesh in work_day.sessions.iter() {
                                week_sessions.push(sesh);
                            }
                            week_hash.insert(week_range, week_sessions);
                        }
                    }

                    // 6. Format week_range string options 
                    let opts = week_hash.iter().map(|(v, _)| v).collect::<Vec<&String>>();

                    // 7. Collect user input
                    let selection = FuzzySelect::new()
                        .with_prompt("Select a week")
                        .items(&opts)
                        .interact()
                        .unwrap();

                    let week_key = opts[selection];

                    // 8. Compute total time within week range and present
                    let mut total_minutes = 0;
                    for &sesh in week_hash[week_key].iter() {
                        let duration = sesh.end.unwrap_or(Local::now()) - sesh.start;

                        total_minutes += duration.num_minutes();
                    }

                    println!(
                        "Hours: {}, Minutes: {}",
                        total_minutes / 60,
                        total_minutes % 60
                    );
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

    fn get_total_minutes_from_day(&self, day: &NaiveDate) -> i64 {
        let sessions = &self.data.get(day).unwrap().sessions;
        let mut total_minutes = 0;
        for sesh in sessions.iter() {
            let duration = sesh.end.unwrap_or(Local::now()) - sesh.start;

            total_minutes += duration.num_minutes();
        }

        return total_minutes;
    }
    pub fn create_session(&mut self) -> Result<&Session, &'static str> {
        let work_day = self.data.get_mut(&self.today).unwrap();

        if work_day.active_session.is_some() {
            println!("A session is already active");
            return Err("Cannot create a session while theres an active session");
        } else {
            let sesh = Session {
                start: Local::now(),
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
        let work_day = self.data.get_mut(&self.today).unwrap();

        if let Some(idx) = work_day.active_session {
            work_day.sessions[idx].end = Some(Local::now());
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
