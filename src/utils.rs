use std::{fs, path::PathBuf, process::Command};

pub fn clear_console() {
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

pub fn instantiate_data_dir() -> PathBuf {
    let mut data_path = dirs::data_dir().expect("Couldn't locate config directory");
    data_path.push("clocko");

    if !data_path.exists() {
        fs::create_dir_all(&data_path).expect("Couldnt create the config");
    }

    data_path.push("data.json");

    return data_path;
}
