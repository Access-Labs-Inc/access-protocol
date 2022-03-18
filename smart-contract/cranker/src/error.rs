use std::fmt::Display;

#[derive(Debug)]
pub enum ProgramError {
    Rpc,
}

impl Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramError::Rpc => write!(f, "RPC error"),
        }
    }
}

impl std::error::Error for ProgramError {}
