use super::encryptor::{AESEncryptor, BlankEncryptor, DynamicEncryptor, Encryprtor};

const BLANKENCRYPTOR_ID: u8 = 0;
const AESENCRYPTOR_ID: u8 = 1;

pub trait Identifiable {
    fn id(&self) -> u8;
}

impl Identifiable for DynamicEncryptor {
    fn id(&self) -> u8 {
        self.0
    }
}

impl Identifiable for BlankEncryptor {
    fn id(&self) -> u8 {
        BLANKENCRYPTOR_ID
    }
}

impl Identifiable for AESEncryptor {
    fn id(&self) -> u8 {
        AESENCRYPTOR_ID
    }
}

pub fn encryptor_from_id(id: u8, key: &[u8]) -> Option<Box<dyn Encryprtor>> {
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
    fn test_encryptor_from_id() {
        let k = "a";
        let mut a = encryptor_from_id(BLANKENCRYPTOR_ID, k.as_ref()).unwrap();
        let s = "foobar";
        assert_eq!(
            a.encrypt(s.as_ref()).unwrap(),
            s.bytes().collect::<Vec<u8>>().into_boxed_slice()
        );
    }
}
