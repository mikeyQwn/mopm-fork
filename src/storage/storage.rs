use std::{fs::create_dir, io, path::PathBuf};

use thiserror::Error;

pub struct Storage {}

#[derive(Error, Debug)]
pub enum StorageError {
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
    pub fn init() -> Result<(), StorageError> {
        let root = Self::root()?;

        if root.exists() {
            return Ok(());
        }

        create_dir(root).map_err(StorageError::from)
    }

    pub fn clear() -> Result<(), StorageError> {
        let root = Self::root()?;

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
