use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::event::Key;
use termion::color;

const STATUS_BG_COLOR: color::Rgb = color::Rgb(75, 75, 75);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(200, 200, 200);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
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
    status_message: StatusMessage,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-Q = quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = match Document::open(file_name) {
                Ok(doc) => doc,
                Err(_) => {
                    initial_status = format!("Err: could not open file {}", file_name);
                    Document::default()
                },
            };
            doc
        } else {
            Document::default()
        };


        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
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

    fn draw_cursor(&self) {
        // This is basically here so we can store the cursor's absolute
        // position, and display it according to the current line
        let Position {x, y} = self.cursor_position;
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
            return
        }
        Terminal::cursor_position(&Position {
            x: x.saturating_sub(self.offset.x),
            y: y.saturating_sub(self.offset.y),
        });
    }

    fn draw_status_bar(&self) {
        let width = self.terminal.size().width as usize;
        // Da pra definir file_name como imutavel?
        let mut status = "[No_name]".to_string();
        if let Some(name) = &self.document.file_name {
            status = name.clone();
            status.truncate(20);
        }

        let line_indicator = format!{
            "{},{}   {}%",
            self.cursor_position.y,
            self.cursor_position.x,
            {
                if self.cursor_position.y == 0 {
                    0
                }
                else {
                   (self.cursor_position.y*100)/self.document.len()
                }
            },
        };


        let len = status.len() + line_indicator.len();
        if width > status.len() {
            status.push_str(&" ".repeat(width - len));
        }

        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
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
            print!("{}", text);
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            #[rustfmt::skip]
            Key::Up 
            | Key::Down 
            | Key::Left 
            | Key::Right 
            | Key::Ctrl('d') 
            | Key::Ctrl('u') => {
                self.move_cursor(pressed_key)
            },
            _ => (),
        }
        self.scroll();
        Ok(())
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

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::Ctrl('u') => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::Ctrl('d') => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        self.cursor_position = Position { x, y }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("mtx editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row)
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
