pub mod cursor_cmds {
    use crate::Document;
    use crate::Mode;
    use crate::Position;

    pub fn update_cursor(at: &mut Position, doc: &Document, mode: &Mode) {
        // This function must be called everytime an operation that
        // could leave the cursor in an invalid position is called
        // (maybe the functions should call this directly?)
        //
        // NOTE: there could be a way to guarantee that a cursor is
        // never left in an invalid position after a command, but doing
        // it this way just streamlines the process

        let doc_len = doc.len();
        let line_len: usize = match doc.row(at.y) {
            Some(line) => line.len(),
            // If Some(at.y) is None, then we won't reach
            // any of the else if statements so it can be
            // any value
            None => 0,
        };

        match mode {
            Mode::Normal => {
                if at.y >= doc_len {
                    at.y = doc_len.saturating_sub(1);
                    at.x = doc.row(at.y).unwrap().len();
                } else if at.x >= line_len {
                    at.x = line_len.saturating_sub(1);
                }
            }
            Mode::Visual => {
                if at.y >= doc_len {
                    at.y = doc_len.saturating_sub(1);
                    at.x = doc.row(at.y).unwrap().len();
                } else if at.x >= line_len {
                    at.x = line_len;
                }
            }
            Mode::Insert => {
                if at.y >= doc_len {
                    at.y = doc_len.saturating_sub(1);
                    at.x = doc.row(at.y).unwrap().len().saturating_sub(1);
                } else if at.x >= line_len {
                    at.x = line_len;
                }
            }
            Mode::Command => (),
        }
    }

    pub fn move_cursor_eol(at: &mut Position, doc: &Document, eol: bool) {
        let len = doc.row(at.y).unwrap().len();
        if eol {
            at.x = len.saturating_sub(1);
        } else {
            at.x = len.saturating_sub(2);
        }
    }

    pub fn move_cursor_bol(at: &mut Position) {
        at.x = 0;
    }

    pub fn move_cursor_up_n(at: &mut Position, number: usize) {
        at.y = at.y.saturating_sub(number);
    }

    pub fn move_cursor_up(at: &mut Position) {
        at.y = at.y.saturating_sub(1);
    }

    pub fn move_cursor_down_n(
        at: &mut Position,
        doc: &Document,
        number: usize,
    ) {
        let len = doc.len();
        if at.y.saturating_add(number) < len {
            at.y = at.y.saturating_add(number);
        } else {
            at.y = len.saturating_sub(1);
        }
    }

    pub fn move_cursor_down(at: &mut Position, doc: &Document) {
        let len = doc.len();
        if at.y.saturating_add(1) < len {
            at.y = at.y.saturating_add(1);
        } else {
            at.y = len.saturating_sub(1);
        }
    }

    pub fn move_cursor_left_n(
        at: &mut Position,
        doc: &Document,
        wrap: bool,
        number: usize,
    ) {
        for _ in [..number] {
            move_cursor_left(at, doc, wrap);
        }
    }

    pub fn move_cursor_left(at: &mut Position, doc: &Document, wrap: bool) {
        // NOTE: if wrap option is set, then it wraps until the '\n'
        // character, not before it

        if let Some(row) = doc.row(at.y) {
            let width = row.len();
            if at.x == 0 && at.y > 0 && wrap {
                at.y = at.y.saturating_sub(1);
                at.x = doc.row(at.y).unwrap().len();
            } else if at.x > width && width > 0 {
                // When the actual cursor is further to the right than the line
                at.x = width.saturating_sub(1);
            } else {
                at.x = at.x.saturating_sub(1);
            }
        }
    }

    pub fn move_cursor_right_n(
        at: &mut Position,
        doc: &Document,
        eol: bool,
        wrap: bool,
        number: usize,
    ) {
        for _ in [..number] {
            move_cursor_right(at, doc, eol, wrap);
        }
    }

