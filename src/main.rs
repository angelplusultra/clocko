mod app;
mod structs;
mod utils;
use app::App;

fn main() {
    let mut app = App::new();

    app.init();
}
