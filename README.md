# MTX - Minimal Text Editor

Minimal text editor made in rust following along with hecto-tutorial
and heavily inspired by NeoVim. 

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

### Cursor

- Cursor doesn't "remember" its position after going to a line that's smaller
than it (should be fixed with draw-cursor func)
- Cursor is able to go until line.len(), not line.len() - 1. This is necessary
for now as there is no "append" insert capability

### Text editing

- Add editing text capabilities (insert, remove, select)
- Add modes (normal, insert, maybe visual)

### Visual

- Add line count on the left
- Add basic syntax highlighting support
