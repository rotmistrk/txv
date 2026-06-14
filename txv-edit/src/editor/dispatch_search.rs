//! Editor dispatch: search and command-mode commands.

use super::command::Command;
use super::keymap::EditorMode;
use super::{Editor, EditorAction};

impl Editor {
    pub(super) fn dispatch_search_and_command(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::EnterSearchMode => {
                self.enter_search_forward();
                EditorAction::ModeChanged
            }
            Command::EnterSearchBackward => {
                self.enter_search_backward();
                EditorAction::ModeChanged
            }
            Command::SearchForward(ref pat) => {
                self.search_forward(pat);
                EditorAction::CursorMoved
            }
            Command::SearchBackward(ref pat) => {
                self.search_backward(pat);
                EditorAction::CursorMoved
            }
            Command::SearchNext => {
                self.search_next();
                EditorAction::CursorMoved
            }
            Command::SearchPrev => {
                self.search_prev();
                EditorAction::CursorMoved
            }
            Command::SearchWordForward => {
                self.search_word(true);
                EditorAction::CursorMoved
            }
            Command::SearchWordBackward => {
                self.search_word(false);
                EditorAction::CursorMoved
            }
            Command::EnterCommandMode => {
                self.enter_command_mode();
                EditorAction::ModeChanged
            }
            Command::CompletionNext | Command::CompletionPrev => EditorAction::LspCompletion,
            _ => EditorAction::None,
        }
    }

    fn enter_search_forward(&mut self) {
        self.incsearch_origin = Some((self.cursor_line, self.cursor_col));
        self.mode = EditorMode::Search;
    }

    fn enter_search_backward(&mut self) {
        self.incsearch_origin = Some((self.cursor_line, self.cursor_col));
        self.search_direction_forward = false;
        self.mode = EditorMode::Search;
    }

    fn enter_command_mode(&mut self) {
        self.mode = EditorMode::Command;
    }
}
