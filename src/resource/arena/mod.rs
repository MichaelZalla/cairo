use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::serde::PostDeserialize;

use super::handle::Handle;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ArenaEntry<T: PostDeserialize> {
    pub uuid: Uuid,
    // #[serde(flatten)]
    pub item: T,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Arena<T: PostDeserialize> {
    pub entries: Vec<Option<ArenaEntry<T>>>,
}

impl<T: PostDeserialize> PostDeserialize for Arena<T> {
    fn post_deserialize(&mut self) {
        for slot in self.entries.iter_mut() {
            match slot {
                Some(entry) => {
                    let item = &mut entry.item;

                    item.post_deserialize();
                }
                None => (),
            }
        }
    }
}

impl<T: PostDeserialize> Arena<T> {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_handle(&self, index: usize) -> Result<Handle, String> {
        assert!(index < self.entries.len());

        if let Some(entry) = &self.entries[index] {
            let handle = Handle {
                index,
                uuid: entry.uuid,
            };

            Ok(handle)
        } else {
            Err(format!(
                "Called Arena::get_handle() on invalid index `{}`!",
                index
            ))
        }
    }

    pub fn get(&self, handle: &Handle) -> Result<&ArenaEntry<T>, String> {
        match self.validate_handle(handle) {
            Ok(index) => {
                let entry = self.entries[index].as_ref().unwrap();

                Ok(entry)
            }
            Err(err) => Err(err),
        }
    }

    pub fn get_mut(&mut self, handle: &Handle) -> Result<&mut ArenaEntry<T>, String> {
        match self.validate_handle(handle) {
            Ok(index) => {
                let entry = self.entries[index].as_mut().unwrap();

                Ok(entry)
            }
            Err(err) => Err(err),
        }
    }

    pub fn insert(&mut self, uuid: Uuid, item: T) -> Handle {
        let entry = ArenaEntry { uuid, item };

        let index = self.get_first_empty_index();

        if index == self.entries.len() {
            self.entries.push(Some(entry));
        } else {
            self.entries[index] = Some(entry)
        };

        Handle { index, uuid }
    }

    fn get_first_empty_index(&self) -> usize {
        let mut first_empty_index: usize = 0;

        while first_empty_index < self.entries.len() && self.entries[first_empty_index].is_some() {
            first_empty_index += 1;
        }

        first_empty_index
    }

    pub fn remove(&mut self, handle: &Handle) -> Result<ArenaEntry<T>, String> {
        match self.validate_handle(handle) {
            Ok(index) => {
                if self.entries[index].is_none() {
                    Err(
                        (format!("Called Arena::remove() on empty slot at index {}!", index))
                            .to_string(),
                    )
                } else {
                    Ok(self.entries[index].take().unwrap())
                }
            }
            Err(err) => Err(err),
        }
    }

    fn validate_handle(&self, handle: &Handle) -> Result<usize, String> {
        if handle.index >= self.entries.len() {
            return Err(format!(
                "Invalid entry index {} for arena with length {}.",
                handle.index,
                self.entries.len()
            ));
        }

        match &self.entries[handle.index] {
            Some(entry) => {
                if entry.uuid == handle.uuid {
                    Ok(handle.index)
                } else {
                    Err(format!(
                        "Entry at index {} has non-matching UUID {} for handle with UUID {}!",
                        handle.index, entry.uuid, handle.uuid
                    ))
                }
            }
            None => Err(format!("Entry at index {} is None!", handle.index)),
        }
    }
}
