use codec::{Decode, Encode};
use gstd::{prelude::*, TypeInfo};

/// Used to represent high and low parts of unsigned 256-bit integer.
pub type RandomSeed = (u128, u128);

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct Random {
    pub randomness: RandomSeed,
    pub signature: Vec<u8>,
    pub prev_signature: Vec<u8>,
}
