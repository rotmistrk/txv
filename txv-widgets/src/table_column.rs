//! Column — a single column definition for Table.

pub struct Column {
    pub(crate) title: String,
    pub(crate) width: u16,
}

impl Column {
    pub fn new(title: impl Into<String>, width: u16) -> Self {
        Self {
            title: title.into(),
            width,
        }
    }
}
