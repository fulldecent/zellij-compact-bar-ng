//! One row: Zellij, the session name, the compact-bar mode, ribbon tabs,
//! two spaces, then the key hints. Tab names stay whole. The hints shorten
//! into the columns that remain.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use zellij_tile::prelude::InputMode;

use crate::from_palette::Theme;
use crate::layout::{render_bar_with, Click, Hit as StatusHit, Ink};
use crate::tab_ribbon::{render_tab_strip, strip_ansi as strip_ribbon_ansi, TabAction, TabSpec};

/// What a click on this row should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Tab(TabAction),
    Status(Click),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarHit {
    pub start: usize,
    pub end: usize,
    pub action: Action,
}

/// Snapshot the compact row would paint. No Zellij I/O.
pub struct BarState<'a> {
    pub tabs: &'a [TabSpec],
    pub session_name: Option<&'a str>,
    pub hide_session_name: bool,
    pub mode: InputMode,
    pub dimmed: bool,
    pub theme: Theme,
    pub hover_col: Option<usize>,
}

impl<'a> Default for BarState<'a> {
    fn default() -> Self {
        Self {
            tabs: &[],
            session_name: None,
            hide_session_name: false,
            mode: InputMode::Normal,
            dimmed: false,
            theme: Theme::default(),
            hover_col: None,
        }
    }
}

pub fn render_line(cols: usize, state: &BarState<'_>) -> String {
    painted(cols, state).0
}

pub fn render_hits(cols: usize, state: &BarState<'_>) -> Vec<BarHit> {
    painted(cols, state).1
}

pub fn visible_width(rendered: &str) -> usize {
    strip_ansi(rendered).width()
}

pub fn strip_ansi(rendered: &str) -> String {
    strip_ribbon_ansi(rendered)
}

fn painted(cols: usize, state: &BarState<'_>) -> (String, Vec<BarHit>) {
    if cols == 0 {
        return (String::new(), Vec::new());
    }

    let (prefix_ansi, prefix_w) = prefix(state, cols);
    let remain = cols - prefix_w;
    let mode = mode_name(state.mode);
    // Full ribbons, including `+`, come out of `remain` before any hint does.
    // Hints shorten in what is left. A tab name shrinks only when the ribbons
    // themselves are wider than the row.
    let tab_reserve = preferred_tab_width(state.tabs, &state.theme.tabs).min(remain);
    let (status_w, gap) = status_span(mode, remain - tab_reserve, &state.theme);
    let tab_w = remain - gap - status_w;

    let tab_hover = state.hover_col.and_then(|col| {
        if tab_w > 0 && col >= prefix_w && col < prefix_w + tab_w {
            Some(col - prefix_w)
        } else {
            None
        }
    });
    let (tab_ansi, tab_hits) = tabs_painted(state.tabs, tab_w, &state.theme, tab_hover);
    let tab_painted = visible_width(&tab_ansi);
    let tab_pad = tab_w.saturating_sub(tab_painted);

    let status_start = prefix_w + tab_w + gap;
    let status_hover = state.hover_col.and_then(|col| {
        if status_w > 0 && col >= status_start && col < status_start + status_w {
            Some(col - status_start)
        } else {
            None
        }
    });
    let (status_ansi, status_hits) = if status_w == 0 {
        (String::new(), Vec::new())
    } else {
        render_bar_with(mode, status_w, &state.theme.status, status_hover)
    };
    let status_painted = visible_width(&status_ansi);

    let bg = state.theme.prefix.bar;
    let mut line = prefix_ansi;
    line.push_str(&tab_ansi);
    line.push_str(&bar_spaces(tab_pad + gap, bg));
    line.push_str(&status_ansi);
    let used = prefix_w + tab_painted + tab_pad + gap + status_painted;
    if used < cols {
        line.push_str(&bar_spaces(cols - used, bg));
    }

    let mut hits = Vec::new();
    for hit in tab_hits {
        hits.push(BarHit {
            start: prefix_w + hit.start,
            end: prefix_w + hit.end,
            action: Action::Tab(hit.action),
        });
    }
    for StatusHit { start, end, click } in status_hits {
        hits.push(BarHit {
            start: status_start + start,
            end: status_start + end,
            action: Action::Status(click),
        });
    }
    (line, hits)
}

