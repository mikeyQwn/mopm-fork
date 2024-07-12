use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Version {
    V0_0,
}

impl Version {
    pub fn from_u8(id: u8) -> Option<Self> {
        Version::try_from(id).ok()
    }

    pub fn to_u8(&self) -> u8 {
        (*self).into()
    }

    pub fn current_version() -> Self {
        Self::V0_0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn sanity_check() {
        assert_eq!(std::mem::size_of::<Version>(), 1);
    }
}
