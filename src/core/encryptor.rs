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
    fn encrypt<T>(&mut self, data: T) -> Result<String, EncryprtorError>
    where
        T: AsRef<[u8]>;

    fn decrypt<T>(&mut self, data: T) -> Result<String, EncryprtorError>
    where
        T: AsRef<[u8]>;
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
    fn encrypt<T>(&mut self, data: T) -> Result<String, EncryprtorError>
    where
        T: AsRef<[u8]>,
    {
        let nonce = aes_gcm::Aes256Gcm::generate_nonce(OsRng);
        let encrypted_bytes = self
            .cipher
            .encrypt(&nonce, data.as_ref())
            .map_err(|err| EncryprtorError::EncryptionError(err.to_string()))?;

        let ciphertext = nonce.into_iter().chain(encrypted_bytes.into_iter());

        Ok(hex::encode(ciphertext.collect::<Vec<_>>()))
    }

    fn decrypt<T>(&mut self, data: T) -> Result<String, EncryprtorError>
    where
        T: AsRef<[u8]>,
    {
        const NONCE_LENGTH: usize = 12;

        let data =
            hex::decode(data).map_err(|err| EncryprtorError::DecryptionError(err.to_string()))?;
        if data.len() < NONCE_LENGTH {
            return Err(EncryprtorError::DecryptionError(
                "invalid nonce size".to_string(),
            ));
        };

        let (nonce, ciphertext) = data.split_at(NONCE_LENGTH);
        let plaintext = self
            .cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|err| EncryprtorError::DecryptionError(err.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|err| EncryprtorError::DecryptionError(err.to_string()))
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
        let encrypted = encryptor.encrypt(data).unwrap();
        assert_eq!(Ok(data.to_owned()), encryptor.decrypt(encrypted));

        let data = "";
        let encrypted = encryptor.encrypt(data).unwrap();
        assert_eq!(Ok(data.to_owned()), encryptor.decrypt(encrypted));

        let data = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let encrypted = encryptor.encrypt(data).unwrap();
        assert_eq!(Ok(data.to_owned()), encryptor.decrypt(encrypted));
    }
}
