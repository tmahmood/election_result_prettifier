use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct InvalidConstituencyName;

impl fmt::Display for InvalidConstituencyName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to get constituency name")
    }
}

impl Error for InvalidConstituencyName {
    fn description(&self) -> &str {
        "Failed to get constituency name"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
pub struct NotFamiliarRowType;

impl fmt::Display for NotFamiliarRowType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't detect the type of row automatically")
    }
}

impl Error for NotFamiliarRowType {
    fn description(&self) -> &str {
        "Can't detect the type of row automatically"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