/// Columns the ribbons need when every name is whole and `+` is showing.
fn preferred_tab_width(tabs: &[TabSpec], chrome: &crate::tab_ribbon::Chrome) -> usize {
    if tabs.is_empty() {
        return 0;
    }
    let mut budget = 8;
    for tab in tabs {
        budget += 8 + tab.name.width();
        if tab.fullscreen {
            budget += 7;
        }
        if tab.sync {
            budget += 7;
        }
    }
    visible_width(&render_tab_strip(tabs, budget, chrome, false).0)
}

/// Widest hint row that fits in `remain`, leaving two spaces when that row fits beside them.
fn status_span(mode: &str, remain: usize, theme: &Theme) -> (usize, usize) {
    if remain == 0 {
        return (0, 0);
    }
    if remain > 2 {
        let (ansi, _) = render_bar_with(mode, remain - 2, &theme.status, None);
        let width = visible_width(&ansi);
        if width > 0 {
            return (width, 2);
        }
    }
    let (ansi, _) = render_bar_with(mode, remain, &theme.status, None);
    (visible_width(&ansi), 0)
}

fn tabs_painted(
    tabs: &[TabSpec],
    cols: usize,
    theme: &Theme,
    hover_col: Option<usize>,
) -> (String, Vec<crate::tab_ribbon::TabHit>) {
    if cols == 0 || tabs.is_empty() {
        return (String::new(), Vec::new());
    }
    let (line, hits) = render_tab_strip(tabs, cols, &theme.tabs, false);
    let plus_hovered = hover_col.is_some_and(|col| {
        hits.iter()
            .any(|hit| matches!(hit.action, TabAction::NewTab) && col >= hit.start && col < hit.end)
    });
    if plus_hovered {
        render_tab_strip(tabs, cols, &theme.tabs, true)
    } else {
        (line, hits)
    }
}

fn prefix(state: &BarState<'_>, cols: usize) -> (String, usize) {
    let colors = &state.theme.prefix;
    let mut ansi = String::new();
    let mut used = 0usize;

    let zellij = " Zellij ";
    let zellij_text: String = zellij
        .chars()
        .scan(0usize, |used_cols, ch| {
            let width = ch.width().unwrap_or(0);
            if *used_cols + width > cols {
                return None;
            }
            *used_cols += width;
            Some(ch)
        })
        .collect();
    if !zellij_text.is_empty() {
        let width = zellij_text.width();
        ansi.push_str(&paint(
            &zellij_text,
            colors.text,
            colors.bar,
            !state.dimmed,
            state.dimmed,
        ));
        used += width;
    }

    if !state.hide_session_name {
        if let Some(name) = state.session_name.filter(|name| !name.is_empty()) {
            let part = format!("({name})");
            let width = part.width();
            if used + width <= cols {
                ansi.push_str(&paint(
                    &part,
                    colors.text,
                    colors.bar,
                    !state.dimmed,
                    state.dimmed,
                ));
                used += width;
            }
        }
    }

    let mode_text = format!(" {} ", format!("{:?}", state.mode).to_uppercase());
    let mode_width = mode_text.width();
    if used + mode_width <= cols {
        let fg = if state.dimmed {
            colors.text
        } else {
            match state.mode {
                InputMode::Normal => colors.normal,
                InputMode::Locked => colors.locked,
                _ => colors.other,
            }
        };
        ansi.push_str(&paint(
            &mode_text,
            fg,
            colors.bar,
            !state.dimmed,
            state.dimmed,
        ));
        used += mode_width;
    }

    (ansi, used)
}

fn paint(text: &str, fg: Ink, bg: Ink, bold: bool, italic: bool) -> String {
    let attr = if italic {
        "3"
    } else if bold {
        "1"
    } else {
        "22"
    };
    format!(
        "\u{1b}[0m\u{1b}[{attr};{}m\u{1b}[{}m{text}",
        fg.fg(),
        bg.bg()
    )
}

fn bar_spaces(n: usize, bg: Ink) -> String {
    if n == 0 {
        String::new()
    } else {
        format!("\u{1b}[0m\u{1b}[{}m{}", bg.bg(), " ".repeat(n))
    }
}

