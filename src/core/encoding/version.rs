use std::fmt::Display;

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

    pub fn to_u8(self) -> u8 {
        self.into()
    }

    pub fn current_version() -> Self {
        Self::V0_0
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Version::V0_0 => write!(f, "v0.0"),
        }
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
