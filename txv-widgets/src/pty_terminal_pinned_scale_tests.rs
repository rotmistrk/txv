//! Pinned scrollback at realistic scale: thousands of lines, a full
//! scrollback ring, wrapped lines. Assertions check rendered screen text,
//! not internal counters.
//!
//! Scenario (user requirement): a 51-row terminal shows lines 5000..5050.
//! After PgUp the top area shows older history and the bottom area shows
//! the live tail. When 100 more lines arrive, the top area must not move
//! and the bottom area must show the new tail.

use txv_core::prelude::*;
use txv_render::termbuf::TermBuf;

use crate::pty_terminal::PtyTerminal;

const COLS: u16 = 100;
const ROWS: u16 = 51;
const CURSOR_AREA: u16 = 5;
/// Rows above the separator: ROWS - CURSOR_AREA - 1.
const TOP_AREA: u16 = ROWS - CURSOR_AREA - 1;
/// kairn's default scrollback for shell and kiro tabs.
const SCROLLBACK: usize = 2000;

fn make_terminal(scrollback: usize) -> PtyTerminal {
    let mut term = PtyTerminal {
        state: ViewState::default(),
        termbuf: TermBuf::with_scrollback(COLS, ROWS, scrollback),
        session: None,
        base_title: "test".to_string(),
        title: "test".to_string(),
        osc_suffix: String::new(),
        prev_cols: COLS,
        prev_rows: ROWS,
        exited: true,
        scroll_offset: 0,
        had_output: false,
        pinned_mode: false,
        pinned_bottom_hist: 0,
        cursor_area_lines: CURSOR_AREA,
    };
    term.state.set_bounds(Rect::new(0, 0, COLS, ROWS));
    term
}

/// Deterministic pseudo-random generator (no external crates, no flakiness).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

const WORDS: &[&str] = &[
    "Compiling",
    "warning:",
    "unused",
    "variable",
    "src/handler_open.rs",
    "-->",
    "fn",
    "process_event(&mut",
    "self)",
    "ok",
    "error[E0308]:",
    "mismatched",
    "types",
    "expected",
    "`usize`,",
    "found",
    "`u16`",
    "tool_call",
    "edit_buffer",
    "kiro>",
    "Reading",
    "file",
    "lines",
    "1..240",
    "test",
    "result:",
    "passed;",
];

/// One line of realistic output: a unique tag, then a variable number of
/// words. About 10% of lines are long enough to wrap; some are empty.
fn output_line(index: usize, rng: &mut Lcg) -> String {
    let roll = rng.next() % 100;
    if roll < 5 {
        return String::new();
    }
    let word_count = if roll < 15 {
        20 + rng.next() % 25
    } else {
        1 + rng.next() % 12
    };
    let mut line = format!("L{index:05}");
    for _ in 0..word_count {
        line.push(' ');
        line.push_str(WORDS[(rng.next() as usize) % WORDS.len()]);
    }
    line
}

/// Feed lines `from..to` as one PTY output burst per 50 lines.
fn feed(term: &mut PtyTerminal, from: usize, to: usize, rng: &mut Lcg) {
    let mut chunk = String::new();
    for i in from..to {
        chunk.push_str(&output_line(i, rng));
        chunk.push_str("\r\n");
        if (i + 1) % 50 == 0 {
            term.termbuf.process(chunk.as_bytes());
            chunk.clear();
        }
    }
    term.termbuf.process(chunk.as_bytes());
}

fn row_text(term: &PtyTerminal, y: u16) -> String {
    (0..COLS)
        .map(|x| term.state.buffer().cell(x, y).ch())
        .collect::<String>()
        .trim_end()
        .to_string()
}

fn rows_text(term: &PtyTerminal, from: u16, to: u16) -> Vec<String> {
    (from..to).map(|y| row_text(term, y)).collect()
}

fn top_area(term: &mut PtyTerminal) -> Vec<String> {
    term.draw();
    rows_text(term, 0, TOP_AREA)
}

fn cursor_area(term: &mut PtyTerminal) -> Vec<String> {
    term.draw();
    rows_text(term, ROWS - CURSOR_AREA, ROWS)
}

/// Tags of rows that start a logical line ("L01234 ..."), in screen order.
fn tags(rows: &[String]) -> Vec<String> {
    rows.iter()
        .filter(|r| r.starts_with('L') && r.len() >= 6)
        .map(|r| r[..6].to_string())
        .collect()
}

