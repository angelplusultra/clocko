use std::process::Command;

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
