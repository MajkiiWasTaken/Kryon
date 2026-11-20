use std::ops::Range;

use gpui::{HighlightStyle, rgb};

const KEYWORD: u32 = 0xC4A0FF;
const STRING: u32 = 0xA4D9B2;
const COMMENT: u32 = 0x778397;
const NUMBER: u32 = 0xE4BA78;
const FUNCTION: u32 = 0x87AFFF;
const TYPE: u32 = 0xE5C07B;

pub fn highlight_rust(source: &str) -> Vec<(Range<usize>, HighlightStyle)> {
    let bytes = source.as_bytes();
    let mut result = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let start = index;

        if bytes[index..].starts_with(b"//") {
            add(&mut result, start..bytes.len(), COMMENT);
            break;
        }

        if bytes[index] == b'"' {
            index += 1;

            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                    continue;
                }

                if bytes[index] == b'"' {
                    index += 1;
                    break;
                }

                index += 1;
            }

            add(&mut result, start..index, STRING);
            continue;
        }

        if bytes[index].is_ascii_digit() {
            index += 1;

            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric()
                    || bytes[index] == b'_'
                    || bytes[index] == b'.')
            {
                index += 1;
            }

            add(&mut result, start..index, NUMBER);
            continue;
        }

        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            index += 1;

            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }

            let word = &source[start..index];

            let color = if is_keyword(word) {
                Some(KEYWORD)
            } else if word == "self" || word == "Self" || word == "true" || word == "false" {
                Some(KEYWORD)
            } else if bytes.get(index) == Some(&b'!') {
                Some(FUNCTION)
            } else if bytes.get(index) == Some(&b'(') {
                Some(FUNCTION)
            } else if word.starts_with(char::is_uppercase) {
                Some(TYPE)
            } else {
                None
            };

            if let Some(color) = color {
                add(&mut result, start..index, color);
            }

            continue;
        }

        // Advance over complete UTF-8 characters.
        let ch = source[index..]
            .chars()
            .next()
            .expect("Valid UTF-8 boundary");

        index += ch.len_utf8();
    }

    result
}

fn add(result: &mut Vec<(Range<usize>, HighlightStyle)>, range: Range<usize>, color: u32) {
    result.push((range, HighlightStyle::color(rgb(color).into())));
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "where"
            | "match"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "return"
            | "break"
            | "continue"
            | "async"
            | "await"
            | "move"
            | "const"
            | "static"
            | "type"
            | "as"
            | "in"
            | "ref"
            | "unsafe"
            | "extern"
            | "crate"
            | "super"
    )
}
