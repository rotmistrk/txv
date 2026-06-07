//! Test helper completer for modal_key_tests.

use std::error::Error;

use txv_core::prelude::*;

use crate::modal_key_test_completion::HelpCompletion;

pub(crate) struct TestCompleter;

impl Completer for TestCompleter {
    fn complete(&self, input: &str, _cursor: usize, visitor: &mut CompletionVisitor<'_>) -> Result<(), Box<dyn Error>> {
        if input == "he" {
            visitor(&HelpCompletion)?;
        }
        Ok(())
    }
}
