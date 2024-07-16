use std::{
    collections::HashMap,
    io::{self, Read, Write},
    mem::size_of,
};

use thiserror::Error;

use super::{
    encoding::version::Version,
    encryptor::{DynamicEncryptor, Encryprtor},
    hasher::{Hasher, Sha256Hasher},
    identifiers::{encryptor_from_id, Identifiable},
    manager::PasswordManager,
};

#[derive(Error, Debug)]
pub enum EncoderError {
    #[error("cannot parse the body")]
    BodyParseError,
    #[error("header size is not correct")]
    InvalidHeaderSize,
    #[error("could not read from reader")]
    ReaderError(io::Error),
    #[error("invalid header format")]
    HeaderParseError,
    #[error("unsupported encryptor version")]
    UnsupportedEncryptorVersionError,
    #[error("invalid key")]
    IvalidKeyError,
}

pub struct Encoder {}

impl Encoder {
    pub fn decode(
        key: &[u8],
        reader: &mut impl Read,
    ) -> Result<PasswordManager<DynamicEncryptor>, EncoderError> {
        let header = Header::try_from_reader(reader)?;
        let mut encryptor = encryptor_from_id(header.encryptor_id, key)
            .ok_or(EncoderError::UnsupportedEncryptorVersionError)?;

        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf).unwrap();
        let body_decrypted = encryptor.decrypt(&buf).unwrap();

        if header
            .body_sha
            .iter()
            .ne(Sha256Hasher::new().hash(&body_decrypted).iter())
        {
            return Err(EncoderError::IvalidKeyError);
        };

        let kv = Body::try_from_bytes(body_decrypted.as_ref()).unwrap();

        Ok(PasswordManager::from_raw_parts(
            kv,
            DynamicEncryptor(header.encryptor_id, encryptor),
        ))
    }

    pub fn encode<T>(w: &mut impl Write, pm: &mut PasswordManager<T>)
    where
        T: Encryprtor + Identifiable,
    {
        let body_bytes = Body::to_bytes(&pm.kv);
        let body_sha = Sha256Hasher::new().hash(&body_bytes);

        let body_encrypted = pm.encryptor.encrypt(&body_bytes).unwrap();

        let header = Header {
            version: Version::current_version(),
            encryptor_id: pm.encryptor.id(),
            body_sha: body_sha[..].try_into().unwrap(),
        };

        let bytes = header.to_bytes();
        let _ = w.write(&bytes);
        let _ = w.write(&body_encrypted);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    version: Version,
    encryptor_id: u8,
    body_sha: [u8; 32],
}

impl Header {
    const SIZE: usize = std::mem::size_of::<Self>();

