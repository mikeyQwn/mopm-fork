use sha2::Digest;

pub trait Hasher {
    fn hash(&mut self, data: &[u8]) -> Box<[u8]>;
}

pub struct Sha256Hasher {}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hasher for Sha256Hasher {
    fn hash(&mut self, data: &[u8]) -> Box<[u8]> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(data.as_ref());

        let v: [u8; 32] = hasher.finalize().into();
        Box::from(v)
    }
}
