// Data structure for a row (line) of text
// Should only support basic features such as:
// deleting/inserting/replacing one character
// deleting a slice of text
// delete until end of row
// appending text to the end of the row
// return text to be rendered in unicode
//
// NOTE: No row should ever contain '\n', instead
// it should be added directly when printing, such as
// println!("{row}")

use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug, PartialEq)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    #[must_use]

    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let result: String = self
            .string
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .map(|grapheme| if grapheme == "\t" { " " } else { grapheme })
            .collect();

        result
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
        } else {
            let mut result = String::new();
            let mut length = 0;
            for (index, grapheme) in self.string[..].graphemes(true).enumerate()
            {
                if index == at {
                    length += 1;
                    result.push(c);
                }
                length += 1;
                result.push_str(grapheme);
            }
            self.len = length;
            self.string = result;
        }
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut result: String =
            self.string[..].graphemes(true).take(at).collect();
        let remainder: String =
            self.string[..].graphemes(true).skip(at + 1).collect();
        result.push_str(&remainder);
        self.string = result;
        self.update_len();
    }

    pub fn delete_slice(&mut self, start: usize, end: usize) {
        let mut first_slice: String =
            self.string[..].graphemes(true).take(start).collect();
        let second_slice: String = self.string[..]
            .graphemes(true)
            .skip(end.saturating_add(1))
            .collect();
        first_slice.push_str(second_slice.as_str());
        self.string = first_slice;
        self.update_len();
    }

    pub fn delete_until_eol(&mut self, start: usize) {
        self.string = self.string[..].graphemes(true).take(start).collect();
        self.update_len();
    }

    pub fn append(&mut self, row: &Row) {
        // Maybe we should take a string only, not a row
        self.string = format!("{}{}", self.string, row.string);
        self.update_len();
    }

    #[must_use]
    pub fn split(&mut self, at: usize) -> Self {
        let first_half: String =
            self.string[..].graphemes(true).take(at).collect();
        let second_half: String =
            self.string[..].graphemes(true).skip(at).collect();
        self.string = first_half;
        self.update_len();
        Self::from(&second_half[..])
    }

    pub fn slice(&self, start: usize, end: usize) -> &str {
        if end == 0 {
            return &self.string[start..];
        }
        &self.string[start..end]
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }
}
