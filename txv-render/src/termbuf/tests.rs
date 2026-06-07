use super::*;
use txv_core::cell::Color;

#[test]
fn basic_print() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"Hello");
    assert_eq!(tb.cells[0].cells[0].ch, 'H');
    assert_eq!(tb.cells[0].cells[4].ch, 'o');
    assert_eq!(tb.cursor(), (5, 0));
}

#[test]
fn newline_and_cr() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"A\r\nB");
    assert_eq!(tb.cells[0].cells[0].ch, 'A');
    assert_eq!(tb.cells[1].cells[0].ch, 'B');
}

#[test]
fn cursor_movement() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"\x1b[5;10H");
    assert_eq!(tb.cursor(), (9, 4));
}

#[test]
fn erase_line() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"ABCDEF\x1b[4G\x1b[K");
    assert_eq!(tb.cells[0].cells[0].ch, 'A');
    assert_eq!(tb.cells[0].cells[1].ch, 'B');
    assert_eq!(tb.cells[0].cells[2].ch, 'C');
    assert_eq!(tb.cells[0].cells[3].ch, ' ');
}

#[test]
fn sgr_colors() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"\x1b[31mR\x1b[0m");
    assert_eq!(tb.cells[0].cells[0].ch, 'R');
    assert_eq!(tb.cells[0].cells[0].style.fg(), Color::Ansi(1));
}

#[test]
fn scroll_on_overflow() {
    let mut tb = TermBuf::new(80, 3);
    tb.process(b"A\r\nB\r\nC\r\nD");
    assert_eq!(tb.cells[0].cells[0].ch, 'B');
    assert_eq!(tb.cells[1].cells[0].ch, 'C');
    assert_eq!(tb.cells[2].cells[0].ch, 'D');
}

#[test]
fn render_to_buffer() {
    let mut tb = TermBuf::new(10, 5);
    tb.process(b"Hi");
    let mut buf = Buffer::new(10, 5);
    tb.render_to(&mut buf);
    assert_eq!(buf.cell(0, 0).ch(), 'H');
    assert_eq!(buf.cell(1, 0).ch(), 'i');
}

#[test]
fn resize_preserves_content() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"Hello");
    tb.resize(40, 12);
    assert_eq!(tb.cells[0].cells[0].ch, 'H');
    assert_eq!(tb.cols, 40);
    assert_eq!(tb.rows, 12);
}

#[test]
fn cursor_visibility() {
    let mut tb = TermBuf::new(80, 24);
    assert!(tb.cursor_visible());
    tb.process(b"\x1b[?25l");
    assert!(!tb.cursor_visible());
    tb.process(b"\x1b[?25h");
    assert!(tb.cursor_visible());
}

#[test]
fn swallow_esc_k_title_sequence() {
    let mut tb = TermBuf::new(80, 24);
    // ESC k sets tmux title — content should NOT appear on screen
    tb.process(b"A\x1bktitle text\x1b\\B");
    assert_eq!(tb.cells[0].cells[0].ch, 'A');
    assert_eq!(tb.cells[0].cells[1].ch, 'B');
    assert_eq!(tb.cursor(), (2, 0));
}

#[test]
fn swallow_esc_k_terminated_by_bel() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"X\x1bkmy title\x07Y");
    assert_eq!(tb.cells[0].cells[0].ch, 'X');
    assert_eq!(tb.cells[0].cells[1].ch, 'Y');
}

#[test]
fn swallow_real_prompt_esc_k() {
    let mut tb = TermBuf::new(80, 24);
    // Real prompt: ESC[32m ESC_k kairn/f4 ESC\ core ESC[34m : ...
    tb.process(b"\x1b[00;32m\x1bkkairn/f4\x1b\\core\x1b[01;34m:");
    // Should see "core:" without "kairn/f4"
    let row: String = tb.cells[0].cells.iter().take(10).map(|c| c.ch).collect();
    assert_eq!(row.trim(), "core:");
}

