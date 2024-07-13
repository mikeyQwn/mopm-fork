use std::iter;

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, KeyInit, KeySizeUser,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum EncryprtorError {
    #[error("cannot encrypt the data")]
    EncryptionError(String),
    #[error("cannot decrypt the data")]
    DecryptionError(String),
}

pub trait Encryprtor {
    fn encrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError>;

    fn decrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError>;
}

pub struct DynamicEncryptor(pub u8, pub Box<dyn Encryprtor>);
impl Encryprtor for DynamicEncryptor {
    fn encrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError> {
        self.1.encrypt(data)
    }

    fn decrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError> {
        self.1.decrypt(data)
    }
}

pub struct BlankEncryptor;

impl BlankEncryptor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Encryprtor for BlankEncryptor {
    fn encrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError> {
        Ok(data.into())
    }

    fn decrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError> {
        Ok(data.into())
    }
}

pub struct AESEncryptor {
    cipher: aes_gcm::Aes256Gcm,
}

impl AESEncryptor {
    pub fn new<T>(key: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        let key_size = aes_gcm::Aes256Gcm::key_size();
        let mut byte_key = key.as_ref();

        if byte_key.len() > key_size {
            byte_key = &byte_key[..key_size];
        };
        let key_bytes: Vec<_>;
        if byte_key.len() < key_size {
            key_bytes = byte_key
                .iter()
                .copied()
                .chain(iter::repeat(0).take(key_size - byte_key.len()))
                .collect();
            byte_key = &key_bytes[..key_size]
        };

        let cipher = aes_gcm::Aes256Gcm::new_from_slice(byte_key)
            .expect("byte_key is not resized correctly. this should not happen");
        Self { cipher }
    }
}

impl Encryprtor for AESEncryptor {
    fn encrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError> {
        let nonce = aes_gcm::Aes256Gcm::generate_nonce(OsRng);
        let encrypted_bytes = self
            .cipher
            .encrypt(&nonce, data.as_ref())
            .map_err(|err| EncryprtorError::EncryptionError(err.to_string()))?;

        let ciphertext = nonce.into_iter().chain(encrypted_bytes.into_iter());

        Ok(ciphertext.collect::<Vec<_>>().into_boxed_slice())
    }

    fn decrypt(&mut self, data: &[u8]) -> Result<Box<[u8]>, EncryprtorError> {
        const NONCE_LENGTH: usize = 12;

        if data.len() < NONCE_LENGTH {
            return Err(EncryprtorError::DecryptionError(
                "invalid nonce size".to_string(),
            ));
        };

        let (nonce, ciphertext) = data.split_at(NONCE_LENGTH);
        Ok(self
            .cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|err| EncryprtorError::DecryptionError(err.to_string()))?
            .into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        AESEncryptor::new("aaaa");
        AESEncryptor::new("");
        AESEncryptor::new("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"); // 32
        AESEncryptor::new("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"); // 33
    }

    #[test]
    fn test_enc_dec() {
        let mut encryptor = AESEncryptor::new("foobar");

        let data = "foo";
        let encrypted = encryptor.encrypt(data.as_ref()).unwrap();
        assert_eq!(
            Ok(data.to_owned()),
            String::from_utf8(encryptor.decrypt(encrypted.as_ref()).unwrap().into())
        );

        let data = "";
        let encrypted = encryptor.encrypt(data.as_ref()).unwrap();
        assert_eq!(
            Ok(data.to_owned()),
            String::from_utf8(encryptor.decrypt(encrypted.as_ref()).unwrap().into())
        );

        let data = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let encrypted = encryptor.encrypt(data.as_ref()).unwrap();
        assert_eq!(
            Ok(data.to_owned()),
            String::from_utf8(encryptor.decrypt(encrypted.as_ref()).unwrap().into())
        );
    }
}
