use std::{
    borrow::BorrowMut,
    collections::HashMap,
    io::{Cursor, Read},
};

use thiserror::Error;

use super::{
    encryptor::{AESEncryptor, Encryprtor, EncryprtorError},
    hasher::{Hasher, Sha256Hasher},
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PasswordManagerError {
    #[error("ecryptor error")]
    EncryptorError(EncryprtorError),
    #[error("no matching passwords found")]
    NoPasswordFound,
}

impl From<EncryprtorError> for PasswordManagerError {
    fn from(value: EncryprtorError) -> Self {
        Self::EncryptorError(value)
    }
}

#[derive(Debug)]
pub struct PasswordManager<T>
where
    T: Encryprtor,
{
    kv: HashMap<String, String>,
    encryptor: T,
}

impl<T> PasswordManager<T>
where
    T: Encryprtor,
{
    pub fn get_password(&mut self, key: &str) -> Result<String, PasswordManagerError> {
        let encrypted_password = self
            .kv
            .get(key)
            .ok_or(PasswordManagerError::NoPasswordFound)?;

        self.encryptor
            .decrypt(encrypted_password)
            .map_err(PasswordManagerError::from)
    }

    pub fn store_password(&mut self, key: String, value: &str) -> Result<(), PasswordManagerError> {
        let encrypted_password = self.encryptor.encrypt(value)?;

        self.kv.insert(key.to_string(), encrypted_password);
        Ok(())
    }
}

pub struct PasswordManagerOptions<E, B>
where
    E: Encryprtor,
    B: Read,
{
    hashed_key: String,

    encryptor: E,
    body: Option<B>,
}

impl PasswordManagerOptions<AESEncryptor, Cursor<Vec<u8>>> {
    pub fn new<T: AsRef<[u8]>>(key: T) -> Self {
        Self::new_with_hasher(key, Sha256Hasher::new().borrow_mut())
    }

    pub fn new_with_hasher<T, U>(key: T, hasher: &mut U) -> Self
    where
        T: AsRef<[u8]>,
        U: Hasher,
    {
        let hashed_key = hasher.hash(key);

        Self {
            hashed_key: hashed_key.clone(),

            encryptor: AESEncryptor::new(hashed_key),
            body: None,
        }
    }
}

impl<E, B> PasswordManagerOptions<E, B>
where
    E: Encryprtor,
    B: Read,
{
    pub fn with_encryptor<T, F>(self, f: F) -> PasswordManagerOptions<T, B>
    where
        T: Encryprtor,
        F: FnOnce(&str) -> T,
    {
        let encryptor = f(self.hashed_key.as_str());

        PasswordManagerOptions::<T, B> {
            hashed_key: self.hashed_key,
            encryptor: encryptor,
            body: self.body,
        }
    }

    pub fn build(self) -> PasswordManager<E> {
        PasswordManager {
            kv: HashMap::new(),
            encryptor: self.encryptor,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::encryptor::AESEncryptor;

    use super::*;

    #[test]
    fn test_load_store() {
        let mut pm = PasswordManager {
            kv: HashMap::new(),
            encryptor: AESEncryptor::new("foo"),
        };

        assert!(pm.store_password("foo".to_owned(), "bar").is_ok());
        assert_eq!(pm.get_password("foo"), Ok("bar".to_owned()));

        assert_eq!(
            pm.get_password("bar"),
            Err(PasswordManagerError::NoPasswordFound)
        );
    }

    #[test]
    fn test_builder() {
        let key = "Hello world!".to_string();
        let pm = PasswordManagerOptions::new(&key)
            .with_encryptor(|key| AESEncryptor::new(key))
            .build();

        match pm {
            PasswordManager {
                kv: v,
                encryptor: _,
            } if v.len() == 0 => {}
            _ => panic!("Password manager built incorrectly"),
        }
    }
}