#[test]
fn rprompt_no_spill_with_tmux_sequences() {
    let mut tb = TermBuf::new(60, 24);

    // precmd: window title (OSC 0)
    tb.process(b"\x1b]0;user@host [OK]~/kairn/f4>\x07");
    // precmd: mux title (ESC k ... ESC \)
    tb.process(b"\x1bkkairn/f4\x1b\\");
    // precmd: pane title (OSC 2 ... ST)
    tb.process(b"\x1b]2;OK kairn/f4>\x1b\\");

    // Cursor should not have moved
    assert_eq!(tb.cursor(), (0, 0));

    // PS1 rendering: zero-width (SGR + tmux title) then visible text
    tb.process(b"\r\x1b[00;32m\x1bkkairn/f4\x1b\\");
    assert_eq!(tb.cursor(), (0, 0), "zero-width sequences should not move cursor");

    // Visible prompt: "core:OK ~/k/f4> " = 16 chars
    tb.process(b"core\x1b[01;34m:\x1b[00;32mOK\x1b[01;34m ~/k/f4>\x1b[0m\x1b[1m ");
    assert_eq!(tb.cursor().0, 16);

    // RPROMPT at column 54 (60 - 6): save, move, print, restore
    tb.process(b"\x1b7\x1b[55G");
    assert_eq!(tb.cursor().0, 54);
    tb.process(b"master");
    // Should end at column 60 (= cols), no wrap
    assert_eq!(tb.cursor(), (60, 0));
    // Row 1 should be empty
    let row1: String = tb.cells[1].cells.iter().take(10).map(|c| c.ch).collect();
    assert_eq!(row1.trim(), "");
}

#[test]
fn rprompt_spills_when_columns_exceeds_actual_width() {
    // Simulates: shell thinks COLUMNS=80 (initial PTY size)
    // but terminal was resized to 60 columns.
    // Shell positions RPROMPT at col 74 (80-6), but terminal only has 60 cols.
    let mut tb = TermBuf::new(60, 24);

    // Shell renders prompt thinking COLUMNS=80
    tb.process(b"\rcore:OK ~/k/f4> ");
    assert_eq!(tb.cursor().0, 16);

    // RPROMPT at column 74 (80 - 6): save, move to col 75 (1-indexed)
    tb.process(b"\x1b7\x1b[75G");
    // TermBuf clamps to cols-1 = 59
    assert_eq!(tb.cursor().0, 59);
    tb.process(b"master");
    // "m" at col 59, then wraps: "aster" on next line
    // This demonstrates the bug!
    let row0: String = tb.cells[0].cells.iter().map(|c| c.ch).collect();
    let row1: String = tb.cells[1].cells.iter().take(10).map(|c| c.ch).collect();
    assert!(row0.ends_with('m'), "Only 'm' should fit: {:?}", row0.trim_end());
    assert!(row1.starts_with("aster"), "Rest spills: {:?}", row1.trim_end());
}

#[test]
fn scrollback_captures_lines_on_scroll() {
    let mut tb = TermBuf::new(80, 3);
    tb.process(b"A\r\nB\r\nC\r\nD");
    // Line "A" was pushed off the top
    assert_eq!(tb.scrollback_len(), 1);
    let line = tb.scrollback_line(0).map(|l| l[0].ch);
    assert_eq!(line, Some('A'));
}

#[test]
fn scrollback_accumulates_multiple_lines() {
    let mut tb = TermBuf::new(80, 3);
    tb.process(b"1\r\n2\r\n3\r\n4\r\n5\r\n6");
    // Lines 1, 2, 3 were pushed off (3 scrolls happened)
    assert_eq!(tb.scrollback_len(), 3);
    assert_eq!(tb.scrollback_line(0).map(|l| l[0].ch), Some('3'));
    assert_eq!(tb.scrollback_line(2).map(|l| l[0].ch), Some('1'));
}

#[test]
fn scrollback_respects_limit() {
    let mut tb = TermBuf::with_scrollback(80, 3, 2);
    tb.process(b"A\r\nB\r\nC\r\nD\r\nE\r\nF");
    // Only 2 lines kept (limit=2)
    assert_eq!(tb.scrollback_len(), 2);
    assert_eq!(tb.scrollback_line(0).map(|l| l[0].ch), Some('C'));
    assert_eq!(tb.scrollback_line(1).map(|l| l[0].ch), Some('B'));
}

#[test]
fn resize_shrink_preserves_scrollback() {
    let mut tb = TermBuf::new(80, 10);
    // Fill 8 lines with content, cursor on line 7
    for i in 0..8 {
        let line = format!("line {i}\r\n");
        tb.process(line.as_bytes());
    }
    // Cursor should be on line 8 (after 8 newlines)
    assert_eq!(tb.cursor().1, 8);

    // Shrink to 5 rows — lines above cursor should go to scrollback
    tb.resize(80, 5);

    // Cursor should still be valid (within new height)
    assert!(tb.cursor().1 < 5, "cursor_y={} should be < 5", tb.cursor().1);

    // Lines should have been pushed to scrollback
    assert!(tb.scrollback_len() > 0, "Expected lines in scrollback after shrink");
}

