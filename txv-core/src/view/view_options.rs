//! ViewOptions — configuration flags for a View.

/// Options flags for a View.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ViewOptions {
    pub(crate) preprocess: bool,
    pub(crate) postprocess: bool,
    pub(crate) focusable: bool,
    pub(crate) modal: bool,
}

impl ViewOptions {
    pub fn preprocess(&self) -> bool {
        self.preprocess
    }
    pub fn postprocess(&self) -> bool {
        self.postprocess
    }
    pub fn focusable(&self) -> bool {
        self.focusable
    }
    pub fn modal(&self) -> bool {
        self.modal
    }

    pub fn with_focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    pub fn with_modal(mut self) -> Self {
        self.modal = true;
        self
    }

    pub fn with_preprocess(mut self) -> Self {
        self.preprocess = true;
        self
    }

    pub fn with_postprocess(mut self) -> Self {
        self.postprocess = true;
        self
    }

    pub fn with_modal_cond(mut self, active: bool) -> Self {
        self.modal = active;
        self
    }
}
