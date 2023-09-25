#![warn(clippy::all, clippy::pedantic, clippy::correctness)]
#![allow(
    clippy::missing_errors_doc,
    clippy::implicit_return,
    clippy::missing_docs_in_private_items,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else
)]

mod document;
mod editor;
mod row;
mod terminal;
// mod tests;
pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;

fn main() {
    #[allow(clippy::single_call_fn)]
    Editor::default().run();
}
