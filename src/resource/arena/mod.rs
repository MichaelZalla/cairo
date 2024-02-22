use uuid::Uuid;

use super::handle::Handle;

#[derive(Default, Debug, Clone)]
pub struct ArenaEntry<T> {
    pub uuid: Uuid,
    pub item: T,
}

#[derive(Default, Debug, Clone)]
pub struct Arena<T> {
    entries: Vec<Option<ArenaEntry<T>>>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
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
        // @TODO Validate `item`?

        let entry = ArenaEntry {
            uuid: uuid.clone(),
            item,
        };

        let mut first_empty_index: usize = 0;

        while first_empty_index < self.entries.len() && self.entries[first_empty_index].is_some() {
            first_empty_index += 1;
        }

        if first_empty_index == self.entries.len() {
            self.entries.push(Some(entry));
        } else {
            self.entries[first_empty_index] = Some(entry)
        };

        Handle {
            index: first_empty_index,
            uuid,
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
