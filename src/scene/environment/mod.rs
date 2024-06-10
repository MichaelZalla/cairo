use core::fmt;

use serde::{Deserialize, Serialize};

use crate::serde::PostDeserialize;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Environment {}

impl PostDeserialize for Environment {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Environment")
    }
}
