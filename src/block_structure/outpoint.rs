use super::hash::HashType;

pub struct Outpoint {
    pub hash: HashType,
    pub index: u32,
}