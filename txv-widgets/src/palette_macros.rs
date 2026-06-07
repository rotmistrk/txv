//! Macro for delegating palette resolution to a stored `Option<Arc<dyn Palette>>` field.

/// Generates `set_palette` and `resolve_style` methods that delegate to a palette field.
#[macro_export]
macro_rules! delegate_palette {
    ($field:ident) => {
        fn resolve_style(&self, id: txv_core::prelude::StyleId) -> txv_core::prelude::Style {
            match &self.$field {
                Some(p) => p.style(id),
                None => txv_core::prelude::palette().style(id),
            }
        }
    };
}
