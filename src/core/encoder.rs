use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
enum EncoderError {
    #[error("cannot parse the body")]
    BodyParseError,
}

const SEPARATOR_KV: u8 = 0;
const SEPARATOR_ENTRY: u8 = 1;

pub struct Encoder {}

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
    pub fn test_() {
        let mut kv = HashMap::new();
        kv.insert("foo".to_string(), "bar".to_string());
        kv.insert("".to_string(), "".to_string());
        kv.insert("ƥƫƯȭ".to_string(), "ƥḌ ".to_string());
        let data = Body { kv };
        assert_eq!(data, Body::try_from_bytes(data.to_bytes()).unwrap());
        assert_eq!(data.kv.len(), 3);
    }
}
