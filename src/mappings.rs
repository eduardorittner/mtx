use crate::commands::cursor_cmds;
use crate::commands::edit_cmds;
use crate::editor::Mode;
use crate::Document;
use crate::Position;
use crate::Row;
use std::collections::HashMap;

pub type MoveMappings = fn(&mut Position, &Document, Vec<bool>);

pub struct Mappings {
    mode: Mode,
    commands: HashMap<String, MoveMappings>,
}

impl Default for Mappings {
    fn default() -> Self {
        Self {
            mode: Mode::Normal,
            commands: {
                let mut commands = HashMap::new();
                commands.insert(
                    "h".to_string(),
                    cursor_cmds::move_cursor_left as MoveMappings,
                );
                commands.insert(
                    "j".to_string(),
                    cursor_cmds::move_cursor_down as MoveMappings,
                );
                commands.insert("k".to_string(), cursor_cmds::move_cursor_up as MoveMappings);
                commands.insert(
                    "l".to_string(),
                    cursor_cmds::move_cursor_right as MoveMappings,
                );
                commands
            },
        }
    }
}

impl Mappings {
    pub fn call_fn(&self, arg: &String, pos: &mut Position, doc: &Document, opts: Vec<bool>) {
        if let Some(func) = self.commands.get(arg) {
            func(pos, doc, opts);
        } else {
            panic!("DEu boras");
        }
    }
}
