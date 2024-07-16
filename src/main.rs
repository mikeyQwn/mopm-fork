use core::{encoder::Encoder, manager::PasswordManager};
use std::io::{BufRead, Read, Write};

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
    let mut key: String = String::new();
    std::io::stdin().lock().read_line(&mut key).unwrap();

    let mut pm = PasswordManager::init(key.trim());

    match Storage::init(&mut pm) {
        Ok(_) => println!("The momp storage has been successfully initialized!"),
        Err(StorageError::RootAlreadyExistsErorr) => {
            println!("The mopm storage has already been initialized")
        }
        Err(err) => panic!("{}", err),
    }
}

fn handle_store(key: String, value: String) {
    let mut pm_key: String = String::new();
    std::io::stdin().lock().read_line(&mut pm_key).unwrap();

    let mut pm_reader = Storage::get_data_reader().unwrap();
    let mut pm = Encoder::decode(pm_key.trim().as_ref(), &mut pm_reader).unwrap();

    pm.store_password(key, value.as_ref()).unwrap();
    let mut writer = Storage::get_data_writer().unwrap();
    Encoder::encode(&mut writer, &mut pm);
}

fn handle_get(key: String) {
    let mut pm_key: String = String::new();
    std::io::stdin().lock().read_line(&mut pm_key).unwrap();

    let mut pm_reader = Storage::get_data_reader().unwrap();
    let mut pm = Encoder::decode(pm_key.trim().as_ref(), &mut pm_reader).unwrap();
    println!("{}", pm.get_password(&key).unwrap())
}

fn main() {
    let config = Config::from_args().unwrap();
    match config.command {
        Command::Init => handle_init(),
        Command::Clear => handle_clear(),
        Command::Store(key, value) => handle_store(key, value),
        Command::Get(key) => handle_get(key),
    }
}
