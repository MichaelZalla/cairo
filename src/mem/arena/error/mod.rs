use std::{alloc::LayoutError, error::Error, fmt::Display};

#[derive(Debug)]
pub enum AllocatorError {
    Null,
    Layout(LayoutError),
    InvalidArguments,
}

impl Display for AllocatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            AllocatorError::Null => "AllocatorError::Null".to_string(),
            AllocatorError::Layout(err) => format!("AllocatorError::Layout({})", err.to_string()),
            AllocatorError::InvalidArguments => "AllocatorError::InvalidArguments".to_string(),
        };

        write!(f, "{}", description)?;

        Ok(())
    }
}

impl Error for AllocatorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AllocatorError::Null => None,
            AllocatorError::Layout(err) => err.source(),
            AllocatorError::InvalidArguments => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            AllocatorError::Null => None,
            AllocatorError::Layout(err) => Some(err),
            AllocatorError::InvalidArguments => None,
        }
    }
}
