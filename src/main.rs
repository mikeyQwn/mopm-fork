use cli::config::{Command, Config};
use storage::storage::{Storage, StorageError};

mod cli;
mod core;
mod storage;

fn handle_clear() {
    match Storage::clear() {
        Ok(_) => {
            println!("The momp storage has been cleared. All data is lost");
        }
        Err(StorageError::RootDoesNotExistErorr) => {
            println!("The mopm storage is not initialized")
        }
        Err(err) => panic!("{}", err),
    }
}

fn handle_init() {
    match Storage::init() {
        Ok(_) => println!("The momp storage has been successfully initialized!"),
        Err(StorageError::RootAlreadyExistsErorr) => {
            println!("The mopm storage has already been initialized")
        }
        Err(err) => panic!("{}", err),
    }
}

fn main() {
    let config = Config::from_args().unwrap();
    match config.command {
        Command::Init => handle_init(),
        Command::Clear => handle_clear(),
    }
}
