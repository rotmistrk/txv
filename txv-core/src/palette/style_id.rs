//! StyleId — semantic identifiers for palette styles.

/// Identifies a semantic style role. Views query the palette by ID.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum StyleId {
    // Base
    Text,
    Dim,
    Bright,
    Border,
    Separator,
    TreeDir,
    // Interactive
    CursorFocused,
    CursorUnfocused,
    InputCursor,
    EditOverlay,
    EditSelection,
    SearchMatch,
    VisualSelection,
    Disabled,
    // Chrome
    ChromeBar,
    TabFocused,
    TabFocusedArrow,
    TabFocusedBadge,
    TabActive,
    TabActiveArrow,
    TabActiveBadge,
    TabInactive,
    StatusBar,
    StatusBarModal,
    ScrollbarTrack,
    ScrollbarThumb,
    // Popup
    PopupBackground,
    PopupBorder,
    PopupSelected,
    PopupTableHeader,
    // State
    StateError,
    StateWarning,
    StateInfo,
    StateSuccess,
    StateHint,
    // Editor (app-level, but common enough)
    EditorGutter,
}

impl StyleId {
    pub const COUNT: usize = 39;
}
