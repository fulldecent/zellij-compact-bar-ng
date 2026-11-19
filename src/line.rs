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
    visible_width(&render_tab_strip(tabs, budget, chrome, None).0)
}

/// Widest hint row that fits in `remain`.
///
/// A labeled hint already includes the first chip's left pad, so one more
/// space separates it from the tabs. A key-only hint has no pad, so two
/// spaces separate it. Either way the glyph has two spaces in front of it.
fn status_span(mode: &str, remain: usize, theme: &Theme) -> (usize, usize) {
    if remain == 0 {
        return (0, 0);
    }
    if remain > 1 {
        let (ansi, _) = render_bar_with(mode, remain - 1, &theme.status, None);
        let text = strip_ansi(&ansi);
        let width = text.width();
        if width > 0 && text.starts_with(' ') && width < remain {
            return (width, 1);
        }
    }
    if remain > 2 {
        let (ansi, _) = render_bar_with(mode, remain - 2, &theme.status, None);
        let text = strip_ansi(&ansi);
        let width = text.width();
        // A help title can fill the wider budget and leave the buttons out.
        // The narrower row still starts with a space, so the gap stays one.
        if width > 0 && text.starts_with(' ') && width < remain {
            return (width, 1);
        }
        if width > 0 && !text.starts_with(' ') {
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
    render_tab_strip(tabs, cols, &theme.tabs, hover_col)
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
    use crate::layout::{Chrome, Click};

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
    fn rename_controls_stay_when_one_more_column_is_available() {
        let theme = Theme::default();
        let at_16 = status_span("rename_pane", 16, &theme);
        let at_17 = status_span("rename_pane", 17, &theme);
        assert!(at_16.0 > 0, "{at_16:?}");
        assert!(
            at_17.0 > 0,
            "17 columns dropped the rename controls: {at_17:?}"
        );
        assert_eq!(at_17.1, 1, "gap: {at_17:?}");
        let text = strip_ansi(&render_bar_with("rename_pane", at_17.0, &theme.status, None).0);
        assert!(text.contains("ESC"), "{text}");
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
        assert!(hint_text.starts_with(" ^G Lock"), "{hint_text}");
        assert!(text.ends_with(&hint_text));
        let at = text.len() - hint_text.len();
        assert!(
            text[..at].ends_with(' '),
            "one separator sits in front of the hint pad: {:?}",
            &text[at.saturating_sub(8)..]
        );
        let glyph = text.rfind("^G").expect("^G");
        assert_eq!(&text[glyph - 2..glyph], "  ", "{text}");
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
        assert!(hint_text.starts_with(" N New"), "{hint_text}");
        assert!(!hint_text.starts_with("Pane"), "{hint_text}");
        assert!(!hint_text.contains("Ctrl +"));
        assert!(text.ends_with(&hint_text));
        assert!(render_line(220, &state(InputMode::Locked, &tabs)).contains("38;5;201"));
    }

    #[test]
    fn mode_word_stays_on_the_left() {
        let tabs = tabs(1, 0);
        for (mode, word, heading) in [
            (InputMode::Pane, "PANE", "Pane"),
            (InputMode::Tab, "TAB", "Tab"),
            (InputMode::Resize, "RESIZE", "Resize"),
            (InputMode::Move, "MOVE", "Move"),
            (InputMode::Scroll, "SCROLL", "Search"),
            (InputMode::Session, "SESSION", "Session"),
            (InputMode::Tmux, "TMUX", "Tmux"),
        ] {
            let text = strip_ansi(&render_line(220, &state(mode, &tabs)));
            assert!(
                text.contains(&format!(" {word} ")),
                "{mode:?} left word: {text}"
            );
            let hints =
                strip_ansi(&render_bar_with(mode_name(mode), 400, &Chrome::default(), None).0);
            assert!(
                !hints.starts_with(heading),
                "{mode:?} repeats {heading} in {hints}"
            );
            for cols in 1..=80 {
                let narrow =
                    strip_ansi(&render_bar_with(mode_name(mode), cols, &Chrome::default(), None).0);
                assert!(
                    !narrow.starts_with(heading),
                    "{mode:?} at {cols} repeats {heading}: {narrow}"
                );
            }
        }
        let scroll = strip_ansi(&render_line(220, &state(InputMode::Scroll, &tabs)));
        assert!(scroll.contains(" SCROLL "), "{scroll}");
        assert!(!scroll.contains(" SEARCH "), "{scroll}");
        let session = strip_ansi(&render_line(220, &state(InputMode::Session, &tabs)));
        assert!(session.contains(" SESSION "), "{session}");
    }

    fn chip_slice(text: &str, hit: &BarHit) -> String {
        text.chars()
            .skip(hit.start)
            .take(hit.end - hit.start)
            .collect()
    }

    #[test]
    fn reviewed_buttons_have_one_pad_on_each_side() {
        let tabs = tabs(1, 0);
        let cases = [
            (InputMode::Normal, Click::Mode("locked"), "^G Lock"),
            (InputMode::Pane, Click::NewPane, "N New"),
            (InputMode::Tab, Click::SelectPane, "ENTER Select Pane"),
            (InputMode::Scroll, Click::EnterSearch, "S Enter Search Term"),
            (InputMode::Session, Click::Detach, "D Detach"),
        ];
        for (mode, click, body) in cases {
            let row = state(mode, &tabs);
            let text = strip_ansi(&render_line(220, &row));
            let hits = render_hits(220, &row);
            let hit = hits
                .iter()
                .find(|hit| hit.action == Action::Status(click))
                .unwrap_or_else(|| panic!("{mode:?} missing {body} in {text}"));
            assert_eq!(
                chip_slice(&text, hit),
                format!(" {body} "),
                "{mode:?} {text}"
            );
        }

        let pane = state(InputMode::Pane, &tabs);
        let pane_hits = render_hits(220, &pane);
        let new = pane_hits
            .iter()
            .find(|hit| hit.action == Action::Status(Click::NewPane))
            .unwrap();
        assert!(pane_hits
            .iter()
            .all(|hit| hit.end <= new.end || hit.start > new.end));

        let mut normal = state(InputMode::Normal, &tabs);
        let plain = strip_ansi(&render_line(220, &normal));
        let glyph = plain
            .chars()
            .collect::<Vec<_>>()
            .windows(2)
            .position(|pair| *pair == ['^', 'G'])
            .unwrap();
        normal.hover_col = Some(glyph - 1);
        let hovered = render_line(220, &normal);
        let cells = hover_cells(&hovered);
        let reverse = "38;5;16;48;5;245";
        assert!(cells[glyph - 1].contains(reverse), "{}", cells[glyph - 1]);
        assert!(cells[glyph].contains(reverse), "{}", cells[glyph]);
        assert!(
            !cells[glyph - 2].contains(reverse),
            "separator lit up: {}",
            cells[glyph - 2]
        );
    }

    fn hover_cells(rendered: &str) -> Vec<String> {
        let mut cells = Vec::new();
        let mut params = String::new();
        let mut chars = rendered.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    let mut seq = String::new();
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                        seq.push(c);
                    }
                    if seq == "0" {
                        params.clear();
                    } else {
                        params = seq;
                    }
                }
                continue;
            }
            cells.push(params.clone());
        }
        cells
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
