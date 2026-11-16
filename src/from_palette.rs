use zellij_tile::prelude::{PaletteColor, Style};

use crate::layout::{Chrome as StatusChrome, Ink as StatusInk};
use crate::tab_ribbon::{Chrome as TabChrome, Ink as TabInk};

/// Inks for the prefix, the ribbon tabs, and the key hints.
#[derive(Clone, Copy, Debug, Default)]
pub struct Theme {
    pub prefix: PrefixColors,
    pub tabs: TabChrome,
    pub status: StatusChrome,
}

/// Compact-bar colors for Zellij, the session name, and the mode word.
///
/// Defaults match `Styling::from(default_palette())` on Zellij 0.45:
/// text 255 on bar 16, normal mode green 154, locked mode magenta 201,
/// every other mode orange 166.
#[derive(Clone, Copy, Debug)]
pub struct PrefixColors {
    pub text: StatusInk,
    pub bar: StatusInk,
    pub normal: StatusInk,
    pub locked: StatusInk,
    pub other: StatusInk,
}

impl Default for PrefixColors {
    fn default() -> Self {
        Self {
            text: StatusInk::Bit(255),
            bar: StatusInk::Bit(16),
            normal: StatusInk::Bit(154),
            locked: StatusInk::Bit(201),
            other: StatusInk::Bit(166),
        }
    }
}

pub fn chrome_from(style: &Style) -> Theme {
    let colors = &style.colors;
    let text = &colors.text_unselected;
    Theme {
        prefix: PrefixColors {
            text: status_ink(text.base),
            bar: status_ink(text.background),
            normal: status_ink(text.emphasis_2),
            locked: status_ink(text.emphasis_3),
            other: status_ink(text.emphasis_0),
        },
        tabs: TabChrome {
            bar: tab_ink(text.background),
            dark: tab_ink(colors.ribbon_unselected.base),
            light: tab_ink(colors.ribbon_unselected.background),
            plus_hover: tab_ink(colors.ribbon_unselected.emphasis_1),
            title: tab_ink(text.base),
            selected_text: tab_ink(colors.ribbon_selected.base),
            selected_fill: tab_ink(colors.ribbon_selected.background),
        },
        status: StatusChrome {
            bar: status_ink(text.background),
            dark: status_ink(colors.ribbon_unselected.base),
            light: status_ink(colors.ribbon_unselected.background),
            ctrl: status_ink(text.base),
            alt: status_ink(text.emphasis_0),
            plus_hover: status_ink(colors.ribbon_unselected.emphasis_1),
            key_accent: status_ink(colors.ribbon_unselected.emphasis_0),
        },
    }
}

fn status_ink(color: PaletteColor) -> StatusInk {
    match color {
        PaletteColor::EightBit(n) => StatusInk::Bit(n),
        PaletteColor::Rgb((r, g, b)) => StatusInk::Rgb(r, g, b),
    }
}

fn tab_ink(color: PaletteColor) -> TabInk {
    match color {
        PaletteColor::EightBit(n) => TabInk::Bit(n),
        PaletteColor::Rgb((r, g, b)) => TabInk::Rgb(r, g, b),
    }
}
