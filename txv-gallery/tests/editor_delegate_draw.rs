//! Tests for EditorViewDelegate draw/view data providers.

use txv_core::prelude::*;
use txv_edit::editor::keymap::EditorMode;
use txv_edit::editor::Editor;
use txv_edit::view::delegate::{CursorRender, EditorViewDelegate, HighlightRange};
use txv_edit::view::EditorView;

/// Combined draw delegate for title, gutter sign, highlights, and cursor.
struct DrawDelegate;

impl EditorViewDelegate for DrawDelegate {
    fn title(&self, _editor: &Editor) -> Option<&str> {
        Some("Custom Title")
    }

    fn gutter_sign(&self, line: usize) -> Option<(char, Style)> {
        if line == 1 {
            Some(('!', Style::default()))
        } else {
            None
        }
    }

    fn extra_gutter_width(&self) -> u16 {
        1
    }

    fn highlight_ranges(&self, line: usize) -> &[HighlightRange] {
        static RANGES: std::sync::LazyLock<Vec<HighlightRange>> = std::sync::LazyLock::new(|| {
            vec![HighlightRange::new(
                0,
                3,
                Style::new(Color::Reset, Color::Rgb(255, 255, 0)),
            )]
        });
        if line == 0 {
            &RANGES
        } else {
            &[]
        }
    }

    fn cursor_render(&self, _mode: EditorMode) -> CursorRender {
        CursorRender::Software(Style::new(Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255)))
    }
}

#[test]
fn delegate_title_override() {
    let ev = EditorView::with_delegate(DrawDelegate);
    assert_eq!(ev.title(), "Custom Title");
}

#[test]
fn delegate_gutter_sign() {
    let mut ev = EditorView::with_delegate(DrawDelegate);
    ev.set_content("line1\nline2\nline3", "");
    ev.editor_mut().options_mut().set_number(false);
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    let ch = ev.buffer().cell(0, 1).ch();
    assert_eq!(ch, '!', "gutter sign at line 2");
}

#[test]
fn delegate_highlight_ranges() {
    let mut ev = EditorView::with_delegate(DrawDelegate);
    ev.set_content("hello world", "");
    ev.editor_mut().options_mut().set_number(false);
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    // Col 1 (not cursor — cursor has software style override)
    let cell = ev.buffer().cell(2, 0);
    assert_eq!(cell.style().bg(), Color::Rgb(255, 255, 0), "highlight bg at col 2");
}

#[test]
fn delegate_software_cursor_no_hardware() {
    let mut ev = EditorView::with_delegate(DrawDelegate);
    ev.set_content("hello", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    assert!(ev.cursor().is_none(), "no hardware cursor");
}

#[test]
fn delegate_software_cursor_style() {
    let mut ev = EditorView::with_delegate(DrawDelegate);
    ev.set_content("hello", "");
    ev.editor_mut().options_mut().set_number(false);
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    // Cursor at (0,0) — gutter_width=1 (extra), so cursor cell is at buffer x=1
    let cell = ev.buffer().cell(1, 0);
    assert_eq!(cell.style().fg(), Color::Rgb(0, 0, 0), "software cursor fg");
    assert_eq!(cell.style().bg(), Color::Rgb(255, 255, 255), "software cursor bg");
}
