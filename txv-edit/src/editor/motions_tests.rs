#![cfg(test)]

use super::*;

#[test]
fn test_word_forward() {
    let buf = PieceTable::from_text("hello world foo");
    assert_eq!(word_forward(&buf, 0, 0), (0, 6));
    assert_eq!(word_forward(&buf, 0, 6), (0, 12));
}

#[test]
fn test_word_backward() {
    let buf = PieceTable::from_text("hello world");
    assert_eq!(word_backward(&buf, 0, 6), (0, 0));
}

#[test]
fn test_word_end() {
    let buf = PieceTable::from_text("hello world");
    assert_eq!(word_end(&buf, 0, 0), (0, 4));
}

#[test]
fn test_first_non_blank() {
    let buf = PieceTable::from_text("    hello");
    assert_eq!(first_non_blank(&buf, 0), 4);
}

#[test]
fn test_find_char() {
    let buf = PieceTable::from_text("hello world");
    assert_eq!(find_char(&buf, 0, 0, 'o'), Some(4));
    assert_eq!(find_char_back(&buf, 0, 7, 'o'), Some(4));
}

#[test]
fn test_match_bracket() {
    let buf = PieceTable::from_text("(hello)");
    assert_eq!(match_bracket(&buf, 0, 0), Some((0, 6)));
    assert_eq!(match_bracket(&buf, 0, 6), Some((0, 0)));
}

#[test]
fn test_match_bracket_offset_past_end() {
    // Cursor col beyond content length should return None, not panic
    let buf = PieceTable::from_text("(");
    assert_eq!(match_bracket(&buf, 0, 0), None); // no matching close
    assert_eq!(match_bracket(&buf, 0, 5), None); // col past end
    assert_eq!(match_bracket(&buf, 99, 0), None); // line past end
}

#[test]
fn test_match_curly_braces() {
    let buf = PieceTable::from_text("fn main() {\n    hello\n}");
    // { at (0, 10) should match } at (2, 0)
    assert_eq!(match_bracket(&buf, 0, 10), Some((2, 0)));
    // } at (2, 0) should match { at (0, 10)
    assert_eq!(match_bracket(&buf, 2, 0), Some((0, 10)));
}

#[test]
fn test_match_open_brace_multiline() {
    // Simulates a real file with fn opening brace
    let text = "    pub fn foo() {\n        bar();\n    }";
    let buf = PieceTable::from_text(text);
    // { is at end of line 0
    let line0 = buf.line(0).unwrap_or_default();
    let open_col = line0.chars().count() - 1; // last char is {
    assert_eq!(line0.chars().last(), Some('{'));
    let result = match_bracket(&buf, 0, open_col);
    assert!(result.is_some(), "should find matching }} for {{ at (0, {open_col})");
    // } is at line 2, col 4
    let close_result = match_bracket(&buf, 2, 4);
    assert_eq!(close_result, Some((0, open_col)));
}