    pub fn move_cursor_right(
        at: &mut Position,
        doc: &Document,
        eol: bool,
        wrap: bool,
    ) {
        match doc.row(at.y) {
            Some(row) => {
                let width = row.len();
                if at.x < width.saturating_sub(1) {
                    at.x = at.x.saturating_add(1);
                } else if at.x == width.saturating_sub(1) && eol {
                    at.x = at.x.saturating_add(1);
                } else if at.x == width
                    && at.y < doc.len().saturating_sub(1)
                    && wrap
                {
                    at.x = 0;
                    at.y = at.y.saturating_add(1);
                }
            }
            None => (),
        }
    }

    pub fn move_next_word(at: &mut Position, doc: &Document) {}

    pub fn move_last_word(at: &mut Position, doc: &Document) {}

    pub fn move_page_up(
        at: &mut Position,
        offset: &mut Position,
        terminal_height: usize,
    ) {
        at.y = at.y.saturating_sub(terminal_height);
        offset.y = offset.y.saturating_sub(terminal_height);
    }

    pub fn move_page_down(
        at: &mut Position,
        offset: &mut Position,
        doc: &Document,
        terminal_height: usize,
    ) {
        let height = doc.len();
        at.y = if at.y.saturating_add(terminal_height) < height {
            offset.y = offset.y.saturating_add(terminal_height);
            at.y.saturating_add(terminal_height)
        } else {
            offset.y = offset
                .y
                .saturating_add(height.saturating_sub(at.y).saturating_sub(1));
            height - 1
        }
    }
}

pub mod edit_cmds {
    use crate::commands::cursor_cmds;
    use crate::Document;
    use crate::Position;

    pub fn delete(at: &Position, doc: &mut Document) {
        doc.delete(at);
    }

    pub fn delete_line(at: &mut Position, doc: &mut Document) {
        let len = doc.len();
        if len != 0 {
            doc.delete_line(at.y);
            if at.y == doc.len() {
                at.y = at.y.saturating_sub(1);
            }
        }
    }

    pub fn delete_selection(at: &Position, end: &Position, doc: &mut Document) {
        doc.delete_slice(at, end);
    }

    pub fn delete_until_eol(at: &Position, doc: &mut Document) {
        // Deletes all the characters below the cursor and to
        // the right except for '\n'

        doc.delete_until_eol(&at);
    }

    pub fn delete_to_eol(at: &Position, doc: &mut Document) {
        // Similar to delete_until_eol(), however this function deletes
        // the newline character and appends the next line to the current
        // one

        doc.delete_to_eol(at);
    }

    pub fn insert(at: &Position, doc: &mut Document, c: char) {
        doc.insert(at, c);
    }

    pub fn insert_newline(at: &mut Position, doc: &mut Document) {
        doc.insert_newline(at);
    }

    pub fn insert_newline_below(at: &mut Position, doc: &mut Document) {
        match doc.row(at.y) {
            Some(row) => {
                doc.insert_newline(&Position {
                    x: row.len(),
                    y: at.y,
                });
            }
            None => doc.insert_newline(&Position { x: 0, y: 0 }),
        }
    }

    pub fn insert_newline_above(at: &mut Position, doc: &mut Document) {
        match doc.row(at.y.saturating_sub(1)) {
            Some(row) => {
                doc.insert_newline(&Position {
                    x: row.len(),
                    y: at.y.saturating_sub(1),
                });
            }
            None => doc.insert_newline(&Position { x: 0, y: 0 }),
        }
    }

    pub fn delete_backspace(at: &mut Position, doc: &mut Document) {
        cursor_cmds::move_cursor_left(at, &doc, true);
        delete(at, doc);
    }
}

pub mod visual_cmds {
    use crate::Document;
    use crate::Mode;
    use crate::Position;
    use crate::SelectedText;

    pub fn enter_visual_mode(
        at: &Position,
        selected: &mut SelectedText,
        mode: &mut Mode,
    ) {
        // TODO: add bound checking for current position
        selected.start = at.clone();
        selected.end = at.clone();
        *mode = Mode::Visual;
    }

    pub fn update_selection(
        at: &Position,
        selected: &mut SelectedText,
        doc: &Document,
    ) {
        let len = doc.row(at.y).unwrap().len().saturating_sub(1);
        if at.x >= len {
            selected.end = Position { x: len, y: at.y };
        } else {
            selected.end = at.clone();
        }
    }
}
