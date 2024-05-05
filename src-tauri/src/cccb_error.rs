#[derive(Debug, Clone)]
pub enum CccbError {
    MajorConnError(String),
    MinorConnError(String),
}

impl std::fmt::Display for CccbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CccbError::MajorConnError(e) => write!(f, "major error: {}", e),
            CccbError::MinorConnError(e) => write!(f, "minor error: {}", e),
        }
    }
}