#[test]
fn top_area_stays_put_when_output_arrives_with_full_scrollback() {
    let mut term = make_terminal(SCROLLBACK);
    let mut rng = Lcg(42);
    feed(&mut term, 0, 5000, &mut rng);
    assert_eq!(term.termbuf.scrollback_len(), SCROLLBACK, "precondition: ring is full");

    term.scroll_up_page();
    let before = top_area(&mut term);
    assert!(!tags(&before).is_empty(), "top area shows history: {before:?}");

    feed(&mut term, 5000, 5100, &mut rng);
    let after = top_area(&mut term);

    assert_eq!(after, before, "top area must not move when new output arrives");
}

#[test]
fn cursor_area_follows_new_output_while_pinned() {
    let mut term = make_terminal(SCROLLBACK);
    let mut rng = Lcg(7);
    feed(&mut term, 0, 5000, &mut rng);
    term.scroll_up_page();

    feed(&mut term, 5000, 5100, &mut rng);
    let live = cursor_area(&mut term);
    let live_tags = tags(&live);

    assert!(
        live_tags.iter().any(|t| t.as_str() >= "L05090"),
        "cursor area shows the newest lines, got {live:?}"
    );
    assert!(
        live_tags.iter().all(|t| t.as_str() >= "L05080"),
        "cursor area holds only the tail, got {live:?}"
    );
}

#[test]
fn top_area_stays_put_below_scrollback_limit() {
    let mut term = make_terminal(SCROLLBACK);
    let mut rng = Lcg(3);
    feed(&mut term, 0, 800, &mut rng);
    assert!(
        term.termbuf.scrollback_len() < SCROLLBACK,
        "precondition: ring not full"
    );

    term.scroll_up_page();
    let before = top_area(&mut term);
    feed(&mut term, 800, 900, &mut rng);

    assert_eq!(top_area(&mut term), before);
}

#[test]
fn several_page_ups_then_output_keeps_older_view() {
    let mut term = make_terminal(SCROLLBACK);
    let mut rng = Lcg(11);
    feed(&mut term, 0, 6000, &mut rng);
    for _ in 0..5 {
        term.scroll_up_page();
    }
    let before = top_area(&mut term);

    for start in (6000..6300).step_by(30) {
        feed(&mut term, start, start + 30, &mut rng);
        assert_eq!(top_area(&mut term), before, "top area moved after line {start}");
    }
}

#[test]
fn pinned_history_evicted_shows_oldest_kept_lines() {
    let mut term = make_terminal(SCROLLBACK);
    let mut rng = Lcg(19);
    feed(&mut term, 0, 3000, &mut rng);
    term.scroll_up_page();

    // More output than the whole ring: the pinned rows are gone.
    feed(&mut term, 3000, 3000 + SCROLLBACK + 500, &mut rng);
    let top = top_area(&mut term);

    // Expected: the oldest TOP_AREA rows still kept in scrollback.
    let sb_len = term.termbuf.scrollback_len();
    let oldest: Vec<String> = (0..TOP_AREA as usize)
        .map(|i| {
            term.termbuf
                .scrollback_line(sb_len - 1 - i)
                .map(|cells| cells.iter().map(|c| c.ch()).collect::<String>())
                .unwrap_or_default()
                .trim_end()
                .to_string()
        })
        .collect();
    assert!(!tags(&top).is_empty(), "top area is not blank: {top:?}");
    assert_eq!(top, oldest, "view clamps to the oldest rows still in scrollback");
    assert!(term.pinned_mode, "still pinned after eviction");
}

#[test]
fn cursor_area_shows_prompt_after_clear_screen() {
    let mut term = make_terminal(SCROLLBACK);
    let mut rng = Lcg(23);
    feed(&mut term, 0, 3000, &mut rng);
    // `clear`, then a prompt at the top of the screen.
    term.termbuf.process(b"\x1b[H\x1b[2J$ cargo test");
    let (_, cursor_y) = term.termbuf.cursor();
    assert_eq!(cursor_y, 0, "precondition: cursor at top after clear");

    term.scroll_up_page();
    let live = cursor_area(&mut term);

    assert!(
        live.iter().any(|r| r.contains("$ cargo test")),
        "cursor area must show the prompt line, got {live:?}"
    );
}
