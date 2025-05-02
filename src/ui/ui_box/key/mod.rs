use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UIKey {
    hash: Option<String>,
}

impl fmt::Display for UIKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UIKey(hash={})",
            if let Some(hash) = &self.hash {
                format!("\"{}\"", hash)
            } else {
                "None".to_string()
            }
        )
    }
}

impl From<String> for UIKey {
    fn from(id: String) -> Self {
        Self { hash: Some(id) }
    }
}

impl UIKey {
    pub fn is_null(&self) -> bool {
        self.hash.is_none()
    }
}
