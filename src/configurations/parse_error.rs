/// Represents all the possible error that can appear in the parsing process
#[derive(Debug, std::cmp::PartialEq)]
pub enum ParseError {
    /// It will appear when there isn't a configuration at all
    ErrorIncompleteConfiguration,

    /// It will appear when the value to parse isn't in the correct format  
    ErrorCantParseValue,

    /// It will appear when there isn't a value with a given property name
    ErrorFieldNotFound,

    /// It will appear when the property name appears more than ones
    ErrorEncounterFieldMoreThanOnes,

    /// It will appear when the line of the configuration isn't given by the format `key: value`
    ErrorInvalidFormat,

    /// It will appear when there given readable gives an error when read 
    ErrorReadableError,

    /// It will appear when there isn't a structure with a given property name
    ErrorConfigurationNoFount,
}
