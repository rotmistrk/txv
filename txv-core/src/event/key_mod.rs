//! Key modifiers.

/// Key modifiers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct KeyMod {
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) shift: bool,
}

impl KeyMod {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const SHIFT: Self = Self {
        ctrl: false,
        alt: false,
        shift: true,
    };

    pub fn ctrl(self) -> bool {
        self.ctrl
    }

    pub fn alt(self) -> bool {
        self.alt
    }

    pub fn shift(self) -> bool {
        self.shift
    }

    pub const fn with_ctrl(self) -> Self {
        Self {
            ctrl: true,
            alt: self.alt,
            shift: self.shift,
        }
    }

    pub const fn with_alt(self) -> Self {
        Self {
            ctrl: self.ctrl,
            alt: true,
            shift: self.shift,
        }
    }

    pub const fn with_shift(self) -> Self {
        Self {
            ctrl: self.ctrl,
            alt: self.alt,
            shift: true,
        }
    }
}
