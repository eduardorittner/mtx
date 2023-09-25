use crate::Position;
use crate::Row;
use std::cmp::Ordering;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
        })
    }

    pub fn save(&mut self, file_name: Option<String>) -> Result<(), Error> {
        if let Some(file_name) = file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
            return Ok(());
        }

        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        self.dirty = false;
        Ok(())
    }

    #[allow(clippy::indexing_slicing)]
    pub fn insert(&mut self, at: &Position, c: char) {
        let len = self.rows.len();
        match at.y.cmp(&len) {
            Ordering::Greater => (),
            Ordering::Equal => {
                self.dirty = true;
                let mut row = Row::default();
                row.insert(0, c);
                self.rows.push(row);
            }
            Ordering::Less => {
                self.dirty = true;
                let row = &mut self.rows[at.y];
                row.insert(at.x, c);
            }
        }
    }

    #[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
    pub fn insert_newline(&mut self, at: &Position) {
        let len = self.len();
        if at.y > len {
            return;
        }
        self.dirty = true;
        if at.y == len {
            self.rows.push(Row::default());
            return;
        }
        let new_row = self.rows[at.y].split(at.x);
        self.rows.insert(at.y + 1, new_row);
    }

    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    pub fn delete(&mut self, at: &Position) {
        // Panics when deleting the last line sometimes
        let len = self.rows.len();
        if at.y >= len {
            return;
        }

        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }

    #[must_use]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::Document;

    #[test]
    fn test_delete() {
        let doc = Document::default();
    }
}
