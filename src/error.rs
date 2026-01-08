use std::fmt;

#[derive(Debug)]
pub enum MusubiError {
    Network(String),
    Parse(String),
    Write(String),
    Config(String),
}

impl fmt::Display for MusubiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusubiError::Network(msg) => write!(f, "Network error: {}", msg),
            MusubiError::Parse(msg) => write!(f, "Parse error: {}", msg),
            MusubiError::Write(msg) => write!(f, "Write error: {}", msg),
            MusubiError::Config(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for MusubiError {}
