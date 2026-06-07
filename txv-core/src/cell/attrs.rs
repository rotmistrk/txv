//! Attrs — text attribute flags.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Attrs {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) dim: bool,
}

impl Attrs {
    pub fn bold_val(&self) -> bool {
        self.bold
    }
    pub fn italic_val(&self) -> bool {
        self.italic
    }
    pub fn underline_val(&self) -> bool {
        self.underline
    }
    pub fn dim_val(&self) -> bool {
        self.dim
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    pub fn set_bold(&mut self, v: bool) {
        self.bold = v;
    }
    pub fn set_italic(&mut self, v: bool) {
        self.italic = v;
    }
    pub fn set_underline(&mut self, v: bool) {
        self.underline = v;
    }
    pub fn set_dim(&mut self, v: bool) {
        self.dim = v;
    }
}
