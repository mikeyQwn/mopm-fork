use std::{
    fs::create_dir,
    io::{self, Read, Write},
    path::PathBuf,
};

use thiserror::Error;

use crate::core::{
    encoder::Encoder, encryptor::Encryprtor, identifiers::Identifiable, manager::PasswordManager,
};

pub struct Storage {}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("the root directory already exists")]
    RootAlreadyExistsErorr,
    #[error("the root directory does not exist")]
    RootDoesNotExistErorr,
    #[error("cannot extract home directory")]
    HomedirExtractionError,
    #[error("io error")]
    IoError(io::Error),
}

impl From<io::Error> for StorageError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl Storage {
    pub fn init<T>(pm: &mut PasswordManager<T>) -> Result<(), StorageError>
    where
        T: Encryprtor + Identifiable,
    {
        let mut root = Self::root()?;

        if root.exists() {
            return Err(StorageError::RootAlreadyExistsErorr);
        }

        create_dir(&root)?;

        root.push("data");
        let mut password_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(root)
            .map_err(StorageError::from)?;

        Encoder::encode(&mut password_file, pm);
        Ok(())
    }

    pub fn get_data_reader() -> Result<impl Read, StorageError> {
        let mut root = Self::root()?;
        root.push("data");

        let password_file = std::fs::OpenOptions::new()
            .read(true)
            .open(root)
            .map_err(StorageError::from)?;
        Ok(password_file)
    }

    pub fn get_data_writer() -> Result<impl Write, StorageError> {
        let mut root = Self::root()?;
        root.push("data");
        let password_file = std::fs::OpenOptions::new()
            .write(true)
            .open(root)
            .map_err(StorageError::from)?;
        Ok(password_file)
    }

    pub fn clear() -> Result<(), StorageError> {
        let root = Self::root()?;
        if !root.exists() {
            return Err(StorageError::RootDoesNotExistErorr);
        }

        std::fs::remove_dir_all(root).map_err(StorageError::from)
    }

    fn root() -> Result<PathBuf, StorageError> {
        let mut homedir = Self::homedir()?;
        homedir.push(".mopm");

        Ok(homedir)
    }

    #[cfg(unix)]
    fn homedir() -> Result<PathBuf, StorageError> {
        match std::env::var_os("HOME") {
            Some(user) => Ok(PathBuf::from(user)),
            None => nix::unistd::User::from_uid(nix::unistd::Uid::current())
                .or(Err(StorageError::HomedirExtractionError))?
                .map(|u| u.dir)
                .ok_or(StorageError::HomedirExtractionError),
        }
    }
}
