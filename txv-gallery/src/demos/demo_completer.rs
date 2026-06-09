//! Demo completer for InputLine — completes widget names.

use txv_core::prelude::*;

use crate::widget_list::WIDGET_NAMES;

pub(crate) struct DemoCompleter;

struct DemoCompletion {
    text: String,
}

impl Completion for DemoCompletion {
    fn text(&self) -> &str {
        &self.text
    }
    fn display(&self) -> &str {
        &self.text
    }
    fn kind(&self) -> &str {
        "widget"
    }
}

impl Completer for DemoCompleter {
    fn complete(
        &self,
        input: &str,
        cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let prefix = &input[..cursor];
        for name in WIDGET_NAMES {
            if name.to_lowercase().starts_with(&prefix.to_lowercase()) && !prefix.is_empty() {
                let c = DemoCompletion { text: name.to_string() };
                if !visitor(&c)? {
                    break;
                }
            }
        }
        Ok(())
    }
}
