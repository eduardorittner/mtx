use crate::Position;
use crate::Row;
use std::cmp::Ordering;
use std::fs;
use std::io::{Error, Write};

#[derive(Default, Debug)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let rows: Vec<Row> = contents.lines().map(Row::from).collect();
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

    #[allow(clippy::indexing_slicing)]
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
        self.rows.insert(at.y.saturating_add(1), new_row);
    }

    #[allow(clippy::indexing_slicing)]
    pub fn delete(&mut self, at: &Position) {
        // Panics when deleting the last line sometimes
        let len = self.rows.len();
        if at.y >= len {
            return;
        }

        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y.saturating_add(1) < len {
            // When at the end of a line
            let next_row = self.rows.remove(at.y.saturating_add(1));
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }

    pub fn insert_spaces(&mut self, at: &Position, number: usize) {
        for _ in [..number] {
            self.insert(&at, ' ');
        }
    }

    pub fn delete_line(&mut self, at: usize) {
        self.rows.remove(at);
    }

    pub fn delete_lines(&mut self, start: usize, end: usize) {
        // Go in reverse since after deleting a line all further
        // indexes are now -1, so we don't have to deal with that.
        for i in (start..end.saturating_add(1)).rev() {
            self.rows.remove(i);
        }
    }

    pub fn delete_slice(&mut self, start: &Position, end: &Position) {
        // TODO: clean this shit
        // Deletes a (continuous) slice of text that may span multiple
        // lines or not
        // NOTE: start must be smaller than end

        if start.x == 0
            && end.x == self.row(end.y).unwrap().len().saturating_sub(1)
        {
            self.delete_lines(start.y, end.y);
            return;
        }

        match start.y.cmp(&end.y) {
            Ordering::Greater => (),
            Ordering::Equal => {
                self.row_as_mut(start.y)
                    .unwrap()
                    .delete_slice(start.x, end.x);
            }
            Ordering::Less => {
                self.delete_until_eol(start);
                let mut y = start.y.saturating_add(1);
                let next_line = start.y.saturating_add(1);
                while y < end.y {
                    self.delete_line(next_line);
                    y = y.saturating_add(1);
                }
                let last_row = self.row(next_line).unwrap();
                let last_row = if !last_row.is_empty() {
                    Row::from(last_row.slice(end.x.saturating_add(1), 0))
                } else {
                    if let Some(last_row) =
                        self.row(next_line.saturating_add(1))
                    {
                        // If line after cursor is not empty
                        Row::from(last_row.slice(0, 0))
                    } else {
                        return;
                    }
                };
                self.row_as_mut(start.y).unwrap().append(&last_row);
                self.delete_line(next_line);
            }
        }
    }

    pub fn delete_until_eol(&mut self, at: &Position) {
        if let Some(row) = self.row_as_mut(at.y) {
            row.delete_until_eol(at.x);
        }
    }

    pub fn delete_to_eol(&mut self, start: &Position) {
        // Deletes from current position until end of line character,
        // appending the line below to this one

        let next_row = self.rows.remove(start.y.saturating_add(1));
        let row = self.row_as_mut(start.y).unwrap();
        row.delete_until_eol(start.x);
        row.append(&next_row);
    }

    #[must_use]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn row_as_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }

    pub fn row_len(&self, index: usize) -> Option<usize> {
        match self.rows.get(index) {
            Some(x) => Some(x.len()),
            None => None,
        }
    }

    pub fn is_out_of_bounds(&self, at: &Position) -> bool {
        return at.y >= self.len() || at.x >= self.row_len(at.y).unwrap();
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

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        for (index, row) in self.rows.iter().enumerate() {
            let Some(other_row) = other.rows.get(index) else {
                return false;
            };
            if other_row != row {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::Position;

    #[test]
    fn test_insert_empty() {
        let mut doc = Document::default();
        let doc_test = Document::open("./tests/test1.txt").unwrap();
        doc.insert(&Position { x: 0, y: 0 }, 'c');
        assert_eq!(doc_test, doc);
    }

    #[test]
    fn test_insert_simple() {
        let mut doc = Document::open("./tests/2.in").unwrap();
        let doc_test = Document::open("./tests/2.out").unwrap();
        doc.insert(&Position { x: 10, y: 2 }, 'k');
        assert_eq!(doc_test, doc);
    }

    #[test]
    fn test_insert_newline() {
        let mut doc = Document::open("./tests/3.in").unwrap();
        let doc_test = Document::open("./tests/3.out").unwrap();
        doc.insert_newline(&Position { x: 14, y: 0 });
        assert_eq!(doc_test, doc);
    }

    #[test]
    fn test_delete_to_empty() {
        let mut doc = Document::open("./tests/4.in").unwrap();
        let doc_test = Document::default();
        doc.delete(&Position { x: 0, y: 0 });
        assert_eq!(doc_test, doc);
    }

    #[test]
    fn test_delete_simple() {
        let mut doc = Document::open("./tests/5.in").unwrap();
        let doc_test = Document::open("./tests/5.out").unwrap();
        doc.delete(&Position { x: 10, y: 1 });
        assert_eq!(doc_test, doc);
    }

    #[test]
    fn test_delete_newline() {
        let mut doc = Document::open("./tests/6.in").unwrap();
        let doc_test = Document::open("./tests/6.out").unwrap();
        doc.delete(&Position { x: 12, y: 0 });
        assert_eq!(doc_test, doc);
    }
}
