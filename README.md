# MTX - Minimal Text Editor

Minimal text editor made in rust following along with hecto-tutorial
and heavily inspired by NeoVim. 

## Using MTX

Mtx is based on neovim's (and therefore vim) style of text editing, which
are heavily reliant on "modes". The application starts in normal mode, where
you can move around using "h", "j", "k", and "l" to move around (or the arrow
keys). To enter insert mode press "i", to insert before the cursor, or "a" to
insert after it. To go back to normal mode press "Ctrl-c" or "Esc".
To save a file in insert mode, press "Ctrl-s" to save to the file's current
name or "Ctrl-w" to save to a different name. In normal mode press "<Space>s" 
and "<Space>w", respectively.

## Objectives

- Learn rust
- Implement a simple text editor base
- Implement the ability to change the internal text data structure representation
during runtime (or at least as a command argument). This would be a useful exercise
in understanding the different ways to implement text editing functionality and
also a nice way to check for their practical differences in speed, situations, etc.
That way, you could use a gap buffer for everything but have the ability to change
to a piece table representation for very large files.

## TODOS

### Project structure
- Add automated testing for basic functions such as insert, remove, etc. with
input and expected output
- Add way to load mappings from a mtx.conf file
- Hot reloading of config(?)

### Cursor

- Cursor doesn't "remember" its position after going to a line that's smaller
than it (should be fixed with draw-cursor func) -- DONE
- Cursor is able to go until line.len(), not line.len() - 1. This is necessary
for now as there is no "append" insert capability
- Cursor is not shown on status message prompt -- DONE

### Text editing

- Add editing text capabilities (insert, remove, select)
- Add modes (normal, insert, maybe visual)
- Undo/Redo operation
- Search 

### Visual

- Add line count on the left (probably not worth it for the hassle right now)
- Add basic syntax highlighting support

### Rendering

- Maybe only render what has been updated(?)
