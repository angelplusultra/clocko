mod app;
mod structs;
use app::App;

fn main() {
    let mut app = App::new();

    app.init();
}