fn mode_name(mode: InputMode) -> &'static str {
    match mode {
        InputMode::Normal => "normal",
        InputMode::Locked => "locked",
        InputMode::Pane => "pane",
        InputMode::Tab => "tab",
        InputMode::Resize => "resize",
        InputMode::Move => "move",
        InputMode::Scroll => "scroll",
        InputMode::EnterSearch => "enter_search",
        InputMode::Search => "search",
        InputMode::RenameTab => "rename_tab",
        InputMode::RenamePane => "rename_pane",
        InputMode::Session => "session",
        InputMode::Tmux => "tmux",
        InputMode::Prompt => "prompt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Chrome;

    fn tabs(n: usize, active: usize) -> Vec<TabSpec> {
        (0..n)
            .map(|i| TabSpec {
                index: (i + 1) as u32,
                name: format!("Tab #{}", i + 1),
                active: i == active,
                fullscreen: false,
                sync: false,
            })
            .collect()
    }

    fn state<'a>(mode: InputMode, tabs: &'a [TabSpec]) -> BarState<'a> {
        BarState {
            tabs,
            session_name: Some("demo"),
            hide_session_name: false,
            mode,
            dimmed: false,
            theme: Theme::default(),
            hover_col: None,
        }
    }

    #[test]
    fn wide_row_is_zellij_session_mode_tabs_then_hints() {
        let tabs = tabs(3, 1);
        let row = state(InputMode::Normal, &tabs);
        let line = render_line(220, &row);
        let text = strip_ansi(&line);
        assert_eq!(text.width(), 220);
        assert!(text.starts_with(" Zellij (demo) NORMAL "));
        assert!(text.contains("Tab #1"));
        assert!(text.contains("Tab #2"));
        assert!(text.contains("\u{e0ba}"));
        assert_eq!(text.matches("Zellij").count(), 1);
        assert!(!text.contains("Ctrl +"));
        assert!(!text.contains("Alt"));
        assert!(!text.contains('⌥'));

        let (hints, _) = render_bar_with("normal", 400, &Chrome::default(), None);
        let hint_text = strip_ansi(&hints);
        assert!(hint_text.starts_with("^G Lock"));
        assert!(text.ends_with(&hint_text));
        let at = text.len() - hint_text.len();
        assert!(
            text[..at].ends_with("  "),
            "two spaces sit in front of the hints: {:?}",
            &text[at.saturating_sub(8)..]
        );
        assert!(line.contains("38;5;154"));
    }

    #[test]
    fn tab_names_stay_whole_while_hints_shrink() {
        let tabs = tabs(3, 1);
        let text = strip_ansi(&render_line(80, &state(InputMode::Normal, &tabs)));
        assert!(text.contains("Tab #1"));
        assert!(text.contains("Tab #2"));
        assert!(text.contains("Tab #3"));
        assert!(!text.contains('…'));
        assert!(!text.contains('←'));
        assert!(!text.contains("Session"));
        assert!(text.contains("^G"));
    }

    #[test]
    fn hints_use_only_the_columns_they_need() {
        let tabs = tabs(1, 0);
        let text = strip_ansi(&render_line(220, &state(InputMode::Normal, &tabs)));
        let (hints, _) = render_bar_with("normal", 400, &Chrome::default(), None);
        let hint_text = strip_ansi(&hints);
        assert!(hint_text.width() < 120);
        assert!(text.ends_with(&hint_text));
        assert!(!hint_text.contains("Ctrl +"));
        assert!(!hint_text.contains("Alt"));
    }

    #[test]
    fn pane_mode_keeps_the_compact_word_and_drops_ctrl() {
        let tabs = tabs(1, 0);
        let text = strip_ansi(&render_line(220, &state(InputMode::Pane, &tabs)));
        assert!(text.contains(" Zellij (demo) PANE "));
        let (hints, _) = render_bar_with("pane", 400, &Chrome::default(), None);
        let hint_text = strip_ansi(&hints);
        assert!(hint_text.starts_with("Pane "));
        assert!(!hint_text.contains("Ctrl +"));
        assert!(text.ends_with(&hint_text));
        assert!(render_line(220, &state(InputMode::Locked, &tabs)).contains("38;5;201"));
    }

    #[test]
    fn hidden_session_name_is_left_out() {
        let tabs = tabs(1, 0);
        let mut row = state(InputMode::Normal, &tabs);
        row.hide_session_name = true;
        let text = strip_ansi(&render_line(80, &row));
        assert!(text.starts_with(" Zellij  NORMAL "));
        assert!(!text.contains("(demo)"));
    }

    #[test]
    fn every_width_fills_the_row_and_stays_inside_it() {
        let tabs = tabs(4, 2);
        let row = state(InputMode::Session, &tabs);
        for cols in 1..=100 {
            let line = render_line(cols, &row);
            assert_eq!(visible_width(&line), cols, "{cols}");
            let text = strip_ansi(&line);
            assert!(!text.contains("Ctrl +"), "{cols}: {text}");
            assert!(!text.contains("Alt"), "{cols}: {text}");
        }
    }
}
