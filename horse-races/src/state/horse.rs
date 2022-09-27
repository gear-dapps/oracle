use codec::{Decode, Encode};
use gstd::{prelude::*, TypeInfo};

#[derive(Debug, Clone, Hash, Encode, Decode, TypeInfo)]
pub struct Horse {
    pub max_speed: u8,
}

impl Horse {
    pub fn get_power(&self) -> u128 {
        self.max_speed.into()
    }
}
