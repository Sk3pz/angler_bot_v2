use std::fmt::{Display, Formatter};

use rand_distr::TriangularError;
use serenity::all::HttpError;

// a play on "real error"
#[derive(Debug)]
pub enum ReelError {
    IOError(std::io::Error),
    MathError(TriangularError),
    RandomError(String),
    FileLoadFailed(String),
    HttpError(HttpError),
    Error(String),
}

impl Display for ReelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReelError::IOError(err) => write!(f, "IO Error: {}", err),
            ReelError::MathError(err) => write!(f, "Math Error: {}", err),
            ReelError::RandomError(err) => write!(f, "Random Error: {}", err),
            ReelError::FileLoadFailed(err) => write!(f, "Failed To Load: {}", err),
            ReelError::HttpError(err) => write!(f, "Http Error: {}", err),
            ReelError::Error(e) => write!(f, "Error: {}", e),
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

// Helper to allow `?` on Http Errors
impl From<HttpError> for ReelError {
    fn from(e: HttpError) -> Self {
        ReelError::HttpError(e)
    }
}

// Helper to allow `?` on any error that can be converted to a string
impl From<String> for ReelError {
    fn from(e: String) -> Self {
        ReelError::Error(e)
    }
}

impl Into<String> for ReelError {
    fn into(self) -> String {
        match self {
            ReelError::IOError(err) => format!("IO Error: {}", err),
            ReelError::MathError(err) => format!("Math Error: {}", err),
            ReelError::RandomError(err) => format!("Random Error: {}", err),
            ReelError::FileLoadFailed(err) => format!("Failed To Load: {}", err),
            ReelError::HttpError(err) => format!("Http Error: {}", err),
            ReelError::Error(e) => format!("Error: {}", e),
        }
    }
}