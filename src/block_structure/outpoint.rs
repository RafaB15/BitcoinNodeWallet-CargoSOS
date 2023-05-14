use super::hash::HashType;

#[derive(Debug, Clone)]
pub struct Outpoint {
    pub hash: HashType,
    pub index: u32,
}