use super::hash::HashType;

#[derive(Debug)]
pub struct Outpoint {
    pub hash: HashType,
    pub index: u32,
}