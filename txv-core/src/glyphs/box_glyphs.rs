//! Box-drawing characters.

/// Box-drawing characters (light and heavy variants).
#[derive(Clone, Debug)]
pub struct BoxGlyphs {
    pub(crate) h: char,
    pub(crate) v: char,
    pub(crate) tl: char,
    pub(crate) tr: char,
    pub(crate) bl: char,
    pub(crate) br: char,
    pub(crate) h_heavy: char,
    pub(crate) v_heavy: char,
    pub(crate) tl_heavy: char,
    pub(crate) tr_heavy: char,
    pub(crate) bl_heavy: char,
    pub(crate) br_heavy: char,
    pub(crate) tl_round: char,
    pub(crate) tr_round: char,
    pub(crate) bl_round: char,
    pub(crate) br_round: char,
}

impl BoxGlyphs {
    pub fn h(&self) -> char {
        self.h
    }
    pub fn v(&self) -> char {
        self.v
    }
    pub fn tl(&self) -> char {
        self.tl
    }
    pub fn tr(&self) -> char {
        self.tr
    }
    pub fn bl(&self) -> char {
        self.bl
    }
    pub fn br(&self) -> char {
        self.br
    }
    pub fn h_heavy(&self) -> char {
        self.h_heavy
    }
    pub fn v_heavy(&self) -> char {
        self.v_heavy
    }
    pub fn tl_heavy(&self) -> char {
        self.tl_heavy
    }
    pub fn tr_heavy(&self) -> char {
        self.tr_heavy
    }
    pub fn bl_heavy(&self) -> char {
        self.bl_heavy
    }
    pub fn br_heavy(&self) -> char {
        self.br_heavy
    }
    pub fn tl_round(&self) -> char {
        self.tl_round
    }
    pub fn tr_round(&self) -> char {
        self.tr_round
    }
    pub fn bl_round(&self) -> char {
        self.bl_round
    }
    pub fn br_round(&self) -> char {
        self.br_round
    }
}
