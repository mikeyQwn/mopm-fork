use sha2::Digest;

pub trait Hasher {
    fn hash<T>(&mut self, data: T) -> String
    where
        T: AsRef<[u8]>;
}

pub struct Sha256Hasher {}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hasher for Sha256Hasher {
    fn hash<T>(&mut self, data: T) -> String
    where
        T: AsRef<[u8]>,
    {
        let mut hasher = sha2::Sha256::new();
        hasher.update(data.as_ref());

        hex::encode(hasher.finalize())
    }
}
