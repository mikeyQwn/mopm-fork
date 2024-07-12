use std::{
    collections::HashMap,
    io::{self, Read},
};

use thiserror::Error;

use super::encoding::version::Version;

#[derive(Error, Debug)]
enum EncoderError {
    #[error("cannot parse the body")]
    BodyParseError,
    #[error("header size is not correct")]
    InvalidHeaderSize,
    #[error("could not read from reader")]
    ReaderError(io::Error),
    #[error("invalid header format")]
    HeaderParseError,
}

const SEPARATOR_KV: u8 = 0;
const SEPARATOR_ENTRY: u8 = 1;

pub struct Encoder {}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    version: Version,
    hasher_id: u8,
    encoder_id: u8,
    body_sha: [u8; 32],
}

impl Header {
    pub fn try_from_reader(r: &mut impl Read) -> Result<Self, EncoderError> {
        let mut buf = [0; std::mem::size_of::<Self>()];
        let n = r
            .read(&mut buf)
            .or_else(|err| Err(EncoderError::ReaderError(err)))?;
        if n != buf.len() {
            return Err(EncoderError::InvalidHeaderSize);
        }

        Self::try_from_bytes(buf)
    }

    pub fn try_from_bytes(bytes: [u8; std::mem::size_of::<Self>()]) -> Result<Self, EncoderError> {
        let version = Version::from_u8(bytes[0]).ok_or(EncoderError::HeaderParseError)?;
        let hasher_id = bytes[1];
        let encoder_id = bytes[2];
        let body_sha = bytes[3..]
            .try_into()
            .or(Err(EncoderError::HeaderParseError))?;

        Ok(Self {
            version,
            hasher_id,
            encoder_id,
            body_sha,
        })
    }

    pub fn to_bytes(&self) -> [u8; std::mem::size_of::<Self>()] {
        let mut res = [0; std::mem::size_of::<Self>()];
        res[0] = Version::current_version().to_u8();
        res[1] = self.hasher_id;
        res[2] = self.encoder_id;
        res.iter_mut()
            .skip(3)
            .zip(self.body_sha)
            .for_each(|(v, exp)| *v = exp);
        res
    }
}

#[derive(Debug, PartialEq)]
pub struct Body {
    kv: HashMap<String, String>,
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
                acc.extend(v.as_bytes());
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
                        .and_then(|v| {
                            String::from_utf8(v.to_vec()).or(Err(EncoderError::BodyParseError))
                        })?;
                    acc.insert(k, v);
                    Ok(acc)
                })?;
        Ok(Self { kv })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_body() {
        let mut kv = HashMap::new();
        kv.insert("foo".to_string(), "bar".to_string());
        kv.insert("".to_string(), "".to_string());
        kv.insert("ƥƫƯȭ".to_string(), "ƥḌ ".to_string());
        let data = Body { kv };
        assert_eq!(data, Body::try_from_bytes(data.to_bytes()).unwrap());
        assert_eq!(data.kv.len(), 3);
    }

    #[test]
    pub fn test_header() {
        let mut a = Header {
            version: Version::V0_0,
            hasher_id: 20,
            encoder_id: 100,
            body_sha: [1; 32],
        };

        let bytes = a.to_bytes();
        let b = Header::try_from_bytes(bytes).unwrap();

        assert_eq!(a, b)
    }
}
