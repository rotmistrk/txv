//! VTE Perform trait implementation — dispatches terminal escape sequences.

use super::vte_handler::Performer;

impl vte::Perform for Performer<'_> {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => *self.cursor_x = 0,
            b'\x08' => {
                *self.cursor_x = self.cursor_x.saturating_sub(1);
            }
            b'\t' => {
                let next_tab = ((*self.cursor_x / 8) + 1) * 8;
                *self.cursor_x = next_tab.min(self.cols.saturating_sub(1));
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if let Some(&cmd) = params.first() {
            if cmd == b"0" || cmd == b"2" {
                if let Some(text) = params.get(1) {
                    *self.osc_title = Some(String::from_utf8_lossy(text).into_owned());
                }
            }
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
        let ps: Vec<u16> = params.iter().map(|p| p[0]).collect();
        let p1 = ps.first().copied().unwrap_or(0);

        match (action, intermediates) {
            ('A', []) => self.csi_cursor_up(p1),
            ('B', []) => self.csi_cursor_down(p1),
            ('C', []) => self.csi_cursor_forward(p1),
            ('D', []) => self.csi_cursor_back(p1),
            ('H' | 'f', []) => self.csi_cursor_position(p1, &ps),
            ('G', []) => self.csi_cursor_col(p1),
            ('J', []) => self.erase_display(p1),
            ('K', []) => self.erase_line(p1),
            ('L', []) => self.csi_insert_lines(p1),
            ('M', []) => self.csi_delete_lines(p1),
            ('S', []) => self.csi_scroll_up(p1),
            ('T', []) => self.csi_scroll_down(p1),
            ('m', []) => self.csi_sgr(&ps),
            ('r', []) => self.csi_set_scroll_region(p1, &ps),
            ('h', [b'?']) if p1 == 25 => *self.cursor_visible = true,
            ('l', [b'?']) if p1 == 25 => *self.cursor_visible = false,
            ('s', []) => *self.saved_cursor = (*self.cursor_x, *self.cursor_y),
            ('u', []) => {
                *self.cursor_x = self.saved_cursor.0;
                *self.cursor_y = self.saved_cursor.1;
            }
            ('P', []) => self.csi_delete_chars(p1),
            ('@', []) => self.csi_insert_chars(p1),
            ('c', []) => self.responses.push(b"\x1b[?1;2c".to_vec()),
            ('n', []) if p1 == 6 => {
                let reply = format!("\x1b[{};{}R", *self.cursor_y + 1, *self.cursor_x + 1);
                self.responses.push(reply.into_bytes());
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates) {
            (b'7', []) => *self.saved_cursor = (*self.cursor_x, *self.cursor_y),
            (b'8', []) => {
                *self.cursor_x = self.saved_cursor.0;
                *self.cursor_y = self.saved_cursor.1;
            }
            (b'D', []) => self.newline(),
            (b'M', []) => {
                if *self.cursor_y <= *self.scroll_top {
                    self.scroll_down();
                } else {
                    *self.cursor_y -= 1;
                }
            }
            (b'k', []) => *self.swallow_flag = true,
            _ => {}
        }
    }
}
