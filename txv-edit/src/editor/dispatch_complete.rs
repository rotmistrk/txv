//! Ex command Tab completion wiring on the Editor.

use super::Editor;

impl Editor {
    /// Perform Tab completion on the command buffer.
    /// `extra_commands` are app-specific command names.
    /// Returns true if the buffer was modified.
    pub fn complete_ex(&mut self, extra_commands: &[&str]) -> bool {
        let buf = self.command_buf.clone();
        let Some(completed) = self.ex_completer.complete(&buf, extra_commands) else {
            return false;
        };
        let completed = completed.to_string();
        let result = self.ex_completer.build_result(&buf, &completed);
        self.command_buf = result;
        true
    }

    /// Reset completion state (call on any non-Tab keystroke in command mode).
    pub fn reset_ex_completion(&mut self) {
        self.ex_completer.reset();
    }
}
