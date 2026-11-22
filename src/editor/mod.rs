use std::{
    fs, io,
    path::{Path, PathBuf},
};

use ropey::Rope;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
struct Edit {
    start: usize,
    removed: String,
    inserted: String,
    cursor_before: Cursor,
    cursor_after: Cursor,
}

pub struct EditorDocument {
    path: PathBuf,
    buffer: Rope,
    cursor: Cursor,
    undo_stack: Vec<Edit>,
    redo_stack: Vec<Edit>,
    revision: u64,
    saved_revision: u64,
}

impl EditorDocument {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)?;

        Ok(Self::from_text(path, &content))
    }

    pub fn from_text(path: PathBuf, content: &str) -> Self {
        Self {
            path,
            buffer: Rope::from_str(content),
            cursor: Cursor::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            revision: 0,
            saved_revision: 0,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn line_count(&self) -> usize {
        self.buffer.len_lines()
    }

    pub fn char_count(&self) -> usize {
        self.buffer.len_chars()
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn is_modified(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn line(&self, index: usize) -> Option<String> {
        if index >= self.line_count() {
            return None;
        }

        let text = self.buffer.line(index).to_string();

        Some(text.trim_end_matches(['\r', '\n']).to_string())
    }

    pub fn set_cursor(&mut self, line: usize, column: usize) {
        self.cursor = self.clamp_cursor(Cursor { line, column });
    }

    pub fn move_left(&mut self) {
        let offset = self.cursor_offset();

        if offset > 0 {
            self.cursor = self.cursor_from_offset(offset - 1);
        }
    }

    pub fn move_right(&mut self) {
        let offset = self.cursor_offset();

        if offset < self.char_count() {
            self.cursor = self.cursor_from_offset(offset + 1);
        }
    }

    pub fn move_up(&mut self) {
        let line = self.cursor.line.saturating_sub(1);

        self.set_cursor(line, self.cursor.column);
    }

    pub fn move_down(&mut self) {
        let line = (self.cursor.line + 1).min(self.line_count().saturating_sub(1));

        self.set_cursor(line, self.cursor.column);
    }

    pub fn insert(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let offset = self.cursor_offset();
        self.replace_range(offset, offset, text);
    }

    pub fn backspace(&mut self) {
        let offset = self.cursor_offset();

        if offset > 0 {
            self.replace_range(offset - 1, offset, "");
        }
    }

    pub fn delete_forward(&mut self) {
        let offset = self.cursor_offset();

        if offset < self.char_count() {
            self.replace_range(offset, offset + 1, "");
        }
    }

    pub fn insert_newline(&mut self) {
        let offset = self.cursor_offset();

        let current_line = self
            .line(self.cursor.line)
            .unwrap_or_default();

        let indentation: String = current_line
            .chars()
            .take_while(|ch| *ch == ' ' || *ch == '\t')
            .collect();

        // default indent for now is 4 spaces
        let indent_unit = "    ";

        let first_line = self.buffer.line(0).to_string();

        let newline = if first_line.ends_with("\r\n") {
            "\r\n"
        } else {
            "\n"
        };

        let before = if offset > 0 {
            Some(self.buffer.char(offset - 1))
        } else {
            None
        };

        let after = if offset < self.char_count() {
            Some(self.buffer.char(offset))
        } else {
            None
        };

        // special case... pressing enter between brackets
        if matches!(
            (before, after),
            (Some('{'), Some('}'))
                | (Some('('), Some(')'))
                | (Some('['), Some(']'))
        ) {
            let insertion = format!(
                "{newline}{indentation}{indent_unit}{newline}{indentation}"
            );

            self.replace_range(offset, offset, &insertion);

            let cursor_offset = offset
                + newline.chars().count()
                + indentation.chars().count()
                + indent_unit.chars().count();

            self.cursor = self.cursor_from_offset(cursor_offset);

            if let Some(edit) = self.undo_stack.last_mut() {
                edit.cursor_after = self.cursor;
            }
        } else {
            let insertion = format!("{newline}{indentation}");

            self.insert(&insertion);
        }
    }

    pub fn undo(&mut self) {
        let Some(edit) = self.undo_stack.pop() else {
            return;
        };

        let end = edit.start + edit.inserted.chars().count();

        self.buffer.remove(edit.start..end);
        self.buffer.insert(edit.start, &edit.removed);

        self.cursor = edit.cursor_before;
        self.redo_stack.push(edit);
        self.revision += 1;
    }

    pub fn redo(&mut self) {
        let Some(edit) = self.redo_stack.pop() else {
            return;
        };

        let end = edit.start + edit.removed.chars().count();

        self.buffer.remove(edit.start..end);
        self.buffer.insert(edit.start, &edit.inserted);

        self.cursor = edit.cursor_after;
        self.undo_stack.push(edit);
        self.revision += 1;
    }

    pub fn save(&mut self) -> io::Result<()> {
        // Basic implementation. Atomic saving comes later.
        let file = fs::File::create(&self.path)?;
        let mut writer = io::BufWriter::new(file);

        self.buffer.write_to(&mut writer)?;

        use std::io::Write;
        writer.flush()?;

        self.saved_revision = self.revision;

        Ok(())
    }

    fn replace_range(&mut self, start: usize, end: usize, text: &str) {
        let removed = self.buffer.slice(start..end).to_string();
        let cursor_before = self.cursor;

        self.buffer.remove(start..end);
        self.buffer.insert(start, text);

        self.cursor = self.cursor_from_offset(start + text.chars().count());

        let edit = Edit {
            start,
            removed,
            inserted: text.to_string(),
            cursor_before,
            cursor_after: self.cursor,
        };

        self.undo_stack.push(edit);
        self.redo_stack.clear();
        self.revision += 1;
    }

    fn cursor_offset(&self) -> usize {
        let cursor = self.clamp_cursor(self.cursor);

        self.buffer.line_to_char(cursor.line) + cursor.column
    }

    fn cursor_from_offset(&self, offset: usize) -> Cursor {
        let offset = offset.min(self.char_count());
        let line = self.buffer.char_to_line(offset);

        let column = offset - self.buffer.line_to_char(line);

        self.clamp_cursor(Cursor { line, column })
    }

    fn clamp_cursor(&self, cursor: Cursor) -> Cursor {
        let line = cursor.line.min(self.line_count().saturating_sub(1));

        let line_start = self.buffer.line_to_char(line);
        let line_end = if line + 1 < self.line_count() {
            self.buffer.line_to_char(line + 1)
        } else {
            self.char_count()
        };

        let mut content_end = line_end;

        if content_end > line_start && self.buffer.char(content_end - 1) == '\n' {
            content_end -= 1;
        }

        if content_end > line_start && self.buffer.char(content_end - 1) == '\r' {
            content_end -= 1;
        }

        Cursor {
            line,
            column: cursor.column.min(content_end - line_start),
        }
    }

    pub fn type_character(&mut self, ch: char) {
        let offset = self.cursor_offset();

        //If the matching closing bracket is already present,
        //move over it instead of inserting another one..

        if matches!(ch, ')' | ']' | '}') && offset < self.char_count() && self.buffer.char(offset) == ch {
            self.cursor = self.cursor_from_offset(offset + 1);
            return;
        }

        let closing = match ch {
            '(' => Some(')'),
            '[' => Some(']'),
            '{' => Some('}'),
            _ => None,
        };

        if let Some(closing) = closing {
            let pair = format!("{ch}{closing}");

            // insert the pair as a single UNDOABLE operation
            self.replace_range(offset, offset, &pair);

            // place the cursor between brackets
            self.cursor = self.cursor_from_offset(offset + 1);

            //correct the cursor restored by redo
            if let Some(edit) = self.undo_stack.last_mut() {
                edit.cursor_after = self.cursor;
            }
        } else {
            self.insert(&ch.to_string());
        }
    }

    pub fn matching_bracket(
        &self,
        position: usize,
    ) -> Option<usize> {
        if position >= self.char_count() {
            return None;
        }

        let bracket = self.buffer.char(position);

        let (matching, forward) = match bracket {
            '(' => (')', true),
            '[' => (']', true),
            '{' => ('}', true),

            ')' => ('(', false),
            ']' => ('[', false),
            '}' => ('{', false),

            _ => return None,
        };

        let mut depth = 0usize;

        if forward {
            for index in position..self.char_count() {
                let ch = self.buffer.char(index);

                if ch == bracket {
                    depth += 1;
                } else if ch == matching {
                    depth -= 1;

                    if depth == 0 {
                        return Some(index);
                    }
                }
            }
        } else {
            for index in (0..=position).rev() {
                let ch = self.buffer.char(index);

                if ch == bracket {
                    depth += 1;
                } else if ch == matching {
                    depth -= 1;

                    if depth == 0 {
                        return Some(index);
                    }
                }
            }
        }

        None
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(text: &str) -> EditorDocument {
        EditorDocument::from_text(PathBuf::from("test.rs"), text)
    }

    #[test]
    fn insert_and_backspace() {
        let mut doc = document("Hello");

        doc.set_cursor(0, 5);
        doc.insert(" World");

        assert_eq!(doc.line(0).unwrap(), "Hello World");

        doc.backspace();

        assert_eq!(doc.line(0).unwrap(), "Hello Worl");
    }

    #[test]
    fn undo_redo() {
        let mut doc = document("fn main() {}");

        doc.set_cursor(0, 0);
        doc.insert("// ");

        assert_eq!(doc.line(0).unwrap(), "// fn main() {}");

        doc.undo();
        assert_eq!(doc.line(0).unwrap(), "fn main() {}");

        doc.redo();
        assert_eq!(doc.line(0).unwrap(), "// fn main() {}");
    }

    #[test]
    fn unicode_cursor() {
        let mut doc = document("Hello 🌍");

        // Cursor after the emoji.
        doc.set_cursor(0, 7);

        // Remove the emoji.
        doc.backspace();

        assert_eq!(doc.line(0).unwrap(), "Hello ");
        assert_eq!(doc.cursor(), Cursor { line: 0, column: 6 });
    }

    #[test]
    fn newline_preserves_indentation() {
        let mut doc = document("    let value = 10;");

        doc.set_cursor(0, 19);
        doc.insert_newline();

        assert_eq!(doc.line(1).unwrap(), "    ");
    }
}
