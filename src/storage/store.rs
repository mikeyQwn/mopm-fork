use std::{
    fs::create_dir,
    io::{self, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use thiserror::Error;

use crate::core::{
    encoder::{Encoder, EncoderError},
    encryptor::Encryprtor,
    identifiers::Identifiable,
    manager::PasswordManager,
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
    #[error("error while reading/writing: `{0}`")]
    IoError(#[from] io::Error),
    #[error("encoder error: `{0}`")]
    EncoderError(#[from] EncoderError),
    #[error("path buf error: `{0}`")]
    PathBufError(#[from] core::convert::Infallible),
}

impl Storage {
    pub fn init<T>(pm: &mut PasswordManager<T>) -> Result<(), StorageError>
    where
        T: Encryprtor + Identifiable,
    {
        let root = Self::root()?;

        if root.exists() {
            return Err(StorageError::RootAlreadyExistsErorr);
        }

        create_dir(&root)?;

        let mut password_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(Self::data_file()?)
            .map_err(StorageError::from)?;

        Encoder::encode(&mut password_file, pm)?;
        Ok(())
    }

    pub fn create_dummies() -> Result<(), StorageError> {
        let (upper, lower, work) = Self::dummies()?;
        let upper_file = Self::upper_file()?;

        if !upper.exists() {
            create_dir(upper)?;
        };

        if !upper_file.exists() {
            let mut password_file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(upper_file)
                .map_err(StorageError::from)?;
            password_file.write(b"You are not supposed to see this. Get out.")?;
        }

        if !lower.exists() {
            create_dir(lower)?;
        };

        if !work.exists() {
            create_dir(work)?;
        };
        Ok(())
    }

    pub fn get_data_reader() -> Result<impl Read, StorageError> {
        std::fs::OpenOptions::new()
            .read(true)
            .open(Self::data_file()?)
            .map_err(StorageError::from)
    }

    pub fn get_data_writer() -> Result<impl Write, StorageError> {
        std::fs::OpenOptions::new()
            .write(true)
            .open(Self::data_file()?)
            .map_err(StorageError::from)
    }

    pub fn clear() -> Result<(), StorageError> {
        let root = Self::root()?;
        if !root.exists() {
            return Err(StorageError::RootDoesNotExistErorr);
        }

        std::fs::remove_dir_all(root).map_err(StorageError::from)
    }

    pub fn is_initialized() -> Result<bool, StorageError> {
        Ok(Self::root()?.exists() && Self::data_file()?.exists())
    }

    pub fn root() -> Result<PathBuf, StorageError> {
        let mut root = Self::homedir()?;
        root.push(".mopm");

        Ok(root)
    }

    fn data_file() -> Result<PathBuf, StorageError> {
        let mut data = Self::root()?;
        data.push(".data");

        Ok(data)
    }

    pub fn dummies() -> Result<(PathBuf, PathBuf, PathBuf), StorageError> {
        let upper = PathBuf::from_str("/tmp/mopm-dummy-upper")?;
        let lower = PathBuf::from_str("/tmp/mopm-dummy-lower")?;
        let work = PathBuf::from_str("/tmp/mopm-dummy-work")?;

        Ok((upper, lower, work))
    }

    pub fn upper_file() -> Result<PathBuf, StorageError> {
        let mut upper = PathBuf::from_str("/tmp/mopm-dummy-upper")?;
        upper.push("not-a-honeypot.txt");

        Ok(upper)
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
