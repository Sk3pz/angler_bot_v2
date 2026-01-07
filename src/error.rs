use std::fmt::{Display, Formatter};

use rand_distr::TriangularError;

// a play on "real error"
#[derive(Debug)]
pub enum ReelError {
    IOError(std::io::Error),
    MathError(TriangularError),
}

impl Display for ReelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReelError::IOError(err) => write!(f, "IO Error: {}", err),
            ReelError::MathError(err) => write!(f, "Math Error: {}", err),
        }
    }
}

// Helper to allow `?` on IO errors
impl From<std::io::Error> for ReelError {
    fn from(e: std::io::Error) -> Self {
        ReelError::IOError(e)
    }
}

// Helper to allow `?` on Triangular errors
impl From<TriangularError> for ReelError {
    fn from(e: TriangularError) -> Self {
        ReelError::MathError(e)
    }
}