#[test]
fn resize_grow_keeps_content() {
    let mut tb = TermBuf::new(80, 5);
    tb.process(b"line0\r\nline1\r\nline2");
    let cy = tb.cursor().1;

    // Grow to 10 rows
    tb.resize(80, 10);

    // Cursor position unchanged
    assert_eq!(tb.cursor().1, cy);
    // Content preserved
    assert_eq!(tb.cells[0].cells[0].ch, 'l');
    assert_eq!(tb.cells[0].cells[4].ch, '0');
}

#[test]
fn resize_shrink_cols_preserves_content() {
    let mut tb = TermBuf::new(80, 10);
    tb.process(b"hello world");
    tb.resize(5, 10);
    // First 5 chars preserved
    assert_eq!(tb.cells[0].cells[0].ch, 'h');
    assert_eq!(tb.cells[0].cells[4].ch, 'o');
}

#[test]
fn reflow_wrap_on_shrink() {
    let mut tb = TermBuf::new(20, 5);
    tb.process(b"abcdefghij1234567890"); // 20 chars, fills row 0 exactly
    tb.process(b"ABCDE"); // 5 more on row 1
                          // Row 0 is full (wrapped=true because cursor hit col 20)
                          // Row 1 has "ABCDE"
    assert_eq!(tb.cells[0].cells[0].ch, 'a');
    assert_eq!(tb.cells[1].cells[0].ch, 'A');

    // Shrink to 10 cols: "abcdefghij1234567890ABCDE" reflows to 3 rows of 10
    tb.resize(10, 5);
    let r0: String = tb.cells[0].cells.iter().take(10).map(|c| c.ch).collect();
    let r1: String = tb.cells[1].cells.iter().take(10).map(|c| c.ch).collect();
    let r2: String = tb.cells[2].cells.iter().take(10).map(|c| c.ch).collect();
    assert_eq!(r0.trim(), "abcdefghij");
    assert_eq!(r1.trim(), "1234567890");
    assert_eq!(r2.trim(), "ABCDE");
}

#[test]
fn reflow_unwrap_on_grow() {
    let mut tb = TermBuf::new(10, 5);
    // Write 20 chars — wraps to 2 rows of 10
    tb.process(b"abcdefghij1234567890");
    assert_eq!(tb.cells[0].cells[0].ch, 'a');
    assert_eq!(tb.cells[1].cells[0].ch, '1');

    // Grow to 20 cols: should unwrap back to 1 row
    tb.resize(20, 5);
    let r0: String = tb.cells[0].cells.iter().take(20).map(|c| c.ch).collect();
    assert_eq!(r0.trim(), "abcdefghij1234567890");
    // Row 1 should be blank
    assert!(tb.cells[1].is_blank());
}

#[test]
fn reflow_preserves_hard_newlines() {
    let mut tb = TermBuf::new(20, 5);
    tb.process(b"hello\r\nworld");
    // Two separate logical lines
    assert_eq!(tb.cells[0].cells[0].ch, 'h');
    assert_eq!(tb.cells[1].cells[0].ch, 'w');

    // Shrink to 3 cols: "hello" → "hel"+"lo", "world" → "wor"+"ld"
    tb.resize(3, 10);
    let r0: String = tb.cells[0].cells.iter().take(3).map(|c| c.ch).collect();
    let r1: String = tb.cells[1].cells.iter().take(3).map(|c| c.ch).collect();
    let r2: String = tb.cells[2].cells.iter().take(3).map(|c| c.ch).collect();
    let r3: String = tb.cells[3].cells.iter().take(3).map(|c| c.ch).collect();
    assert_eq!(r0.trim(), "hel");
    assert_eq!(r1.trim(), "lo");
    assert_eq!(r2.trim(), "wor");
    assert_eq!(r3.trim(), "ld");

    // Grow back to 20: should restore original 2 lines
    tb.resize(20, 5);
    let r0: String = tb.cells[0].cells.iter().take(5).map(|c| c.ch).collect();
    let r1: String = tb.cells[1].cells.iter().take(5).map(|c| c.ch).collect();
    assert_eq!(r0.trim(), "hello");
    assert_eq!(r1.trim(), "world");
}
