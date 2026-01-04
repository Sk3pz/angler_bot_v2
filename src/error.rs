use std::fmt::{Display, Formatter};

// a play on "real error"
#[derive(Debug)]
pub enum ReelError {
    CommandError(String),
    IOError(std::io::Error),
}

impl Display for ReelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReelError::CommandError(msg) => write!(f, "Command Error: {}", msg),
            ReelError::IOError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

// Helper to allow `?` on IO errors
impl From<std::io::Error> for ReelError {
    fn from(e: std::io::Error) -> Self {
        ReelError::IOError(e)
    }
}
