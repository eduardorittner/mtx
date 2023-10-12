use crate::commands::{cursor_cmds, edit_cmds, visual_cmds};
use crate::Document;
use crate::Row;
use crate::Terminal;
use std::cmp::Ordering;
use std::env;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const STATUS_BG_COLOR: color::Rgb = color::Rgb(75, 75, 75);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(200, 200, 200);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Mode {
    Normal = 0,
    Insert,
    Visual,
    Command, // For command line prompts
}

#[derive(Default, PartialEq, Eq, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.y.cmp(&other.y) {
            Ordering::Equal => return Some(self.x.cmp(&other.x)),
            ord => return Some(ord),
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.partial_cmp(other).unwrap();
    }
}

#[derive(Default)]
pub struct SelectedText {
    pub start: Position,
    pub end: Position,
}

struct StatusMessage {
    time: Instant,
    text: String,
}

impl StatusMessage {
    fn from(message: String) -> StatusMessage {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    hl_text: SelectedText,
    status_message: StatusMessage,
    mode: Mode,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-Q = quit");
        let document = if let Some(file_name) = args.get(1) {
            if let Ok(doc) = Document::open(file_name) {
                doc
            } else {
                initial_status =
                    format!("Err: could not open file {file_name}");
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default(),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            hl_text: SelectedText::default(),
            status_message: StatusMessage::from(initial_status),
            mode: Mode::Normal,
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            self.draw_cursor();
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match self.mode {
            Mode::Normal => match pressed_key {
                // Moving around
                Key::Ctrl('q') => self.should_quit = true,
                Key::Char('h') | Key::Left | Key::Backspace => {
                    cursor_cmds::move_cursor_left(
                        &mut self.cursor_position,
                        &self.document,
                        false,
                    );
                }
                Key::Char('j') | Key::Down => {
                    cursor_cmds::move_cursor_down(
                        &mut self.cursor_position,
                        &self.document,
                    );
                }
                Key::Char('k') | Key::Up => {
                    cursor_cmds::move_cursor_up(&mut self.cursor_position);
                }
                Key::Char('l') | Key::Right => {
                    cursor_cmds::move_cursor_right(
                        &mut self.cursor_position,
                        &self.document,
                        false,
                        false,
                    );
                }
                Key::Ctrl('d') => cursor_cmds::move_page_up(
                    &mut self.cursor_position,
                    &mut self.offset,
                    self.terminal.size().height as usize,
                ),
                Key::Ctrl('u') => cursor_cmds::move_page_down(
                    &mut self.cursor_position,
                    &mut self.offset,
                    &self.document,
                    self.terminal.size().height as usize,
                ),

                // Edit commands
                Key::Char('x') => {
                    edit_cmds::delete(&self.cursor_position, &mut self.document)
                }
                Key::Char('d') => match Terminal::read_key()? {
                    Key::Char('d') => edit_cmds::delete_line(
                        &mut self.cursor_position,
                        &mut self.document,
                    ),
                    _ => (),
                },
                Key::Char('o') => {
                    edit_cmds::insert_newline_below(
                        &mut self.cursor_position,
                        &mut self.document,
                    );
                    cursor_cmds::move_cursor_down(
                        &mut self.cursor_position,
                        &self.document,
                    );
                }
                Key::Char('O') => {
                    edit_cmds::insert_newline_above(
                        &mut self.cursor_position,
                        &mut self.document,
                    );
                    cursor_cmds::move_cursor_up(&mut self.cursor_position);
                }

                // Changing modes
                Key::Char('i') => self.mode = Mode::Insert,
                Key::Char('v') => visual_cmds::enter_visual_mode(
                    &self.cursor_position,
                    &mut self.hl_text,
                    &mut self.mode,
                ),
                Key::Char('a') => {
                    cursor_cmds::move_cursor_right(
                        &mut self.cursor_position,
                        &self.document,
                        true,
                        false,
                    );
                    self.mode = Mode::Insert;
                }

                // Misc
                Key::Char(' ') => match Terminal::read_key()? {
                    Key::Char('s') => self.save(false),
                    Key::Char('w') => self.save(true),
                    _ => (),
                },
                _ => (),
            },
            Mode::Insert => match pressed_key {
                Key::Ctrl('q') => self.should_quit = true,
                Key::Ctrl('c') => self.mode = Mode::Normal,
                Key::Ctrl('s') => self.save(false),
                Key::Ctrl('w') => self.save(true),

                // Edit commands
                Key::Delete => {
                    edit_cmds::delete(
                        &mut self.cursor_position,
                        &mut self.document,
                    );
                }
                Key::Backspace => {
                    edit_cmds::delete_backspace(
                        &mut self.cursor_position,
                        &mut self.document,
                    );
                }
                Key::Char('\n') => {
                    edit_cmds::insert_newline(
                        &mut self.cursor_position,
                        &mut self.document,
                    );
                    cursor_cmds::move_cursor_bol(&mut self.cursor_position);
                    cursor_cmds::move_cursor_down(
                        &mut self.cursor_position,
                        &self.document,
                    );
                }
                Key::Char(c) => {
                    edit_cmds::insert(
                        &self.cursor_position,
                        &mut self.document,
                        c,
                    );
                    cursor_cmds::move_cursor_right(
                        &mut self.cursor_position,
                        &self.document,
                        true,
                        false,
                    );
                }
                Key::Left => {
                    cursor_cmds::move_cursor_left(
                        &mut self.cursor_position,
                        &self.document,
                        false,
                    );
                }
                Key::Down => {
                    cursor_cmds::move_cursor_down(
                        &mut self.cursor_position,
                        &self.document,
                    );
                }
                Key::Up => {
                    cursor_cmds::move_cursor_up(&mut self.cursor_position);
                }
                Key::Right => {
                    cursor_cmds::move_cursor_right(
                        &mut self.cursor_position,
                        &self.document,
                        true,
                        false,
                    );
                }
                Key::Ctrl('d') => cursor_cmds::move_page_up(
                    &mut self.cursor_position,
                    &mut self.offset,
                    self.terminal.size().height as usize,
                ),
                Key::Ctrl('u') => cursor_cmds::move_page_down(
                    &mut self.cursor_position,
                    &mut self.offset,
                    &self.document,
                    self.terminal.size().height as usize,
                ),
                _ => (),
            },
            Mode::Visual => match pressed_key {
                Key::Ctrl('q') => self.should_quit = true,
                Key::Char('v') => self.mode = Mode::Normal,

                Key::Ctrl('c') => self.mode = Mode::Normal,
                Key::Char('h') | Key::Left | Key::Backspace => {
                    cursor_cmds::move_cursor_left(
                        &mut self.cursor_position,
                        &self.document,
                        false,
                    );
                    visual_cmds::update_selection(
                        &self.cursor_position,
                        &mut self.hl_text,
                    );
                }
                Key::Char('j') | Key::Down => {
                    cursor_cmds::move_cursor_down(
                        &mut self.cursor_position,
                        &self.document,
                    );
                    visual_cmds::update_selection(
                        &self.cursor_position,
                        &mut self.hl_text,
                    )
                }
                Key::Char('k') | Key::Up => {
                    cursor_cmds::move_cursor_up(&mut self.cursor_position);
                    visual_cmds::update_selection(
                        &self.cursor_position,
                        &mut self.hl_text,
                    )
                }
                Key::Char('l') | Key::Right => {
                    cursor_cmds::move_cursor_right(
                        &mut self.cursor_position,
                        &self.document,
                        true,
                        false,
                    );
                    visual_cmds::update_selection(
                        &self.cursor_position,
                        &mut self.hl_text,
                    )
                }
                Key::Ctrl('d') => {
                    cursor_cmds::move_page_up(
                        &mut self.cursor_position,
                        &mut self.offset,
                        self.terminal.size().height as usize,
                    );
                    visual_cmds::update_selection(
                        &self.cursor_position,
                        &mut self.hl_text,
                    );
                }
                Key::Ctrl('u') => {
                    cursor_cmds::move_page_down(
                        &mut self.cursor_position,
                        &mut self.offset,
                        &self.document,
                        self.terminal.size().height as usize,
                    );
                    visual_cmds::update_selection(
                        &self.cursor_position,
                        &mut self.hl_text,
                    );
                }
                _ => (),
            },
            Mode::Command => {}
        }

        self.scroll();
        Ok(())
    }

    fn save(&mut self, save_as: bool) {
        // Currently the file_name is directly attached
        // to the file that it is saved to, maybe provide an option
        // to save the file as something else but keep the same
        // file_name
        let mut arg = None;

        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message =
                    StatusMessage::from("Save aborted".to_string());
                return;
            }
            self.document.file_name = new_name;
        } else if save_as {
            let name = self.prompt("Save as: ").unwrap_or(None);
            if name.is_none() {
                self.status_message =
                    StatusMessage::from("Save aborted".to_string());
                return;
            }
            arg = name;
        }

        if self.document.save(arg).is_ok() {
            self.status_message = StatusMessage::from(format!(
                "{} written",
                self.document.file_name.clone().unwrap()
            ));
        } else {
            self.status_message =
                StatusMessage::from("Error writing file".to_string());
        }
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn prompt(
        &mut self,
        prompt: &str,
    ) -> Result<Option<String>, std::io::Error> {
        // Is there a way to not have to add a
        // self.mode = insert before every return?
        // maybe a wrapper for command line prompts?

        let last_mode = self.mode.clone();
        self.mode = Mode::Command;
        let mut result = String::new();
        loop {
            self.status_message =
                StatusMessage::from(format!("{prompt}{result}"));
            self.refresh_screen()?;
            match Terminal::read_key()? {
                Key::Backspace => {
                    result.truncate(result.len().saturating_sub(1))
                }
                Key::Char('\n') => break,
                Key::Ctrl('q') => {
                    result.truncate(0);
                    self.mode = last_mode.clone();
                    break;
                }
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                _ => (),
            }
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            self.mode = last_mode;
            return Ok(None);
        }
        self.mode = last_mode;
        Ok(Some(result))
    }

    fn draw_cursor(&self) {
        // This is basically here so we can store the cursor's absolute
        // position, and display it according to the current line

        match self.mode {
            // cursor in insert mode cant be at the end of line
            Mode::Insert | Mode::Normal => {
                let Position { x, y } = self.cursor_position;
                let width = if let Some(row) = self.document.row(y) {
                    row.len()
                } else {
                    0
                };
                if x > width {
                    Terminal::cursor_position(&Position {
                        x: width.saturating_sub(self.offset.x),
                        y: y.saturating_sub(self.offset.y),
                    });
                } else {
                    Terminal::cursor_position(&Position {
                        x: x.saturating_sub(self.offset.x),
                        y: y.saturating_sub(self.offset.y),
                    });
                }
            }
            Mode::Command => Terminal::cursor_position(&Position {
                // Since we can't move right or left on command prompt
                // assume cursor is at the end of string
                x: self.status_message.text.len(),
                y: (self.terminal.size().height + 1) as usize,
            }),
            Mode::Visual => (),
        }
    }

    fn draw_status_bar(&self) {
        let width = self.terminal.size().width as usize;
        let mut status = "[No_name]".to_string();
        let modified = if self.document.is_dirty() {
            " {Modified}"
        } else {
            ""
        };

        if let Some(name) = &self.document.file_name {
            status = name.clone();
            status.truncate(20);
        }

        status = format!("{status}{modified}");

        let line_indicator = format! {
            "{},{}   {}%",
            self.cursor_position.y,
            self.cursor_position.x,
            {
                if self.cursor_position.y == 0 {
                    0
                }
                else {
                    (self.cursor_position.y.saturating_mul(100))
                        .saturating_div(self.document.len())
                }
            },
        };

        let len = status.len().saturating_add(line_indicator.len());

        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{status}{line_indicator}");
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{status}\r");
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn draw_message_bar(&self) {
        // Maybe we could only call this function if 5 seconds had passed
        // or the message was updated?
        Terminal::clear_current_line();
        let message = &self.status_message;
        if message.time.elapsed().as_secs() < 5 {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{text}");
        }
    }
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("mtx editor -- version {VERSION}");
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len).saturating_div(2);
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        println!("{welcome_message}\r");
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        // Decrement size of coument in digits
        let end = self.offset.x.saturating_add(width);
        // Not taking into account side bar size
        let row = row.render(start, end);
        println!("{row}\r");
    }

    fn draw_rows(&self) {
        // Separate this function into draw_starting_screen
        // and draw_rows
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
}

fn die(e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
