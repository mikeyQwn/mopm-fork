use super::{
    encryptor::{AESEncryptor, BlankEncryptor, Encryprtor},
    hasher::{BlankHasher, Hasher, Sha256Hasher},
};

const BLANKHASHER_ID: u8 = 0;
const BLANKENCRYPTOR_ID: u8 = 0;
const SHA256HASHER_ID: u8 = 1;
const AESENCRYPTOR_ID: u8 = 1;

pub trait Identifiable {
    fn id() -> u8;
}

impl Identifiable for BlankHasher {
    fn id() -> u8 {
        BLANKHASHER_ID
    }
}

impl Identifiable for Sha256Hasher {
    fn id() -> u8 {
        SHA256HASHER_ID
    }
}

impl Identifiable for BlankEncryptor {
    fn id() -> u8 {
        BLANKENCRYPTOR_ID
    }
}

impl Identifiable for AESEncryptor {
    fn id() -> u8 {
        AESENCRYPTOR_ID
    }
}

pub fn hasher_from_id(id: u8) -> Option<Box<dyn Hasher>> {
    match id {
        BLANKHASHER_ID => Some(Box::new(BlankHasher::new())),
        SHA256HASHER_ID => Some(Box::new(Sha256Hasher::new())),
        _ => None,
    }
}

pub fn encryptor_from_id(id: u8, key: &str) -> Option<Box<dyn Encryprtor>> {
    match id {
        BLANKENCRYPTOR_ID => Some(Box::new(BlankEncryptor::new())),
        AESENCRYPTOR_ID => Some(Box::new(AESEncryptor::new(key))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hasher_from_id() {
        let mut a = hasher_from_id(BLANKHASHER_ID).unwrap();
        let s = "foobar";
        assert_eq!(
            a.hash(s.as_ref()),
            s.bytes().collect::<Vec<u8>>().into_boxed_slice()
        );
    }

    #[test]
    fn test_encryptor_from_id() {
        let k = "a";
        let mut a = encryptor_from_id(BLANKENCRYPTOR_ID, k).unwrap();
        let s = "foobar";
        assert_eq!(
            a.encrypt(s.as_ref()).unwrap(),
            s.bytes().collect::<Vec<u8>>().into_boxed_slice()
        );
    }
}
