pub mod cursor_cmds {
    use crate::Document;
    use crate::Position;

    pub fn move_cursor_eol(at: &mut Position, doc: &Document, opts: Vec<bool>) {
        if let Some(eol) = opts.get(0) {
            let len = doc.row(at.y).unwrap().len();
            if *eol {
                at.x = len.saturating_sub(1);
            } else {
                at.x = len.saturating_sub(2);
            }
        } else {
            // Maybe add a prompt error message here(?)
            ()
        }
    }

    pub fn move_cursor_bol(at: &mut Position, doc: &Document, _opts: Vec<bool>) {
        at.x = 0;
    }

    pub fn move_cursor_up(at: &mut Position, doc: &Document, _opts: Vec<bool>) {
        at.y = at.y.saturating_sub(1);
    }

    pub fn move_cursor_down(at: &mut Position, doc: &Document, _opts: Vec<bool>) {
        let len = doc.len().saturating_sub(1);
        if at.y < len {
            at.y = at.y.saturating_add(1);
        }
    }

    pub fn move_cursor_left(at: &mut Position, doc: &Document, opts: Vec<bool>) {
        if let Some(wrap) = opts.get(1) {
            let width = doc.row(at.y).unwrap().len();
            if at.x == 0 && at.y > 0 && *wrap {
                at.y = at.y.saturating_sub(1);
                at.x = doc.row(at.y).unwrap().len();
                return;
            } else if at.x > width && width > 0 {
                // When the actual cursor is further to the right than the line
                at.x = width.saturating_sub(1);
            } else {
                at.x = at.x.saturating_sub(1);
            }
        } else {
            ()
        }
    }

    pub fn move_cursor_right(at: &mut Position, doc: &Document, opts: Vec<bool>) {
        if let Some(eol) = opts.get(0) {
            if let Some(wrap) = opts.get(1) {
                let width = doc.row(at.y).unwrap().len();
                if at.x < width.saturating_sub(1) {
                    at.x = at.x.saturating_add(1);
                } else if at.x == width.saturating_sub(1) && *eol {
                    at.x = at.x.saturating_add(1);
                } else if at.x == width && at.y < doc.len().saturating_sub(1) && *wrap {
                    at.x = 0;
                    at.y = at.y.saturating_add(1);
                }
            } else {
                ()
            }
        } else {
            ()
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

    pub fn insert(at: &Position, doc: &mut Document, c: char) {
        doc.insert(at, c);
    }

    pub fn insert_newline(at: &mut Position, doc: &mut Document) {
        doc.insert_newline(at);
    }

    pub fn delete_backspace(at: &mut Position, doc: &mut Document) {
        cursor_cmds::move_cursor_left(at, &doc, vec![true]);
        if at.x == 0 && at.y == 0 {
            return;
        }
        delete(at, doc);
    }
}
