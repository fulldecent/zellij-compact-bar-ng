//! Key hints for one input mode. The `Ctrl +` label and the Alt group are
//! omitted. Full names, then shorter names, then keys only from the right.
//! Once the keys themselves no longer fit, keep as many as possible and end
//! with "...", and only then a colored ellipsis. The painted block is only
//! as wide as the form that fit.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Style {
    Plain,
    BoldOrange,
    Reverse,
}

/// What a click on a painted range should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    Mode(&'static str),
    Quit,
    NewPane,
    NewPaneDown,
    NewPaneRight,
    NewStacked,
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    ResizeIncrease,
    ResizeDecrease,
    ResizeIncreaseLeft,
    ResizeIncreaseDown,
    ResizeIncreaseUp,
    ResizeIncreaseRight,
    ResizeDecreaseLeft,
    ResizeDecreaseDown,
    ResizeDecreaseUp,
    ResizeDecreaseRight,
    MovePaneLeft,
    MovePaneDown,
    MovePaneUp,
    MovePaneRight,
    ToggleFloating,
    ToggleEmbed,
    ToggleFullscreen,
    ClosePane,
    CloseTab,
    NewTab,
    RenamePane,
    RenameTab,
    SyncTab,
    BreakPane,
    BreakPaneLeft,
    BreakPaneRight,
    ToggleTab,
    SelectPane,
    Detach,
    SessionManager,
    Configuration,
    PluginManager,
    About,
    Share,
    LayoutManager,
    EditScrollback,
    ScrollUp,
    ScrollDown,
    EnterSearch,
    CancelSearch,
    ConfirmSearch,
    SearchDown,
    SearchUp,
    SearchCase,
    SearchWrap,
    SearchWhole,
    TabPrev,
    TabNext,
    PageScrollUp,
    PageScrollDown,
    DoneRenamePane,
    DoneRenameTab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub start: usize,
    pub end: usize,
    pub click: Click,
}

#[derive(Clone, Debug)]
struct Chunk {
    style: Style,
    text: String,
    hit: Option<Click>,
}

impl Chunk {
    fn plain(text: impl Into<String>) -> Self {
        Self {
            style: Style::Plain,
            text: text.into(),
            hit: None,
        }
    }
    fn orange(text: impl Into<String>) -> Self {
        Self {
            style: Style::BoldOrange,
            text: text.into(),
            hit: None,
        }
    }
    fn rev(text: impl Into<String>) -> Self {
        Self {
            style: Style::Reverse,
            text: text.into(),
            hit: None,
        }
    }
    fn rev_hit(text: impl Into<String>, hit: Click) -> Self {
        Self {
            style: Style::Reverse,
            text: text.into(),
            hit: Some(hit),
        }
    }
    fn plain_hit(text: impl Into<String>, hit: Click) -> Self {
        Self {
            style: Style::Plain,
            text: text.into(),
            hit: Some(hit),
        }
    }
}

struct Hint {
    token: &'static str,
    full: &'static str,
    abbr: &'static str,
    kind: Kind,
}

