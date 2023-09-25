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
        let mut result = String::new();
        #[allow(clippy::arithmetic_side_effects)]
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if grapheme == "\t" {
                result.push(' ');
            } else {
                result.push_str(grapheme);
            }
        }
        result
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
        } else {
            let mut result = String::new();
            let mut length = 0;
            for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
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
        let mut result: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at + 1).collect();
        result.push_str(&remainder);
        self.string = result;
        self.update_len();
    }

    pub fn append(&mut self, row: &Row) {
        // Maybe we should take a string only, not a row
        self.string = format!("{}{}", self.string, row.string);
        self.update_len();
    }

    #[must_use]
    pub fn split(&mut self, at: usize) -> Self {
        let first_half: String = self.string[..].graphemes(true).take(at).collect();
        let second_half: String = self.string[..].graphemes(true).skip(at).collect();
        self.string = first_half;
        self.update_len();
        Self::from(&second_half[..])
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
