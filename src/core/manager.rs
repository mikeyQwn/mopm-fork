use std::collections::HashMap;

use thiserror::Error;

use super::encryptor::{Encryprtor, EncryprtorError};

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
}
