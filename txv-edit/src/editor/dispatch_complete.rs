//! Ex command Tab completion — used by InputLine's completer via delegate.

use super::Editor;

impl Editor {
    /// Reset completion state (call on any non-Tab keystroke in command mode).
    pub fn reset_ex_completion(&mut self) {
        self.ex_completer.reset();
    }
}
