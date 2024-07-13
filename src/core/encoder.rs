use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

use thiserror::Error;

use super::{
    encoding::version::Version,
    encryptor::{DynamicEncryptor, Encryprtor},
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
}

const SEPARATOR_KV: u8 = 0;
const SEPARATOR_ENTRY: u8 = 1;

pub struct Encoder {}

impl Encoder {
    pub fn decode(
        key: &[u8],
        reader: &mut impl Read,
    ) -> Result<PasswordManager<DynamicEncryptor>, EncoderError> {
        let header = Header::try_from_reader(reader)?;
        let encryptor = encryptor_from_id(header.encryptor_id, key)
            .ok_or(EncoderError::UnsupportedEncryptorVersionError)?;

        Ok(PasswordManager::from_raw_parts(
            HashMap::new(),
            DynamicEncryptor(header.encryptor_id, encryptor),
        ))
    }

    pub fn encode<T>(w: &mut impl Write, pm: &PasswordManager<T>)
    where
        T: Encryprtor + Identifiable,
    {
        let header = Header {
            version: Version::current_version(),
            encryptor_id: pm.encryptor_id(),
            body_sha: [0; 32],
        };
        let bytes = header.to_bytes();
        let _ = w.write(&bytes);
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
        let n = r
            .read(&mut buf)
            .or_else(|err| Err(EncoderError::ReaderError(err)))?;
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
    pub fn to_bytes(&self) -> Vec<u8> {
        self.kv
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, (k, v))| {
                if i != 0 {
                    acc.push(SEPARATOR_ENTRY);
                }
                acc.extend(k.as_bytes());
                acc.push(SEPARATOR_KV);
                acc.extend(v.iter());
                acc
            })
    }

    pub fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, EncoderError> {
        let kv =
            bytes
                .split(|v| *v == SEPARATOR_ENTRY)
                .try_fold(HashMap::new(), |mut acc, entry| {
                    let mut lr = entry.splitn(2, |v| *v == SEPARATOR_KV);
                    let k = lr
                        .next()
                        .ok_or(EncoderError::BodyParseError)
                        .and_then(|k| {
                            String::from_utf8(k.to_vec()).or(Err(EncoderError::BodyParseError))
                        })?;
                    let v = lr
                        .next()
                        .ok_or(EncoderError::BodyParseError)
                        .map(|v| v.to_vec().into_boxed_slice())?;
                    acc.insert(k, v);
                    Ok(acc)
                })?;
        Ok(Self { kv })
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
        let data = Body { kv };
        assert_eq!(data, Body::try_from_bytes(data.to_bytes()).unwrap());
        assert_eq!(data.kv.len(), 3);
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
        let pm = PasswordManager::from_raw_parts(HashMap::new(), AESEncryptor::new("foobar"));
        let mut v = Vec::new();
        Encoder::encode(&mut v, &pm);
        let mut c = Cursor::new(v);
        let pm2 = Encoder::decode(b"foobar", &mut c).unwrap();
        assert_eq!(pm.encryptor_id(), pm2.encryptor_id())
    }
}
