use std::fmt;

use crate::vec::vec3::Vec3;

#[derive(Default, Debug, Copy, Clone)]
pub enum StaticContactKind {
    #[default]
    Resting,
    Sliding,
    Rolling(f32),
}

impl fmt::Display for StaticContactKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Resting => "Resting".to_string(),
                Self::Sliding => "Sliding".to_string(),
                Self::Rolling(rolling_resistance) => format!("Rolling({})", rolling_resistance),
            }
        )
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct StaticContact {
    pub kind: StaticContactKind,
    pub point: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct StaticContactList<const N: usize> {
    pub static_contacts: [StaticContact; N],
    pub len: usize,
}

impl<const N: usize> Default for StaticContactList<N> {
    fn default() -> Self {
        Self {
            static_contacts: [Default::default(); N],
            len: 0,
        }
    }
}

impl<const N: usize> StaticContactList<N> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn iter(&self) -> Iter<'_, N> {
        Iter {
            len: self.len,
            current_index: 0,
            list: self,
        }
    }

    pub fn push(&mut self, contact: StaticContact) -> Result<(), String> {
        if self.len == N {
            return Err("Overflow.".to_string());
        }

        self.static_contacts[self.len] = contact;

        self.len += 1;

        Ok(())
    }
}

pub struct Iter<'a, const N: usize> {
    list: &'a StaticContactList<N>,
    current_index: usize,
    len: usize,
}

impl<'a, const N: usize> Iterator for Iter<'a, N> {
    type Item = &'a StaticContact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.list.len {
            return None;
        }

        let contact = &self.list.static_contacts[self.current_index];

        self.current_index += 1;

        Some(contact)
    }
}

impl<const N: usize> ExactSizeIterator for Iter<'_, N> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, const N: usize> IntoIterator for &'a StaticContactList<N> {
    type Item = &'a StaticContact;

    type IntoIter = Iter<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