    pub fn try_from_reader(r: &mut impl Read) -> Result<Self, EncoderError> {
        let mut buf = [0; Self::SIZE];
        let n = r.read(&mut buf).map_err(EncoderError::ReaderError)?;
        if n != buf.len() {
            return Err(EncoderError::InvalidHeaderSize);
        }

        Self::try_from_bytes(buf)
    }
    pub fn try_from_bytes(bytes: [u8; Self::SIZE]) -> Result<Self, EncoderError> {
        let version = Version::from_u8(bytes[0]).ok_or(EncoderError::HeaderParseError)?;
        let encoder_id = bytes[1];
        let body_sha = bytes[2..]
            .try_into()
            .or(Err(EncoderError::HeaderParseError))?;

        Ok(Self {
            version,
            encryptor_id: encoder_id,
            body_sha,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut res = [0; Self::SIZE];
        res[0] = Version::current_version().to_u8();
        res[1] = self.encryptor_id;
        res[2..].copy_from_slice(&self.body_sha);
        res
    }
}

#[derive(Debug, PartialEq)]
pub struct Body {
    kv: HashMap<String, Box<[u8]>>,
}

impl Body {
    pub fn to_bytes(kv: &HashMap<String, Box<[u8]>>) -> Vec<u8> {
        kv.iter().fold(Vec::new(), |mut acc, (key, value)| {
            acc.extend((key.len() as u64).to_be_bytes());
            acc.extend((value.len() as u64).to_be_bytes());
            acc.extend(key.as_bytes());
            acc.extend(value.iter());
            acc
        })
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<HashMap<String, Box<[u8]>>, EncoderError> {
        if bytes.is_empty() {
            return Ok(HashMap::new());
        }
        let mut kv = HashMap::new();
        let mut iter = bytes.iter().copied().peekable();

        while iter.peek().is_some() {
            let key_length = Self::read_u64(&mut iter)? as usize;
            let value_length = Self::read_u64(&mut iter)? as usize;

            let key: Vec<u8> = iter.by_ref().take(key_length).collect();
            if key.len() != key_length {
                return Err(EncoderError::BodyParseError);
            }
            let value: Vec<u8> = iter.by_ref().take(value_length).collect();
            if value.len() != value_length {
                return Err(EncoderError::BodyParseError);
            }

            let key_string = String::from_utf8(key).or(Err(EncoderError::BodyParseError))?;
            kv.insert(key_string, value.into_boxed_slice());
        }

        Ok(kv)
    }

    fn read_u64(iter: &mut impl Iterator<Item = u8>) -> Result<u64, EncoderError> {
        Ok(u64::from_be_bytes(
            iter.take(size_of::<u64>())
                .collect::<Vec<u8>>()
                .try_into()
                .or(Err(EncoderError::BodyParseError))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::core::encryptor::{AESEncryptor, BlankEncryptor};

    use super::*;

    #[test]
    pub fn test_body() {
        let mut kv = HashMap::new();
        kv.insert(
            "foo".to_string(),
            "bar".bytes().collect::<Vec<u8>>().into_boxed_slice(),
        );
        kv.insert(
            "".to_string(),
            "".bytes().collect::<Vec<u8>>().into_boxed_slice(),
        );
        kv.insert(
            "ƥƫƯȭ".to_string(),
            "ƥḌ ".bytes().collect::<Vec<u8>>().into_boxed_slice(),
        );
        assert_eq!(
            kv,
            Body::try_from_bytes(Body::to_bytes(&kv).as_ref()).unwrap()
        );
        assert_eq!(kv.len(), 3);
    }

    #[test]
    pub fn test_header() {
        let a = Header {
            version: Version::V0_0,
            encryptor_id: 100,
            body_sha: [1; 32],
        };

        let bytes = a.to_bytes();
        let b = Header::try_from_bytes(bytes).unwrap();

        assert_eq!(a, b)
    }

    #[test]
    pub fn test_encoder() {
        let mut pm = PasswordManager::from_raw_parts(HashMap::new(), AESEncryptor::new("foobar"));
        let _ = pm.store_password("foo".to_string(), "bar");
        let _ = pm.store_password("foo2".to_string(), "baz");
        let mut v = Vec::new();
        Encoder::encode(&mut v, &mut pm);
        let mut c = Cursor::new(v);
        let pm2 = Encoder::decode(b"foobar", &mut c).unwrap();
        assert_eq!(pm.encryptor.id(), pm2.encryptor.id());
        assert_eq!(pm.kv, pm2.kv);

        assert_eq!(pm.get_password("foo2"), Ok("baz".to_string()))
    }

    #[test]
    pub fn test_different_encoder() {
        let mut pm = PasswordManager::from_raw_parts(HashMap::new(), BlankEncryptor::new());
        let _ = pm.store_password("foo".to_string(), "bar");
        let _ = pm.store_password("foo2".to_string(), "baz");
        let mut v = Vec::new();
        Encoder::encode(&mut v, &mut pm);
        let mut c = Cursor::new(v);
        let pm2 = Encoder::decode(b"foobar", &mut c).unwrap();
        assert_eq!(pm.encryptor.id(), pm2.encryptor.id());
        assert_eq!(pm.kv, pm2.kv);

        assert_eq!(pm.get_password("foo2"), Ok("baz".to_string()))
    }
}
