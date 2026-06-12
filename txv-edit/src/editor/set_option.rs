//! Editor `:set` option handling.

use crate::settings::CursorStyle;

use super::Editor;

impl Editor {
    pub(super) fn apply_set_option(&mut self, opt: &str) {
        if let Some(()) = self.apply_bool_option(opt) {
            return;
        }
        self.apply_value_option(opt);
    }

    fn apply_bool_option(&mut self, opt: &str) -> Option<()> {
        match opt {
            "list" | "li" => self.options.list = true,
            "nolist" | "noli" => self.options.list = false,
            "number" | "nu" => self.options.number = true,
            "nonumber" | "nonu" => self.options.number = false,
            "wrap" => self.options.wrap = true,
            "nowrap" => self.options.wrap = false,
            "autoindent" | "ai" => self.options.autoindent = true,
            "noautoindent" | "noai" => self.options.autoindent = false,
            "paste" => self.options.paste = true,
            "nopaste" => self.options.paste = false,
            "expandtab" | "et" => self.options.expandtab = true,
            "noexpandtab" | "noet" => self.options.expandtab = false,
            "hlsearch" | "hls" => self.options.hlsearch = true,
            "nohlsearch" | "nohls" => self.options.hlsearch = false,
            "incsearch" | "is" => self.options.incsearch = true,
            "noincsearch" | "nois" => self.options.incsearch = false,
            "matchparen" => self.options.matchparen = true,
            "nomatchparen" => self.options.matchparen = false,
            "rainbow" => self.options.rainbow = true,
            "norainbow" => self.options.rainbow = false,
            "guides" => self.options.guides = true,
            "noguides" => self.options.guides = false,
            "gutter-signs" => self.options.gutter_signs = true,
            "nogutter-signs" => self.options.gutter_signs = false,
            _ => return None,
        }
        Some(())
    }

    fn apply_value_option(&mut self, opt: &str) {
        if let Some(n) = opt.strip_prefix("scrolloff=").and_then(|s| s.parse().ok()) {
            self.options.scrolloff = n;
        } else if let Some(n) = opt
            .strip_prefix("tabstop=")
            .or_else(|| opt.strip_prefix("ts="))
            .and_then(|s| s.parse().ok())
        {
            self.options.tab_width = n;
        } else if let Some(n) = opt
            .strip_prefix("shiftwidth=")
            .or_else(|| opt.strip_prefix("sw="))
            .and_then(|s| s.parse().ok())
        {
            self.options.shiftwidth = n;
        } else if let Some(style) = self.parse_cursor_set(opt) {
            style
        } else {
            self.status = format!("Unknown option: {opt}");
        }
    }

    fn parse_cursor_set(&mut self, opt: &str) -> Option<()> {
        let (key, val) = opt.split_once('=')?;
        let style = match val {
            "bar" => CursorStyle::Bar,
            "block" => CursorStyle::Block,
            "underline" => CursorStyle::Underline,
            "software" | "none" => CursorStyle::Software,
            _ => return None,
        };
        match key {
            "cursor_insert" => self.options.cursor_insert = style,
            "cursor_normal" => self.options.cursor_normal = style,
            "cursor_command" => self.options.cursor_command = style,
            _ => return None,
        }
        Some(())
    }
}
