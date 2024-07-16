use app::application::App;
use cli::config::Config;
use log::logger::Logger;

mod app;
mod cli;
mod core;
mod log;
mod storage;

fn main() {
    let config = Config::from_args().unwrap();
    let logger = Logger::default();
    let mut app = App::new(config, logger);
    app.run();
}