enum Kind {
    Chip(Click),
    Keys(&'static [Click]),
}

fn width(chunks: &[Chunk]) -> usize {
    chunks.iter().map(|c| c.text.chars().count()).sum()
}

fn plain_text(chunks: &[Chunk]) -> String {
    chunks.iter().map(|c| c.text.as_str()).collect()
}

fn token_chunks(h: &Hint) -> Vec<Chunk> {
    match h.kind {
        Kind::Chip(click) => vec![Chunk::rev_hit(h.token, click)],
        Kind::Keys(keys) => {
            let chars: Vec<char> = h.token.chars().collect();
            let prefix = chars.len().saturating_sub(keys.len());
            let mut out = Vec::new();
            if prefix > 0 {
                let head: String = chars[..prefix].iter().collect();
                out.push(Chunk::rev(head));
            }
            for (i, ch) in chars[prefix..].iter().enumerate() {
                out.push(Chunk::rev_hit(ch.to_string(), keys[i]));
            }
            out
        }
    }
}

fn named(h: &Hint, label: &str) -> Vec<Chunk> {
    let mut out = token_chunks(h);
    match h.kind {
        Kind::Chip(click) => {
            out.push(Chunk::plain_hit(format!(" {label}"), click));
            out.push(Chunk::plain("  "));
        }
        Kind::Keys(_) => {
            out.push(Chunk::plain(format!(" {label}  ")));
        }
    }
    out
}

fn key_only(h: &Hint) -> Vec<Chunk> {
    let mut out = token_chunks(h);
    out.push(Chunk::plain("  "));
    out
}

fn chip(token: &'static str, full: &'static str, abbr: &'static str, click: Click) -> Hint {
    Hint {
        token,
        full,
        abbr,
        kind: Kind::Chip(click),
    }
}

fn keys(
    token: &'static str,
    full: &'static str,
    abbr: &'static str,
    clicks: &'static [Click],
) -> Hint {
    Hint {
        token,
        full,
        abbr,
        kind: Kind::Keys(clicks),
    }
}

/// Largest form first. The last entry is the colored ellipsis with no prefix.
fn variants(prefix: Vec<Chunk>, items: &[Hint], abbreviate: bool) -> Vec<Vec<Chunk>> {
    let mut out: Vec<Vec<Chunk>> = Vec::new();
    let mut push = |chunks: Vec<Chunk>| {
        let text = plain_text(&chunks);
        if out
            .last()
            .map(|prev| plain_text(prev) != text)
            .unwrap_or(true)
        {
            out.push(chunks);
        }
    };

    let mut full = prefix.clone();
    for item in items {
        full.extend(named(item, item.full));
    }
    push(full);

    if abbreviate {
        let mut abbr = prefix.clone();
        for item in items {
            abbr.extend(named(item, item.abbr));
        }
        push(abbr);
    }

    let n = items.len();
    for dropped in 1..=n {
        let mut row = prefix.clone();
        for (i, item) in items.iter().enumerate() {
            if i + dropped < n {
                let label = if abbreviate { item.abbr } else { item.full };
                row.extend(named(item, label));
            } else {
                row.extend(key_only(item));
            }
        }
        push(row);
    }

    // Drop keys from the right, but keep an ellipsis so the cut is visible.
    // One column under the all-keys row still shows every key that fits.
    for keep in (1..n).rev() {
        let mut row = prefix.clone();
        for item in items.iter().take(keep - 1) {
            row.extend(key_only(item));
        }
        row.push(Chunk::rev(items[keep - 1].token));
        row.push(Chunk::plain(" ..."));
        push(row);
    }

    push(vec![Chunk::orange("...")]);
    out
}

fn first_fit(options: &[Vec<Chunk>], cols: usize) -> Vec<Chunk> {
    options
        .iter()
        .find(|option| width(option) <= cols)
        .cloned()
        .unwrap_or_default()
}

/// Colors taken from the same palette fields the stock status bar uses.
/// Defaults match `Styling::from(default_palette())`:
/// `ribbon_unselected.base` is black 16, `ribbon_unselected.background` is fg 245.
#[derive(Clone, Copy, Debug)]
pub struct Chrome {
    pub bar: Ink,
    pub dark: Ink,
    pub light: Ink,
    pub ctrl: Ink,
    pub alt: Ink,
    pub plus_hover: Ink,
    /// `ribbon_unselected.emphasis_0` — the red key letter inside a stock `<g>` chip.
    pub key_accent: Ink,
}

impl Default for Chrome {
    fn default() -> Self {
        Self {
            bar: Ink::Bit(16),
            dark: Ink::Bit(16),
            light: Ink::Bit(245),
            ctrl: Ink::Bit(255),
            alt: Ink::Bit(166),
            plus_hover: Ink::Bit(255),
            key_accent: Ink::Bit(124),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Ink {
    Bit(u8),
    Rgb(u8, u8, u8),
}

impl Ink {
    pub(crate) fn fg(self) -> String {
        match self {
            Ink::Bit(n) => format!("38;5;{n}"),
            Ink::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }
    pub(crate) fn bg(self) -> String {
        match self {
            Ink::Bit(n) => format!("48;5;{n}"),
            Ink::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
        }
    }
}

fn is_chip_label(chunk: &Chunk) -> bool {
    chunk.hit.is_some() && chunk.style == Style::Plain && chunk.text.chars().any(|ch| ch != ' ')
}

fn click_has_label(chunks: &[Chunk], click: Click) -> bool {
    chunks.iter().any(|chunk| {
        chunk.hit == Some(click)
            && chunk.style == Style::Plain
            && chunk.text.chars().any(|ch| ch != ' ')
    })
}

/// Click of a labeled chip whose key chunk is `chunks[idx]`.
fn labeled_chip_at(chunks: &[Chunk], idx: usize) -> Option<Click> {
    let click = chunks.get(idx)?.hit?;
    if !click_has_label(chunks, click) {
        return None;
    }
    for chunk in chunks.iter().skip(idx + 1) {
        if chunk.hit == Some(click)
            && chunk.style == Style::Plain
            && chunk.text.chars().any(|ch| ch != ' ')
        {
            return Some(click);
        }
        if chunk.hit.is_some() && chunk.hit != Some(click) {
            return None;
        }
        if chunk.text.chars().any(|ch| ch != ' ')
            && chunk.hit != Some(click)
            && chunk.style != Style::Reverse
        {
            return None;
        }
    }
    None
}

/// One space on each side of a labeled chip joins that chip. The first of the
/// two spaces after the name is its right side. The second is the left side of
/// the next labeled chip, and stays dead when the next control is not one
/// (`HJKL`, a key with no name, or the end of the row). A prefix space
/// immediately before a labeled chip is that chip's left side.
fn bind_chip_pads(chunks: &[Chunk]) -> Vec<Chunk> {
    let mut spaced = Vec::with_capacity(chunks.len());
    let mut i = 0;
    while i < chunks.len() {
        let separator = chunks
            .get(i + 1)
            .is_some_and(|chunk| chunk.hit.is_none() && chunk.text == "  ");
        if separator && is_chip_label(&chunks[i]) {
            let click = chunks[i].hit.expect("labeled chip");
            spaced.push(chunks[i].clone());
            spaced.push(Chunk::plain_hit(" ", click));
            match labeled_chip_at(chunks, i + 2) {
                Some(next) => spaced.push(Chunk::plain_hit(" ", next)),
                None => spaced.push(Chunk::plain(" ")),
            }
            i += 2;
            continue;
        }
        spaced.push(chunks[i].clone());
        i += 1;
    }

    let mut out = Vec::with_capacity(spaced.len());
    let mut i = 0;
    while i < spaced.len() {
        let peel = spaced[i].hit.is_none()
            && spaced[i].text.ends_with(' ')
            && spaced[i].text.chars().any(|ch| ch != ' ');
        if peel {
            if let Some(click) = labeled_chip_at(&spaced, i + 1) {
                let mut prefix = spaced[i].clone();
                prefix.text.pop();
                out.push(prefix);
                out.push(Chunk::plain_hit(" ", click));
                i += 1;
                continue;
            }
        }
        out.push(spaced[i].clone());
        i += 1;
    }
    out
}

fn hovered_click(chunks: &[Chunk], col: usize) -> Option<Click> {
    let mut x = 0;
    for chunk in chunks {
        let w = chunk.text.chars().count();
        if col >= x && col < x + w {
            return chunk.hit;
        }
        x += w;
    }
    None
}

fn paint(chunks: &[Chunk], chrome: &Chrome, hover: Option<usize>) -> String {
    let active = hover.and_then(|col| hovered_click(chunks, col));
    // A labeled chip reverses the key and the name together. A key with no
    // name still flips from reversed to normal.
    let labeled = active.is_some_and(|click| click_has_label(chunks, click));
    let mut out = String::new();
    for chunk in chunks {
        out.push_str("\u{1b}[0m");
        let same = chunk.hit.is_some() && chunk.hit == active;
        let params = if same && labeled {
            format!("22;{};{}", chrome.dark.fg(), chrome.light.bg())
        } else {
            let invert = same;
            match (chunk.style, invert) {
                (Style::Plain, false) => format!("22;{};{}", chrome.light.fg(), chrome.bar.bg()),
                (Style::Plain, true) => format!("22;{};{}", chrome.bar.fg(), chrome.light.bg()),
                (Style::BoldOrange, false) => format!("22;{};{}", chrome.alt.fg(), chrome.bar.bg()),
                (Style::BoldOrange, true) => format!("22;{};{}", chrome.bar.fg(), chrome.alt.bg()),
                (Style::Reverse, false) => format!("22;{};{}", chrome.dark.fg(), chrome.light.bg()),
                (Style::Reverse, true) => format!("22;{};{}", chrome.light.fg(), chrome.dark.bg()),
            }
        };
        out.push_str(&format!("\u{1b}[{params}m"));
        out.push_str(&chunk.text);
    }
    out.push_str("\u{1b}[0m");
    out
}

fn hits_of(chunks: &[Chunk]) -> Vec<Hit> {
    let mut hits: Vec<Hit> = Vec::new();
    let mut x = 0;
    for chunk in chunks {
        let w = chunk.text.chars().count();
        if let Some(click) = chunk.hit {
            if let Some(last) = hits.last_mut() {
                if last.click == click && last.end == x {
                    last.end = x + w;
                    x += w;
                    continue;
                }
            }
            hits.push(Hit {
                start: x,
                end: x + w,
                click,
            });
        }
        x += w;
    }
    hits
}

const FOCUS_HJKL: [Click; 4] = [
    Click::FocusLeft,
    Click::FocusDown,
    Click::FocusUp,
    Click::FocusRight,
];
const RESIZE_PM: [Click; 2] = [Click::ResizeIncrease, Click::ResizeDecrease];
const RESIZE_INC: [Click; 4] = [
    Click::ResizeIncreaseLeft,
    Click::ResizeIncreaseDown,
    Click::ResizeIncreaseUp,
    Click::ResizeIncreaseRight,
];
const RESIZE_DEC: [Click; 4] = [
    Click::ResizeDecreaseLeft,
    Click::ResizeDecreaseDown,
    Click::ResizeDecreaseUp,
    Click::ResizeDecreaseRight,
];
const MOVE_HJKL: [Click; 4] = [
    Click::MovePaneLeft,
    Click::MovePaneDown,
    Click::MovePaneUp,
    Click::MovePaneRight,
];
const SCROLL_HJKL: [Click; 4] = [
    Click::PageScrollUp,
    Click::ScrollDown,
    Click::ScrollUp,
    Click::PageScrollDown,
];
const TAB_HL: [Click; 2] = [Click::TabPrev, Click::TabNext];
const BREAK_BRACKETS: [Click; 2] = [Click::BreakPaneLeft, Click::BreakPaneRight];

fn normal_modes() -> Vec<Hint> {
    vec![
        chip("^G", "Lock", "Lock", Click::Mode("locked")),
        chip("^P", "Pane", "Pane", Click::Mode("pane")),
        chip("^T", "Tab", "Tab", Click::Mode("tab")),
        chip("^N", "Resize", "Resize", Click::Mode("resize")),
        chip("^H", "Move", "Move", Click::Mode("move")),
        chip("^S", "Search", "Search", Click::Mode("scroll")),
        chip("^O", "Session", "Session", Click::Mode("session")),
        chip("^Q", "Quit", "Quit", Click::Quit),
    ]
}

fn help_title(mode: &str) -> Option<&'static str> {
    match mode {
        "rename_pane" => Some("Renaming Pane"),
        "rename_tab" => Some("Renaming Tab"),
        "enter_search" => Some("Entering Search Term"),
        "search" => Some("Searching"),
        _ => None,
    }
}

fn indicator(mode: &str) -> Option<Vec<Hint>> {
    let items = match mode {
        "normal" => normal_modes(),
        "locked" => vec![chip("^G", "Lock", "Lock", Click::Mode("normal"))],
        _ => return None,
    };
    Some(items)
}

fn mode_actions(mode: &str) -> Vec<Hint> {
    match mode {
        "pane" => vec![
            chip("N", "New", "New", Click::NewPane),
            keys("HJKL", "Change Focus", "Move", &FOCUS_HJKL),
            chip("X", "Close", "Close", Click::ClosePane),
            chip("C", "Rename", "Rename", Click::RenamePane),
            chip(
                "F",
                "Toggle Fullscreen",
                "Fullscreen",
                Click::ToggleFullscreen,
            ),
            chip("W", "Toggle Floating", "Floating", Click::ToggleFloating),
            chip("E", "Toggle Embed", "Embed", Click::ToggleEmbed),
            chip("R", "Split Right", "Right", Click::NewPaneRight),
            chip("D", "Split Down", "Down", Click::NewPaneDown),
            chip("S", "Stack", "Stack", Click::NewStacked),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "tab" => vec![
            chip("N", "New", "New", Click::NewTab),
            keys("HL", "Change Focus", "Move", &TAB_HL),
            chip("X", "Close", "Close", Click::CloseTab),
            chip("R", "Rename", "Rename", Click::RenameTab),
            chip("S", "Sync", "Sync", Click::SyncTab),
            chip("B", "Break Pane To New Tab", "Break Out", Click::BreakPane),
            keys("[]", "Break Pane Left/Right", "Break", &BREAK_BRACKETS),
            chip("TAB", "Toggle", "Toggle", Click::ToggleTab),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "resize" => vec![
            keys(
                "+-",
                "Increase/Decrease Size",
                "Increase/Decrease",
                &RESIZE_PM,
            ),
            keys("HJKL", "Increase To", "Increase", &RESIZE_INC),
            keys("HJKL", "Decrease From", "Decrease", &RESIZE_DEC),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "move" => vec![
            keys("HJKL", "Switch Location", "Move", &MOVE_HJKL),
            chip("ENTER", "When Done", "Back", Click::SelectPane),
        ],
        "scroll" => vec![
            chip("S", "Enter Search Term", "Search", Click::EnterSearch),
            keys("HJKL", "Scroll", "Scroll", &SCROLL_HJKL),
            chip(
                "E",
                "Edit Scrollback In Default Editor",
                "Edit",
                Click::EditScrollback,
            ),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "enter_search" => vec![
            chip("ENTER", "When Done", "Done", Click::ConfirmSearch),
            chip("ESC", "Cancel", "Cancel", Click::CancelSearch),
        ],
        "search" => vec![
            chip("N", "Search Down", "Down", Click::SearchDown),
            chip("P", "Search Up", "Up", Click::SearchUp),
            chip("C", "Case Sensitive", "Case", Click::SearchCase),
            chip("W", "Wrap", "Wrap", Click::SearchWrap),
            chip("O", "Whole Words", "Whole", Click::SearchWhole),
        ],
        "session" => vec![
            chip("D", "Detach", "Detach", Click::Detach),
            chip("W", "Session Manager", "Manager", Click::SessionManager),
            chip("S", "Share", "Share", Click::Share),
            chip("C", "Configure", "Config", Click::Configuration),
            chip("L", "Layout Manager", "Layouts", Click::LayoutManager),
            chip("P", "Plugin Manager", "Plugins", Click::PluginManager),
            chip("A", "About", "About", Click::About),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "tmux" => vec![
            keys("HJKL", "Move Focus", "Move", &FOCUS_HJKL),
            chip("\"", "Split Down", "Down", Click::NewPaneDown),
            chip("%", "Split Right", "Right", Click::NewPaneRight),
            chip("Z", "Fullscreen", "Fullscreen", Click::ToggleFullscreen),
            chip("C", "New Tab", "New", Click::NewTab),
            chip(",", "Rename Tab", "Rename", Click::RenameTab),
            chip("P", "Previous Tab", "Previous", Click::TabPrev),
            chip("N", "Next Tab", "Next", Click::TabNext),
            chip("ENTER", "Select Pane", "Select", Click::SelectPane),
        ],
        "rename_pane" => vec![chip("ESC", "When Done", "Done", Click::DoneRenamePane)],
        "rename_tab" => vec![chip("ESC", "When Done", "Done", Click::DoneRenameTab)],
        _ => vec![],
    }
}

/// Largest hint row that fits in `cols`.
///
/// There is no `Ctrl +` label and no Alt group. The result is only as wide as
/// the chosen row. Callers place that block. They do not stretch it.
fn leading_pad_needed(chunks: &[Chunk]) -> bool {
    labeled_chip_at(chunks, 0).is_some()
}

/// The compact row has no `Ctrl +` space to peel. The first labeled chip takes
/// one column for its left side. That column comes out of the separator in
/// front of the hint block, so the glyph does not move.
fn fit_chunks(mode: &str, cols: usize) -> Vec<Chunk> {
    if cols == 0 {
        return Vec::new();
    }
    let fitted = bind_chip_pads(&status_chunks(mode, cols));
    if !leading_pad_needed(&fitted) || cols == 1 {
        return fitted;
    }
    let mut reserved = bind_chip_pads(&status_chunks(mode, cols - 1));
    if leading_pad_needed(&reserved) {
        let click = labeled_chip_at(&reserved, 0).expect("labeled chip");
        reserved.insert(0, Chunk::plain_hit(" ", click));
        if width(&reserved) <= cols {
            return reserved;
        }
    } else if !reserved.is_empty() && width(&reserved) <= cols {
        return reserved;
    }
    fitted
}

pub fn render_bar_with(
    mode: &str,
    cols: usize,
    chrome: &Chrome,
    hover: Option<usize>,
) -> (String, Vec<Hit>) {
    let chunks = fit_chunks(mode, cols);
    let hits = hits_of(&chunks);
    (paint(&chunks, chrome, hover), hits)
}

fn status_chunks(mode: &str, cols: usize) -> Vec<Chunk> {
    if cols == 0 {
        return Vec::new();
    }

    if mode == "normal" || mode == "locked" {
        let options = indicator(mode)
            .map(|items| variants(Vec::new(), &items, false))
            .unwrap_or_default();
        return first_fit(&options, cols);
    }

    // The mode word is already on the left of the row. Debug of the mode, in
    // uppercase: PANE, TAB, RESIZE, MOVE, SCROLL, SESSION, TMUX. Scroll's word
    // is SCROLL. Session's word is SESSION. Do not repeat either name here.
    if matches!(
        mode,
        "pane" | "tab" | "resize" | "move" | "scroll" | "session" | "tmux"
    ) {
        return first_fit(&variants(Vec::new(), &mode_actions(mode), true), cols);
    }

    let mut remain = cols;
    let sep = if let Some(title) = help_title(mode) {
        let text = format!(" {title}  ");
        if text.chars().count() <= remain {
            Some(vec![Chunk::orange(text)])
        } else if remain >= 2 {
            Some(vec![Chunk::plain("  ")])
        } else {
            None
        }
    } else if remain >= 2 {
        Some(vec![Chunk::plain("  ")])
    } else {
        None
    };
    let Some(sep) = sep else {
        return Vec::new();
    };
    remain = remain.saturating_sub(width(&sep));
    let actions = first_fit(&variants(Vec::new(), &mode_actions(mode), true), remain);
    if actions.is_empty() {
        Vec::new()
    } else {
        let mut out = sep;
        out.extend(actions);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visible(rendered: &str) -> String {
        let mut out = String::new();
        let mut chars = rendered.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    for c in chars.by_ref() {
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                continue;
            }
            out.push(ch);
        }
        out
    }

    fn cell_sgr(rendered: &str) -> Vec<String> {
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

    fn slice(text: &str, hit: &Hit) -> String {
        text.chars()
            .skip(hit.start)
            .take(hit.end - hit.start)
            .collect()
    }

    #[test]
    fn labeled_chip_includes_the_side_spaces() {
        let (line, hits) = render_bar_with("normal", 400, &Chrome::default(), None);
        let text = visible(&line);
        let resize = hits
            .iter()
            .find(|hit| hit.click == Click::Mode("resize"))
            .expect("resize");
        assert_eq!(slice(&text, resize), " ^N Resize ");
        let tab = hits
            .iter()
            .find(|hit| hit.click == Click::Mode("tab"))
            .expect("tab");
        assert_eq!(tab.end, resize.start);
    }

    #[test]
    fn hover_reverses_the_whole_labeled_chip() {
        let chrome = Chrome {
            bar: Ink::Bit(232),
            ..Chrome::default()
        };
        let reverse = format!("22;{};{}", chrome.dark.fg(), chrome.light.bg());
        let (idle, hits) = render_bar_with("normal", 400, &chrome, None);
        let resize = hits
            .iter()
            .find(|hit| hit.click == Click::Mode("resize"))
            .unwrap();
        assert_ne!(cell_sgr(&idle)[resize.start + 4], reverse);
        for col in resize.start..resize.end {
            let cells = cell_sgr(&render_bar_with("normal", 400, &chrome, Some(col)).0);
            assert!(
                cells[resize.start..resize.end]
                    .iter()
                    .all(|cell| cell == &reverse),
                "hover {col} left part of the chip unreversed"
            );
            if resize.start > 0 {
                assert_ne!(cells[resize.start - 1], reverse);
            }
            if resize.end < cells.len() {
                assert_ne!(cells[resize.end], reverse);
            }
        }
    }

    fn side_pads(slice: &str) -> (usize, usize) {
        let lead = slice.chars().take_while(|ch| *ch == ' ').count();
        let trail = slice.chars().rev().take_while(|ch| *ch == ' ').count();
        (lead, trail)
    }

    #[test]
    fn every_labeled_chip_has_one_space_on_each_side() {
        let chrome = Chrome {
            bar: Ink::Bit(232),
            ..Chrome::default()
        };
        let reverse = format!("22;{};{}", chrome.dark.fg(), chrome.light.bg());
        for mode in [
            "normal",
            "locked",
            "pane",
            "tab",
            "resize",
            "move",
            "scroll",
            "session",
            "tmux",
            "search",
            "enter_search",
            "rename_pane",
            "rename_tab",
        ] {
            let (line, hits) = render_bar_with(mode, 400, &chrome, None);
            let text = visible(&line);
            assert!(text.chars().count() <= 400, "{mode} {text}");
            for hit in &hits {
                let body = slice(&text, hit);
                if body.contains(' ') {
                    assert_eq!(side_pads(&body), (1, 1), "{mode} {body} in {text}");
                } else {
                    assert_eq!(side_pads(&body), (0, 0), "{mode} {body}");
                }
            }
            let labeled: Vec<_> = hits
                .iter()
                .filter(|hit| slice(&text, hit).contains(' '))
                .copied()
                .collect();
            for hit in labeled {
                let cells = cell_sgr(&render_bar_with(mode, 400, &chrome, Some(hit.start)).0);
                assert!(
                    cells[hit.start..hit.end]
                        .iter()
                        .all(|cell| cell == &reverse),
                    "{mode} hover did not light the whole chip"
                );
                if hit.start > 0 {
                    assert_ne!(
                        cells[hit.start - 1],
                        reverse,
                        "{mode} lit the left neighbor"
                    );
                }
                if hit.end < cells.len() {
                    assert_ne!(cells[hit.end], reverse, "{mode} lit the right neighbor");
                }
            }
        }

        let (pane, pane_hits) = render_bar_with("pane", 400, &Chrome::default(), None);
        let pane_text = visible(&pane);
        let new = pane_hits
            .iter()
            .find(|hit| hit.click == Click::NewPane)
            .unwrap();
        assert_eq!(slice(&pane_text, new), " N New ");
        assert!(pane_hits
            .iter()
            .all(|hit| hit.end <= new.end || hit.start > new.end));

        let (tab, tab_hits) = render_bar_with("tab", 400, &Chrome::default(), None);
        let tab_text = visible(&tab);
        let enter = tab_hits
            .iter()
            .find(|hit| hit.click == Click::SelectPane)
            .unwrap();
        assert_eq!(slice(&tab_text, enter), " ENTER Select Pane ");
        assert!(
            tab_hits
                .iter()
                .all(|hit| hit.end <= enter.end || hit.start > enter.end),
            "a second space after Select Pane is inside a hit"
        );
        assert_eq!(tab_text.chars().nth(enter.end), Some(' '));

        let (scroll, scroll_hits) = render_bar_with("scroll", 400, &Chrome::default(), None);
        let scroll_text = visible(&scroll);
        let search = scroll_hits
            .iter()
            .find(|hit| hit.click == Click::EnterSearch)
            .unwrap();
        assert_eq!(slice(&scroll_text, search), " S Enter Search Term ");
        assert!(scroll_hits
            .iter()
            .all(|hit| hit.end <= search.end || hit.start > search.end));

        let (session, session_hits) = render_bar_with("session", 400, &Chrome::default(), None);
        let session_text = visible(&session);
        let detach = session_hits
            .iter()
            .find(|hit| hit.click == Click::Detach)
            .unwrap();
        assert_eq!(slice(&session_text, detach), " D Detach ");
        assert!(session_hits.iter().any(|hit| hit.start == detach.end));
    }

    #[test]
    fn key_only_stays_the_key_and_turns_normal_on_hover() {
        let chrome = Chrome {
            bar: Ink::Bit(232),
            ..Chrome::default()
        };
        let mut found = false;
        for cols in 8..40 {
            let (line, hits) = render_bar_with("normal", cols, &chrome, None);
            let text = visible(&line);
            if !text.contains("^G  ^P") || text.contains("Lock") {
                continue;
            }
            let lock = hits
                .iter()
                .find(|hit| hit.click == Click::Mode("locked"))
                .expect("lock");
            assert_eq!(slice(&text, lock), "^G");
            let normal = format!("22;{};{}", chrome.light.fg(), chrome.dark.bg());
            let plain = format!("22;{};{}", chrome.light.fg(), chrome.bar.bg());
            let cells = cell_sgr(&render_bar_with("normal", cols, &chrome, Some(lock.start)).0);
            assert_eq!(cells[lock.start], normal);
            assert_eq!(cells[lock.end], plain);
            found = true;
            break;
        }
        assert!(found, "no key-only row");
    }

    #[test]
    fn letter_combo_hits_stay_one_cell() {
        let (line, hits) = render_bar_with("pane", 400, &Chrome::default(), None);
        let text = visible(&line);
        assert!(text.contains("HJKL"), "{text}");
        assert!(!text.starts_with("Pane"), "{text}");
        let letters: Vec<_> = hits
            .iter()
            .filter(|hit| {
                matches!(
                    hit.click,
                    Click::FocusLeft | Click::FocusDown | Click::FocusUp | Click::FocusRight
                )
            })
            .collect();
        assert_eq!(letters.len(), 4);
        for hit in letters {
            assert_eq!(hit.end - hit.start, 1);
        }
    }
}
