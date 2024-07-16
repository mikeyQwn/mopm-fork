use app::application::App;
use cli::config::{CliError, Config};
use log::logger::Logger;

mod app;
mod cli;
mod core;
mod log;
mod storage;

fn main() {
    let mut logger = Logger::default();
    let config = match Config::from_args() {
        Ok(v) => v,
        Err(err) => {
            logger.error(&err);
            match err {
                CliError::NoCommandSpecifiedError => {
                    logger.fatal(b"No command specified. exiting");
                }
                CliError::InvalidCommandError => {
                    logger.fatal(b"No such command");
                }
                CliError::MissingArgument(_, info) => {
                    logger.fatal(format!("Missing argument: {}", info).as_ref());
                }
            }
        }
    };
    let mut app = App::new(config, logger);
    app.run();
}
