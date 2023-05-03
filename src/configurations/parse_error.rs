#[derive(Debug, std::cmp::PartialEq)]
pub enum ParseError {
    ErrorIncompleteConfiguration,
    ErrorFieldNotFound,
    ErrorEncounterFieldMoreThanOnes,
    ErrorInvalidFormat,
    ErrorFileDoesntExist,
}
